use anyhow::{Context, Result};
use regex::Regex;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use tracing::{info, warn, error};
use uuid::Uuid;

/// Entity types that can be extracted
#[derive(Debug, Clone)]
pub enum EntityType {
    CodeSymbol,    // Functions, classes, methods
    ApiEndpoint,   // REST endpoints, GraphQL queries
    File,          // File paths and modules
    Ticket,        // JIRA, GitHub issues
    PullRequest,   // PR/MR references
    Feature,       // Feature names
    Service,       // Microservice names
}

impl EntityType {
    pub fn as_str(&self) -> &str {
        match self {
            EntityType::CodeSymbol => "code_symbol",
            EntityType::ApiEndpoint => "api_endpoint",
            EntityType::File => "file",
            EntityType::Ticket => "ticket",
            EntityType::PullRequest => "pull_request",
            EntityType::Feature => "feature",
            EntityType::Service => "service",
        }
    }
}

/// Extracted entity with metadata
#[derive(Debug, Clone)]
pub struct ExtractedEntity {
    pub entity_type: EntityType,
    pub canonical_name: String,
    pub normalized_name: String,
    pub confidence: f32,
    pub context_snippet: Option<String>,
    pub start_position: Option<usize>,
    pub end_position: Option<usize>,
    pub metadata: serde_json::Value,
}

/// Service for building and managing the relationship graph
pub struct RelationshipService {
    db_pool: PgPool,
    // Compiled regex patterns for entity extraction
    function_pattern: Regex,
    api_endpoint_pattern: Regex,
    file_path_pattern: Regex,
    ticket_pattern: Regex,
    pr_pattern: Regex,
}

impl RelationshipService {
    pub fn new(db_pool: PgPool) -> Result<Self> {
        // Compile regex patterns for entity extraction
        let function_pattern = Regex::new(
            r"(?:fn|func|function|def|class|interface|trait|impl|struct|enum)\s+([a-zA-Z_][a-zA-Z0-9_]*)"
        )?;
        
        let api_endpoint_pattern = Regex::new(
            r"(?:GET|POST|PUT|PATCH|DELETE|HEAD|OPTIONS)\s+(/[a-zA-Z0-9_/\-{}:]*)"
        )?;
        
        let file_path_pattern = Regex::new(
            r"(?:src/|lib/|app/|tests?/|docs?/)([a-zA-Z0-9_/\-\.]+\.[a-z]{1,4})"
        )?;
        
        let ticket_pattern = Regex::new(
            r"([A-Z]{2,10}-\d+)"
        )?;
        
        let pr_pattern = Regex::new(
            r"(?:PR|MR|#)(\d+)"
        )?;
        
        Ok(Self {
            db_pool,
            function_pattern,
            api_endpoint_pattern,
            file_path_pattern,
            ticket_pattern,
            pr_pattern,
        })
    }
    
    /// Extract entities from a chunk of text
    pub fn extract_entities(&self, text: &str, connector_type: &str) -> Vec<ExtractedEntity> {
        let mut entities = Vec::new();
        
        // Extract code symbols (functions, classes, etc.)
        for cap in self.function_pattern.captures_iter(text) {
            if let Some(name_match) = cap.get(1) {
                let name = name_match.as_str();
                entities.push(ExtractedEntity {
                    entity_type: EntityType::CodeSymbol,
                    canonical_name: name.to_string(),
                    normalized_name: name.to_lowercase(),
                    confidence: 0.9,
                    context_snippet: Some(cap.get(0).unwrap().as_str().to_string()),
                    start_position: Some(name_match.start()),
                    end_position: Some(name_match.end()),
                    metadata: serde_json::json!({
                        "connector_type": connector_type
                    }),
                });
            }
        }
        
        // Extract API endpoints
        for cap in self.api_endpoint_pattern.captures_iter(text) {
            if let Some(endpoint_match) = cap.get(1) {
                let endpoint = endpoint_match.as_str();
                entities.push(ExtractedEntity {
                    entity_type: EntityType::ApiEndpoint,
                    canonical_name: endpoint.to_string(),
                    normalized_name: endpoint.to_lowercase(),
                    confidence: 0.95,
                    context_snippet: Some(cap.get(0).unwrap().as_str().to_string()),
                    start_position: Some(endpoint_match.start()),
                    end_position: Some(endpoint_match.end()),
                    metadata: serde_json::json!({
                        "connector_type": connector_type
                    }),
                });
            }
        }
        
        // Extract file paths
        for cap in self.file_path_pattern.captures_iter(text) {
            if let Some(file_match) = cap.get(1) {
                let file_path = file_match.as_str();
                entities.push(ExtractedEntity {
                    entity_type: EntityType::File,
                    canonical_name: file_path.to_string(),
                    normalized_name: file_path.to_lowercase(),
                    confidence: 0.85,
                    context_snippet: Some(cap.get(0).unwrap().as_str().to_string()),
                    start_position: Some(file_match.start()),
                    end_position: Some(file_match.end()),
                    metadata: serde_json::json!({
                        "connector_type": connector_type
                    }),
                });
            }
        }
        
        // Extract ticket references
        for cap in self.ticket_pattern.captures_iter(text) {
            if let Some(ticket_match) = cap.get(1) {
                let ticket = ticket_match.as_str();
                entities.push(ExtractedEntity {
                    entity_type: EntityType::Ticket,
                    canonical_name: ticket.to_string(),
                    normalized_name: ticket.to_lowercase(),
                    confidence: 0.9,
                    context_snippet: Some(cap.get(0).unwrap().as_str().to_string()),
                    start_position: Some(ticket_match.start()),
                    end_position: Some(ticket_match.end()),
                    metadata: serde_json::json!({
                        "connector_type": connector_type
                    }),
                });
            }
        }
        
        // Extract PR/MR references
        for cap in self.pr_pattern.captures_iter(text) {
            if let Some(pr_match) = cap.get(1) {
                let pr_num = pr_match.as_str();
                entities.push(ExtractedEntity {
                    entity_type: EntityType::PullRequest,
                    canonical_name: format!("#{}", pr_num),
                    normalized_name: pr_num.to_lowercase(),
                    confidence: 0.9,
                    context_snippet: Some(cap.get(0).unwrap().as_str().to_string()),
                    start_position: Some(pr_match.start()),
                    end_position: Some(pr_match.end()),
                    metadata: serde_json::json!({
                        "connector_type": connector_type
                    }),
                });
            }
        }
        
        entities
    }
    
    /// Process a document and its chunks to extract and link entities
    pub async fn process_document(&self, document_id: Uuid, service_name: Option<&str>) -> Result<()> {
        info!("🔍 Processing document {} for relationship extraction", document_id);
        
        // Get all chunks for the document
        let chunks = sqlx::query!(
            r#"
            SELECT id, content, metadata
            FROM document_chunks
            WHERE document_id = $1
            ORDER BY chunk_number
            "#,
            document_id
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        // Get document metadata for context
        let doc = sqlx::query!(
            r#"
            SELECT connector_type, name, path
            FROM source_documents
            WHERE id = $1
            "#,
            document_id
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        let connector_type = doc.connector_type;
        
        for chunk_row in chunks {
            let chunk_id = chunk_row.id;
            let content = &chunk_row.content;
            
            // Extract entities from chunk
            let extracted = self.extract_entities(content, &connector_type);
            
            info!("  📊 Extracted {} entities from chunk {}", extracted.len(), chunk_id);
            
            // Resolve and link entities
            for entity in extracted {
                match self.resolve_or_create_entity(
                    &entity,
                    service_name,
                    None, // language - could be extracted from metadata
                ).await {
                    Ok(entity_id) => {
                        // Link chunk to entity
                        if let Err(e) = self.link_chunk_to_entity(
                            chunk_id,
                            entity_id,
                            "mentions",
                            entity.confidence,
                            &entity,
                        ).await {
                            warn!("Failed to link chunk to entity: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to resolve entity '{}': {}", entity.canonical_name, e);
                    }
                }
            }
        }
        
        info!("✅ Completed relationship extraction for document {}", document_id);
        Ok(())
    }
    
    /// Resolve an entity to an existing ID or create a new one
    async fn resolve_or_create_entity(
        &self,
        entity: &ExtractedEntity,
        service_name: Option<&str>,
        language: Option<&str>,
    ) -> Result<Uuid> {
        // Try to find existing entity
        let existing = sqlx::query!(
            r#"
            SELECT id FROM entities
            WHERE entity_type = $1
              AND normalized_name = $2
              AND (service_name = $3 OR service_name IS NULL)
              AND (language = $4 OR language IS NULL)
            LIMIT 1
            "#,
            entity.entity_type.as_str(),
            entity.normalized_name,
            service_name,
            language
        )
        .fetch_optional(&self.db_pool)
        .await?;
        
        if let Some(row) = existing {
            // Update last_seen and increment count
            sqlx::query!(
                r#"
                UPDATE entities
                SET last_seen_at = CURRENT_TIMESTAMP,
                    occurrence_count = occurrence_count + 1
                WHERE id = $1
                "#,
                row.id
            )
            .execute(&self.db_pool)
            .await?;
            
            Ok(row.id)
        } else {
            // Create new entity
            let id = Uuid::new_v4();
            sqlx::query!(
                r#"
                INSERT INTO entities (
                    id, entity_type, canonical_name, normalized_name,
                    service_name, language, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
                id,
                entity.entity_type.as_str(),
                entity.canonical_name,
                entity.normalized_name,
                service_name,
                language,
                entity.metadata
            )
            .execute(&self.db_pool)
            .await?;
            
            Ok(id)
        }
    }
    
    /// Link a chunk to an entity
    async fn link_chunk_to_entity(
        &self,
        chunk_id: Uuid,
        entity_id: Uuid,
        relation_type: &str,
        confidence: f32,
        entity: &ExtractedEntity,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO chunk_entities (
                chunk_id, entity_id, relation_type, confidence,
                extraction_method, context_snippet,
                start_position, end_position
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (chunk_id, entity_id, relation_type) DO UPDATE
            SET confidence = GREATEST(chunk_entities.confidence, EXCLUDED.confidence)
            "#,
            chunk_id,
            entity_id,
            relation_type,
            confidence,
            "regex",
            entity.context_snippet,
            entity.start_position.map(|p| p as i32),
            entity.end_position.map(|p| p as i32)
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }
    
    /// Create chunk-to-chunk relationships based on embedding similarity
    pub async fn link_similar_chunks(&self, chunk_id: Uuid, similarity_threshold: f32) -> Result<()> {
        // This would query the vector database (Qdrant) for similar chunks
        // and create chunk_relations entries
        // For now, this is a placeholder
        info!("🔗 Would link similar chunks for chunk {}", chunk_id);
        Ok(())
    }
    
    /// Get all entities referenced by a chunk
    pub async fn get_chunk_entities(&self, chunk_id: Uuid) -> Result<Vec<(Uuid, String, String, f32)>> {
        let results = sqlx::query!(
            r#"
            SELECT e.id, e.entity_type, e.canonical_name, ce.confidence
            FROM chunk_entities ce
            JOIN entities e ON ce.entity_id = e.id
            WHERE ce.chunk_id = $1
            ORDER BY ce.confidence DESC
            "#,
            chunk_id
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        Ok(results
            .into_iter()
            .map(|r| (r.id, r.entity_type, r.canonical_name, r.confidence as f32))
            .collect())
    }
    
    /// Get all chunks that reference the same entity
    pub async fn get_related_chunks_via_entity(&self, chunk_id: Uuid) -> Result<Vec<Uuid>> {
        let results = sqlx::query!(
            r#"
            SELECT DISTINCT ce2.chunk_id
            FROM chunk_entities ce1
            JOIN chunk_entities ce2 ON ce1.entity_id = ce2.entity_id
            WHERE ce1.chunk_id = $1 AND ce2.chunk_id != $1
            "#,
            chunk_id
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        Ok(results.into_iter().map(|r| r.chunk_id).collect())
    }
}
