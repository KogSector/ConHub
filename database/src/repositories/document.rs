use async_trait::async_trait;
use anyhow::{Result, Context};
use sqlx::{PgPool, query_as, query};
use uuid::Uuid;

use crate::models::{SourceDocument, DocumentChunk, EmbeddingQueueItem, CreateDocumentInput, Model, Pagination, PaginatedResult};
use super::Repository;

pub struct DocumentRepository {
    pool: PgPool,
}

impl DocumentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_document(&self, input: &CreateDocumentInput) -> Result<SourceDocument> {
        let document = query_as!(
            SourceDocument,
            r#"
            INSERT INTO source_documents (source_id, connector_type, external_id, name, path, content_type, mime_type, size, url, is_folder, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
            input.source_id,
            input.connector_type,
            input.external_id,
            input.name,
            input.path,
            input.content_type,
            input.mime_type,
            input.size,
            input.url,
            input.is_folder,
            input.metadata
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to create document")?;

        Ok(document)
    }

    pub async fn find_by_source(&self, source_id: &Uuid, pagination: &Pagination) -> Result<PaginatedResult<SourceDocument>> {
        let total: i64 = query!(
            "SELECT COUNT(*) as count FROM source_documents WHERE source_id = $1",
            source_id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to count documents")?
        .count
        .unwrap_or(0);

        let documents = query_as!(
            SourceDocument,
            "SELECT * FROM source_documents WHERE source_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            source_id,
            pagination.limit,
            pagination.offset
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to find documents by source")?;

        Ok(PaginatedResult::new(documents, total, pagination))
    }

    pub async fn find_by_external_id(&self, source_id: &Uuid, external_id: &str) -> Result<Option<SourceDocument>> {
        let document = query_as!(
            SourceDocument,
            "SELECT * FROM source_documents WHERE source_id = $1 AND external_id = $2",
            source_id,
            external_id
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find document by external ID")?;

        Ok(document)
    }

    pub async fn mark_indexed(&self, id: &Uuid) -> Result<()> {
        query!(
            "UPDATE source_documents SET indexed_at = CURRENT_TIMESTAMP WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await
        .context("Failed to mark document as indexed")?;

        Ok(())
    }

    pub async fn search_documents(&self, user_id: &Uuid, search_query: &str, pagination: &Pagination) -> Result<PaginatedResult<SourceDocument>> {
        let total: i64 = query!(
            r#"
            SELECT COUNT(*) as count FROM source_documents sd
            INNER JOIN connected_accounts ca ON ca.id = sd.source_id
            WHERE ca.user_id = $1 AND sd.name ILIKE $2
            "#,
            user_id,
            format!("%{}%", search_query)
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to count search results")?
        .count
        .unwrap_or(0);

        let documents = query_as!(
            SourceDocument,
            r#"
            SELECT sd.* FROM source_documents sd
            INNER JOIN connected_accounts ca ON ca.id = sd.source_id
            WHERE ca.user_id = $1 AND sd.name ILIKE $2
            ORDER BY sd.created_at DESC
            LIMIT $3 OFFSET $4
            "#,
            user_id,
            format!("%{}%", search_query),
            pagination.limit,
            pagination.offset
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to search documents")?;

        Ok(PaginatedResult::new(documents, total, pagination))
    }

    // Document chunks operations
    pub async fn create_chunk(&self, document_id: &Uuid, chunk_number: i32, content: &str, start_offset: i32, end_offset: i32, metadata: Option<serde_json::Value>) -> Result<DocumentChunk> {
        let chunk = query_as!(
            DocumentChunk,
            r#"
            INSERT INTO document_chunks (document_id, chunk_number, content, start_offset, end_offset, metadata)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (document_id, chunk_number) DO UPDATE
            SET content = EXCLUDED.content, start_offset = EXCLUDED.start_offset, end_offset = EXCLUDED.end_offset, metadata = EXCLUDED.metadata
            RETURNING *
            "#,
            document_id,
            chunk_number,
            content,
            start_offset,
            end_offset,
            metadata
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to create chunk")?;

        Ok(chunk)
    }

    pub async fn get_chunks(&self, document_id: &Uuid) -> Result<Vec<DocumentChunk>> {
        let chunks = query_as!(
            DocumentChunk,
            "SELECT * FROM document_chunks WHERE document_id = $1 ORDER BY chunk_number",
            document_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to get chunks")?;

        Ok(chunks)
    }

    pub async fn update_chunk_embedding(&self, chunk_id: &Uuid, embedding: &str) -> Result<()> {
        query!(
            "UPDATE document_chunks SET embedding_vector = $1 WHERE id = $2",
            embedding,
            chunk_id
        )
        .execute(&self.pool)
        .await
        .context("Failed to update chunk embedding")?;

        Ok(())
    }

    // Embedding queue operations
    pub async fn add_to_embedding_queue(&self, document_id: &Uuid) -> Result<()> {
        query!(
            r#"
            INSERT INTO embedding_queue (document_id, status)
            VALUES ($1, 'pending')
            ON CONFLICT (document_id) DO NOTHING
            "#,
            document_id
        )
        .execute(&self.pool)
        .await
        .context("Failed to add to embedding queue")?;

        Ok(())
    }

    pub async fn get_pending_embeddings(&self, limit: i32) -> Result<Vec<EmbeddingQueueItem>> {
        let items = query_as!(
            EmbeddingQueueItem,
            "SELECT * FROM embedding_queue WHERE status = 'pending' AND retry_count < 3 ORDER BY created_at LIMIT $1",
            limit as i64
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to get pending embeddings")?;

        Ok(items)
    }

    pub async fn update_embedding_status(&self, id: &Uuid, status: &str, error: Option<String>) -> Result<()> {
        query!(
            r#"
            UPDATE embedding_queue 
            SET status = $1, error_message = $2, processed_at = CURRENT_TIMESTAMP
            WHERE id = $3
            "#,
            status,
            error,
            id
        )
        .execute(&self.pool)
        .await
        .context("Failed to update embedding status")?;

        Ok(())
    }
}

#[async_trait]
impl Repository<SourceDocument> for DocumentRepository {
    async fn create(&self, entity: &SourceDocument) -> Result<SourceDocument> {
        let document = query_as!(
            SourceDocument,
            r#"
            INSERT INTO source_documents (id, source_id, connector_type, external_id, name, path, content_type, mime_type, size, url, is_folder, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
            entity.id,
            entity.source_id,
            entity.connector_type,
            entity.external_id,
            entity.name,
            entity.path,
            entity.content_type,
            entity.mime_type,
            entity.size,
            entity.url,
            entity.is_folder,
            entity.metadata
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to create document")?;

        Ok(document)
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<SourceDocument>> {
        let document = query_as!(
            SourceDocument,
            "SELECT * FROM source_documents WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find document")?;

        Ok(document)
    }

    async fn update(&self, id: &Uuid, entity: &SourceDocument) -> Result<SourceDocument> {
        let document = query_as!(
            SourceDocument,
            r#"
            UPDATE source_documents
            SET name = $1, path = $2, content_type = $3, mime_type = $4, size = $5, url = $6, metadata = $7, updated_at = CURRENT_TIMESTAMP
            WHERE id = $8
            RETURNING *
            "#,
            entity.name,
            entity.path,
            entity.content_type,
            entity.mime_type,
            entity.size,
            entity.url,
            entity.metadata,
            id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to update document")?;

        Ok(document)
    }

    async fn delete(&self, id: &Uuid) -> Result<bool> {
        let result = query!(
            "DELETE FROM source_documents WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await
        .context("Failed to delete document")?;

        Ok(result.rows_affected() > 0)
    }

    async fn list(&self, pagination: &Pagination) -> Result<PaginatedResult<SourceDocument>> {
        let total: i64 = query!("SELECT COUNT(*) as count FROM source_documents")
            .fetch_one(&self.pool)
            .await
            .context("Failed to count documents")?
            .count
            .unwrap_or(0);

        let documents = query_as!(
            SourceDocument,
            "SELECT * FROM source_documents ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            pagination.limit,
            pagination.offset
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to list documents")?;

        Ok(PaginatedResult::new(documents, total, pagination))
    }
}
