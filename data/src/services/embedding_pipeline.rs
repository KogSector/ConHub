use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::PgPool;
use anyhow::{Result, Context};
use std::sync::Arc;
use tracing::{info, warn, error};

use super::EmbeddingClient;

/// Document content for embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentForEmbedding {
    pub id: Uuid,
    pub name: String,
    pub content: String,
    pub content_type: String,
    pub metadata: serde_json::Value,
}

/// Embedding pipeline that processes documents from the queue
pub struct EmbeddingPipeline {
    pool: PgPool,
    embedding_client: Arc<EmbeddingClient>,
    batch_size: usize,
}

impl EmbeddingPipeline {
    pub fn new(pool: PgPool, embedding_client: Arc<EmbeddingClient>) -> Self {
        Self {
            pool,
            embedding_client,
            batch_size: 10,
        }
    }
    
    /// Process pending documents from the embedding queue
    pub async fn process_queue(&self) -> Result<usize> {
        // Fetch pending documents from queue
        let pending_docs = sqlx::query!(
            r#"
            SELECT 
                eq.id as queue_id,
                eq.document_id,
                sd.name,
                sd.content_type,
                sd.metadata
            FROM embedding_queue eq
            INNER JOIN source_documents sd ON sd.id = eq.document_id
            WHERE eq.status = 'pending' 
            AND eq.retry_count < 3
            ORDER BY eq.created_at ASC
            LIMIT $1
            "#,
            self.batch_size as i64
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch pending documents")?;
        
        if pending_docs.is_empty() {
            return Ok(0);
        }
        
        info!("Processing {} documents from embedding queue", pending_docs.len());
        
        let mut processed = 0;
        
        for doc_row in pending_docs {
            // Update status to processing
            sqlx::query!(
                "UPDATE embedding_queue SET status = 'processing', updated_at = CURRENT_TIMESTAMP WHERE id = $1",
                doc_row.queue_id
            )
            .execute(&self.pool)
            .await?;
            
            // Fetch document content
            match self.fetch_document_content(doc_row.document_id).await {
                Ok(content) => {
                    // Process and embed the document
                    match self.embed_document(
                        doc_row.document_id,
                        &doc_row.name,
                        &content,
                        doc_row.content_type.as_deref(),
                        &doc_row.metadata.unwrap_or(serde_json::json!({}))
                    ).await {
                        Ok(_) => {
                            // Mark as completed
                            sqlx::query!(
                                r#"
                                UPDATE embedding_queue 
                                SET status = 'completed', processed_at = CURRENT_TIMESTAMP 
                                WHERE id = $1
                                "#,
                                doc_row.queue_id
                            )
                            .execute(&self.pool)
                            .await?;
                            
                            // Update document indexed_at timestamp
                            sqlx::query!(
                                "UPDATE source_documents SET indexed_at = CURRENT_TIMESTAMP WHERE id = $1",
                                doc_row.document_id
                            )
                            .execute(&self.pool)
                            .await?;
                            
                            processed += 1;
                            info!("Successfully embedded document: {}", doc_row.name);
                        }
                        Err(e) => {
                            error!("Failed to embed document {}: {}", doc_row.name, e);
                            
                            // Update with error
                            sqlx::query!(
                                r#"
                                UPDATE embedding_queue 
                                SET status = 'failed', 
                                    retry_count = retry_count + 1,
                                    error_message = $1,
                                    updated_at = CURRENT_TIMESTAMP
                                WHERE id = $2
                                "#,
                                e.to_string(),
                                doc_row.queue_id
                            )
                            .execute(&self.pool)
                            .await?;
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to fetch content for document {}: {}", doc_row.name, e);
                    
                    sqlx::query!(
                        r#"
                        UPDATE embedding_queue 
                        SET status = 'failed',
                            retry_count = retry_count + 1,
                            error_message = $1,
                            updated_at = CURRENT_TIMESTAMP
                        WHERE id = $2
                        "#,
                        e.to_string(),
                        doc_row.queue_id
                    )
                    .execute(&self.pool)
                    .await?;
                }
            }
        }
        
        Ok(processed)
    }
    
    /// Fetch document content from storage
    async fn fetch_document_content(&self, document_id: Uuid) -> Result<String> {
        // TODO: Implement actual content fetching based on connector type
        // This should use the connector to fetch the actual document content
        // For now, return a placeholder
        
        let doc = sqlx::query!(
            "SELECT connector_type, external_id, url FROM source_documents WHERE id = $1",
            document_id
        )
        .fetch_one(&self.pool)
        .await?;
        
        // Placeholder - in production, this would fetch from the actual source
        Ok(format!("Content of document from {}", doc.connector_type))
    }
    
    /// Embed a document and store in vector database
    async fn embed_document(
        &self,
        document_id: Uuid,
        name: &str,
        content: &str,
        content_type: Option<&str>,
        metadata: &serde_json::Value,
    ) -> Result<()> {
        // Chunk the document content
        let chunks = self.chunk_content(content, content_type);
        
        info!("Document {} chunked into {} pieces", name, chunks.len());
        
        // Embed each chunk
        for (idx, chunk) in chunks.iter().enumerate() {
            // Call embedding service
            let embedding = self.embedding_client
                .embed_text(chunk)
                .await
                .context("Failed to generate embedding")?;
            
            // Store in vector database (Qdrant)
            // TODO: Implement Qdrant storage
            info!("Embedded chunk {}/{} of document {}", idx + 1, chunks.len(), name);
        }
        
        Ok(())
    }
    
    /// Chunk document content for embedding
    fn chunk_content(&self, content: &str, content_type: Option<&str>) -> Vec<String> {
        // Simple chunking strategy - split by paragraphs or fixed size
        // In production, use more sophisticated chunking based on content type
        
        const CHUNK_SIZE: usize = 1000;
        const CHUNK_OVERLAP: usize = 200;
        
        let mut chunks = Vec::new();
        let content_bytes = content.as_bytes();
        
        let mut start = 0;
        while start < content_bytes.len() {
            let end = std::cmp::min(start + CHUNK_SIZE, content_bytes.len());
            
            // Try to find a good break point (space, newline, etc.)
            let chunk_end = if end < content_bytes.len() {
                // Look for a space or newline near the end
                content_bytes[start..end]
                    .iter()
                    .rposition(|&b| b == b' ' || b == b'\n')
                    .map(|pos| start + pos)
                    .unwrap_or(end)
            } else {
                end
            };
            
            if let Ok(chunk) = String::from_utf8(content_bytes[start..chunk_end].to_vec()) {
                if !chunk.trim().is_empty() {
                    chunks.push(chunk);
                }
            }
            
            // Move start forward with overlap
            start = if chunk_end < content_bytes.len() {
                chunk_end.saturating_sub(CHUNK_OVERLAP)
            } else {
                content_bytes.len()
            };
            
            // Avoid infinite loop
            if start >= content_bytes.len() {
                break;
            }
        }
        
        chunks
    }
    
    /// Get queue statistics
    pub async fn get_queue_stats(&self) -> Result<QueueStats> {
        let stats = sqlx::query_as!(
            QueueStats,
            r#"
            SELECT 
                COUNT(*) FILTER (WHERE status = 'pending') as "pending!",
                COUNT(*) FILTER (WHERE status = 'processing') as "processing!",
                COUNT(*) FILTER (WHERE status = 'completed') as "completed!",
                COUNT(*) FILTER (WHERE status = 'failed') as "failed!"
            FROM embedding_queue
            "#
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(stats)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueueStats {
    pub pending: i64,
    pub processing: i64,
    pub completed: i64,
    pub failed: i64,
}
