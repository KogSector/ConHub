use sqlx::PgPool;
use uuid::Uuid;
use tracing::{info, warn};
use std::collections::HashMap;

use conhub_models::chunking::ChunkText;

/// Repository for fetching chunk text from Postgres (single source of truth).
/// This implements Option A: graph_rag pulls chunk text by chunk_id.
pub struct ChunkRepository {
    db_pool: PgPool,
}

impl ChunkRepository {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Fetch chunk texts by IDs (batch query for efficiency).
    /// Returns a HashMap for O(1) lookup by chunk_id.
    pub async fn fetch_by_ids(&self, chunk_ids: &[Uuid]) -> Result<HashMap<Uuid, ChunkText>, sqlx::Error> {
        if chunk_ids.is_empty() {
            return Ok(HashMap::new());
        }

        info!("📖 Fetching {} chunks from Postgres", chunk_ids.len());

        let rows = sqlx::query_as!(
            ChunkRow,
            r#"
            SELECT 
                chunk_id,
                content,
                block_type,
                language,
                metadata
            FROM chunks
            WHERE chunk_id = ANY($1)
            "#,
            chunk_ids
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut result = HashMap::with_capacity(rows.len());
        for row in rows {
            result.insert(row.chunk_id, ChunkText {
                chunk_id: row.chunk_id,
                content: row.content,
                block_type: row.block_type,
                language: row.language,
                metadata: row.metadata.unwrap_or_else(|| serde_json::json!({})),
            });
        }

        if result.len() < chunk_ids.len() {
            let missing_count = chunk_ids.len() - result.len();
            warn!("⚠️ {} chunks not found in Postgres", missing_count);
        }

        info!("✅ Fetched {} chunks from Postgres", result.len());
        Ok(result)
    }

    /// Fetch a single chunk by ID
    pub async fn fetch_by_id(&self, chunk_id: Uuid) -> Result<Option<ChunkText>, sqlx::Error> {
        let row = sqlx::query_as!(
            ChunkRow,
            r#"
            SELECT 
                chunk_id,
                content,
                block_type,
                language,
                metadata
            FROM chunks
            WHERE chunk_id = $1
            "#,
            chunk_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(row.map(|r| ChunkText {
            chunk_id: r.chunk_id,
            content: r.content,
            block_type: r.block_type,
            language: r.language,
            metadata: r.metadata.unwrap_or_else(|| serde_json::json!({})),
        }))
    }
}

/// Internal row type for sqlx mapping
struct ChunkRow {
    chunk_id: Uuid,
    content: String,
    block_type: Option<String>,
    language: Option<String>,
    metadata: Option<serde_json::Value>,
}
