use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

use conhub_models::chunking::{IngestChunksRequest, Chunk};

use crate::models::{Entity, EntityType, DataSource};
use crate::entity_resolution::{EntityResolver, ResolutionConfig};
use crate::knowledge_fusion::FusionEngine;
use crate::extractors::code_entities::CodeEntityExtractor;
use crate::errors::GraphResult;

pub struct ChunkProcessor {
    db_pool: PgPool,
}

#[derive(Debug, Default)]
pub struct ProcessingStats {
    pub chunks_processed: usize,
    pub entities_created: usize,
    pub relationships_created: usize,
}

impl ChunkProcessor {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Process a batch of chunks and extract entities/relationships
    pub async fn process_chunks(
        &self,
        request: &IngestChunksRequest,
    ) -> GraphResult<ProcessingStats> {
        let mut stats = ProcessingStats::default();

        // Initialize entity resolver and fusion engine
        let config = ResolutionConfig::default();
        let resolver = EntityResolver::new(self.db_pool.clone(), config);
        let fusion_engine = FusionEngine::new(self.db_pool.clone(), resolver);

        // Initialize extractors based on source kind
        let code_extractor = CodeEntityExtractor::new();

        for chunk in &request.chunks {
            info!("Processing chunk {}", chunk.chunk_id);

            // Extract entities based on content type
            let entities = Self::extract_entities_from_chunk(chunk, &code_extractor)?;

            if !entities.is_empty() {
                info!("Extracted {} entities from chunk", entities.len());

                // Fuse entities into the graph (includes deduplication)
                match fusion_engine.fuse_entities(entities).await {
                    Ok(canonical_ids) => {
                        stats.entities_created += canonical_ids.len();
                    }
                    Err(e) => {
                        warn!("Failed to fuse entities for chunk {}: {}", chunk.chunk_id, e);
                    }
                }

                // Link chunk to entities (store chunk_id → entity_id mappings)
                if let Err(e) = self.link_chunk_to_entities(chunk.chunk_id).await {
                    warn!("Failed to link chunk to entities: {}", e);
                }
            }

            stats.chunks_processed += 1;
        }

        Ok(stats)
    }

    /// Extract entities from a chunk based on its type
    fn extract_entities_from_chunk(
        chunk: &Chunk,
        code_extractor: &CodeEntityExtractor,
    ) -> GraphResult<Vec<Entity>> {
        let mut entities = Vec::new();

        // Determine extraction strategy based on block_type and language
        if let Some(block_type) = &chunk.block_type {
            match block_type.as_str() {
                "code" => {
                    // Use code entity extractor
                    let extracted = code_extractor.extract(&chunk.content, chunk.language.as_deref());
                    
                    // Convert to Entity structs
                    for extracted_entity in extracted {
                        let source = Self::determine_data_source(chunk);
                        
                        let mut properties = serde_json::Map::new();
                        properties.insert("name".to_string(), serde_json::json!(extracted_entity.name));
                        properties.insert("chunk_id".to_string(), serde_json::json!(chunk.chunk_id));
                        
                        if let Some(lang) = &chunk.language {
                            properties.insert("language".to_string(), serde_json::json!(lang));
                        }
                        
                        // Copy chunk metadata
                        if let Some(obj) = chunk.metadata.as_object() {
                            for (k, v) in obj {
                                properties.insert(k.clone(), v.clone());
                            }
                        }

                        entities.push(Entity::new(
                            extracted_entity.entity_type,
                            source,
                            chunk.chunk_id.to_string(),
                            extracted_entity.name,
                            properties.into_iter().collect(),
                        ));
                    }
                }
                "text" | "chat" => {
                    // For text/chat, we could extract named entities (people, orgs, etc.)
                    // For now, keep it simple - just store the chunk reference
                    // Future: add NER here
                }
                _ => {}
            }
        }

        Ok(entities)
    }

    fn determine_data_source(chunk: &Chunk) -> DataSource {
        // Try to determine source from metadata
        if let Some(connector) = chunk.metadata.get("connector").and_then(|v| v.as_str()) {
            match connector {
                "github" => DataSource::GitHub,
                "slack" => DataSource::Slack,
                "google_drive" => DataSource::GoogleDrive,
                "dropbox" => DataSource::Dropbox,
                "bitbucket" => DataSource::Bitbucket,
                "url_crawler" => DataSource::UrlCrawler,
                _ => DataSource::GitHub, // default
            }
        } else {
            DataSource::GitHub
        }
    }

    /// Link a chunk to extracted entities (for future retrieval)
    async fn link_chunk_to_entities(&self, chunk_id: Uuid) -> GraphResult<()> {
        // This would store chunk_id → entity_id mappings in a chunk_entities table
        // For now, we'll skip this as it requires additional schema
        // The entities themselves contain chunk_id in their properties
        Ok(())
    }
}
