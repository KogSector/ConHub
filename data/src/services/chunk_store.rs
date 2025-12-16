use sqlx::PgPool;
use uuid::Uuid;
use tracing::{info, warn, error};
use anyhow::{Result, Context};

use conhub_models::chunking::{Chunk, ChunkRecord, SourceKind};

/// Service for persisting chunks to the Postgres chunks table.
/// This is the single source of truth for chunk text content.
pub struct ChunkStore {
    db_pool: PgPool,
}

impl ChunkStore {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Persist a batch of chunks to the chunks table.
    /// Returns the number of chunks successfully persisted.
    pub async fn persist_chunks(
        &self,
        chunks: &[Chunk],
        tenant_id: Uuid,
        source_id: Uuid,
        source_kind: &SourceKind,
    ) -> Result<usize> {
        if chunks.is_empty() {
            return Ok(0);
        }

        info!("💾 Persisting {} chunks to Postgres", chunks.len());

        let mut persisted = 0;

        for chunk in chunks {
            let record = ChunkRecord::from_chunk(chunk, tenant_id, source_id, source_kind);

            match self.upsert_chunk(&record).await {
                Ok(_) => persisted += 1,
                Err(e) => {
                    warn!("⚠️ Failed to persist chunk {}: {}", chunk.chunk_id, e);
                }
            }
        }

        info!("✅ Persisted {}/{} chunks to Postgres", persisted, chunks.len());
        Ok(persisted)
    }

    /// Upsert a single chunk record
    async fn upsert_chunk(&self, record: &ChunkRecord) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO chunks (
                chunk_id, tenant_id, source_item_id, source_id, chunk_index,
                content, content_hash, source_kind, block_type, language, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (chunk_id) DO UPDATE SET
                content = EXCLUDED.content,
                content_hash = EXCLUDED.content_hash,
                block_type = EXCLUDED.block_type,
                language = EXCLUDED.language,
                metadata = EXCLUDED.metadata,
                updated_at = CURRENT_TIMESTAMP
            "#,
            record.chunk_id,
            record.tenant_id,
            record.source_item_id,
            record.source_id,
            record.chunk_index,
            record.content,
            record.content_hash,
            record.source_kind,
            record.block_type,
            record.language,
            record.metadata
        )
        .execute(&self.db_pool)
        .await
        .context("Failed to upsert chunk")?;

        Ok(())
    }

    /// Check if a chunk exists and has the same content hash (for change detection)
    pub async fn chunk_changed(&self, chunk_id: Uuid, new_hash: &str) -> Result<bool> {
        let existing = sqlx::query_scalar!(
            r#"SELECT content_hash FROM chunks WHERE chunk_id = $1"#,
            chunk_id
        )
        .fetch_optional(&self.db_pool)
        .await
        .context("Failed to check chunk hash")?;

        match existing {
            Some(hash) => Ok(hash != new_hash),
            None => Ok(true), // Chunk doesn't exist, so it's "changed" (new)
        }
    }

    /// Delete chunks by source_item_id (for re-indexing)
    pub async fn delete_by_source_item(&self, source_item_id: Uuid) -> Result<usize> {
        let result = sqlx::query!(
            r#"DELETE FROM chunks WHERE source_item_id = $1"#,
            source_item_id
        )
        .execute(&self.db_pool)
        .await
        .context("Failed to delete chunks")?;

        Ok(result.rows_affected() as usize)
    }

    /// Delete chunks by source_id (for full re-sync)
    pub async fn delete_by_source(&self, source_id: Uuid) -> Result<usize> {
        let result = sqlx::query!(
            r#"DELETE FROM chunks WHERE source_id = $1"#,
            source_id
        )
        .execute(&self.db_pool)
        .await
        .context("Failed to delete chunks")?;

        Ok(result.rows_affected() as usize)
    }
}
