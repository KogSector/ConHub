use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use tracing::{info, warn, error};
use uuid::Uuid;

use conhub_models::chunking::{ObserveChunksRequest, ObserveChunksResponse, ChunkRef};

use crate::services::chunk_repository::ChunkRepository;
use crate::services::chunk_processor::ChunkProcessor;
use crate::models::{Entity, EntityType, DataSource};
use crate::extractors::code_entities::CodeEntityExtractor;
use crate::errors::GraphResult;

/// Handler for observing chunks (IDs-only) for graph extraction.
/// This implements Option A: graph_rag fetches chunk text from Postgres by chunk_id.
pub async fn observe_chunks(
    payload: web::Json<ObserveChunksRequest>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    info!(
        "🔍 [Graph RAG] Observing {} chunks from source {} (tenant: {})",
        payload.chunks.len(),
        payload.source_id,
        payload.tenant_id
    );

    if payload.chunks.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "No chunks provided"
        }));
    }

    // Initialize chunk repository to fetch text from Postgres
    let chunk_repo = ChunkRepository::new(pool.get_ref().clone());

    // Collect chunk IDs
    let chunk_ids: Vec<Uuid> = payload.chunks.iter().map(|c| c.chunk_id).collect();

    // Fetch chunk texts from Postgres (single source of truth)
    let chunk_texts = match chunk_repo.fetch_by_ids(&chunk_ids).await {
        Ok(texts) => texts,
        Err(e) => {
            error!("❌ Failed to fetch chunks from Postgres: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to fetch chunk texts: {}", e)
            }));
        }
    };

    if chunk_texts.is_empty() {
        warn!("⚠️ No chunks found in Postgres for observation");
        return HttpResponse::Ok().json(ObserveChunksResponse {
            total_chunks: payload.chunks.len(),
            chunks_processed: 0,
            entities_created: 0,
            relationships_created: 0,
            evidence_created: 0,
        });
    }

    // Process chunks for entity/relationship extraction
    let processor = ObserveChunkProcessor::new(pool.get_ref().clone());
    
    match processor.process(&payload, &chunk_texts).await {
        Ok(stats) => {
            info!(
                "✅ Chunk observation complete: {} entities, {} relationships, {} evidence from {} chunks",
                stats.entities_created,
                stats.relationships_created,
                stats.evidence_created,
                stats.chunks_processed
            );

            HttpResponse::Ok().json(ObserveChunksResponse {
                total_chunks: payload.chunks.len(),
                chunks_processed: stats.chunks_processed,
                entities_created: stats.entities_created,
                relationships_created: stats.relationships_created,
                evidence_created: stats.evidence_created,
            })
        }
        Err(e) => {
            error!("❌ Failed to process observed chunks: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to process chunks: {}", e)
            }))
        }
    }
}

/// Processing stats for observation
#[derive(Debug, Default)]
pub struct ObserveStats {
    pub chunks_processed: usize,
    pub entities_created: usize,
    pub relationships_created: usize,
    pub evidence_created: usize,
}

/// Processor for observed chunks (IDs-only with text fetched from Postgres)
pub struct ObserveChunkProcessor {
    db_pool: PgPool,
}

impl ObserveChunkProcessor {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Process observed chunks: extract entities/relationships and write evidence
    pub async fn process(
        &self,
        request: &ObserveChunksRequest,
        chunk_texts: &std::collections::HashMap<Uuid, conhub_models::chunking::ChunkText>,
    ) -> GraphResult<ObserveStats> {
        let mut stats = ObserveStats::default();
        let code_extractor = CodeEntityExtractor::new();

        for chunk_ref in &request.chunks {
            // Get the fetched text for this chunk
            let chunk_text = match chunk_texts.get(&chunk_ref.chunk_id) {
                Some(text) => text,
                None => {
                    warn!("⚠️ Chunk {} not found in fetched texts, skipping", chunk_ref.chunk_id);
                    continue;
                }
            };

            // Extract entities based on block type
            let entities = self.extract_entities(chunk_ref, &chunk_text.content, &code_extractor)?;

            if !entities.is_empty() {
                info!("📊 Extracted {} entities from chunk {}", entities.len(), chunk_ref.chunk_id);

                // Insert entities and create evidence
                for entity in &entities {
                    match self.upsert_entity_with_evidence(entity, chunk_ref.chunk_id).await {
                        Ok(created) => {
                            if created {
                                stats.entities_created += 1;
                            }
                            stats.evidence_created += 1;
                        }
                        Err(e) => {
                            warn!("⚠️ Failed to upsert entity: {}", e);
                        }
                    }
                }
            }

            stats.chunks_processed += 1;
        }

        Ok(stats)
    }

    /// Extract entities from chunk content based on block type
    fn extract_entities(
        &self,
        chunk_ref: &ChunkRef,
        content: &str,
        code_extractor: &CodeEntityExtractor,
    ) -> GraphResult<Vec<ExtractedEntity>> {
        let mut entities = Vec::new();

        // Determine extraction strategy based on block_type
        if let Some(block_type) = &chunk_ref.block_type {
            match block_type.as_str() {
                "code" => {
                    let extracted = code_extractor.extract(content, chunk_ref.language.as_deref());
                    for ext in extracted {
                        entities.push(ExtractedEntity {
                            entity_type: ext.entity_type,
                            name: ext.name,
                            normalized_name: ext.name.to_lowercase(),
                            language: chunk_ref.language.clone(),
                            metadata: chunk_ref.metadata.clone(),
                        });
                    }
                }
                "text" | "heading" | "comment" => {
                    // Future: NER for text chunks (people, orgs, concepts)
                    // For now, skip non-code chunks
                }
                _ => {}
            }
        }

        Ok(entities)
    }

    /// Upsert an entity and create evidence linking it to the chunk
    async fn upsert_entity_with_evidence(
        &self,
        entity: &ExtractedEntity,
        chunk_id: Uuid,
    ) -> GraphResult<bool> {
        // Upsert entity (get or create)
        let entity_id = sqlx::query_scalar!(
            r#"
            INSERT INTO entities (entity_type, canonical_name, normalized_name, language, metadata)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (entity_type, normalized_name, service_name, language) 
            DO UPDATE SET 
                last_seen_at = CURRENT_TIMESTAMP,
                occurrence_count = entities.occurrence_count + 1
            RETURNING id
            "#,
            entity.entity_type.as_str(),
            entity.name,
            entity.normalized_name,
            entity.language,
            entity.metadata
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| crate::errors::GraphError::DatabaseError(e.to_string()))?;

        // Create evidence linking entity to chunk
        sqlx::query!(
            r#"
            INSERT INTO entity_evidence (entity_id, chunk_id, confidence, extraction_method)
            VALUES ($1, $2, 1.0, 'ast')
            ON CONFLICT (entity_id, chunk_id) DO NOTHING
            "#,
            entity_id,
            chunk_id
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| crate::errors::GraphError::DatabaseError(e.to_string()))?;

        Ok(true)
    }
}

/// Extracted entity before persistence
#[derive(Debug)]
struct ExtractedEntity {
    entity_type: EntityType,
    name: String,
    normalized_name: String,
    language: Option<String>,
    metadata: serde_json::Value,
}
