use sqlx::PgPool;
use uuid::Uuid;
use tracing::{info, warn};
use std::collections::HashMap;

use conhub_models::chunking::ChunkText;

/// Repository for fetching chunk text from Postgres (single source of truth).
/// Used by decision_engine to hydrate chunk IDs with actual content.
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

        info!("📖 [Decision Engine] Fetching {} chunks from Postgres", chunk_ids.len());

        let rows: Vec<ChunkRow> = sqlx::query_as(
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
        )
        .bind(chunk_ids)
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

    /// Fetch chunks with their evidence relationships for graph-first queries
    pub async fn fetch_by_entity_ids(&self, entity_ids: &[Uuid]) -> Result<Vec<ChunkText>, sqlx::Error> {
        if entity_ids.is_empty() {
            return Ok(Vec::new());
        }

        info!("📖 [Decision Engine] Fetching chunks for {} entities via evidence", entity_ids.len());

        let rows: Vec<ChunkRow> = sqlx::query_as(
            r#"
            SELECT DISTINCT
                c.chunk_id,
                c.content,
                c.block_type,
                c.language,
                c.metadata
            FROM chunks c
            JOIN entity_evidence ee ON c.chunk_id = ee.chunk_id
            WHERE ee.entity_id = ANY($1)
            ORDER BY c.chunk_id
            "#,
        )
        .bind(entity_ids)
        .fetch_all(&self.db_pool)
        .await?;

        let result: Vec<ChunkText> = rows.into_iter().map(|row| ChunkText {
            chunk_id: row.chunk_id,
            content: row.content,
            block_type: row.block_type,
            language: row.language,
            metadata: row.metadata.unwrap_or_else(|| serde_json::json!({})),
        }).collect();

        info!("✅ Fetched {} chunks for {} entities", result.len(), entity_ids.len());
        Ok(result)
    }

    /// Fetch chunks linked to relationships via evidence
    pub async fn fetch_by_relationship_ids(&self, rel_ids: &[Uuid]) -> Result<Vec<ChunkText>, sqlx::Error> {
        if rel_ids.is_empty() {
            return Ok(Vec::new());
        }

        info!("📖 [Decision Engine] Fetching chunks for {} relationships via evidence", rel_ids.len());

        let rows: Vec<ChunkRow> = sqlx::query_as(
            r#"
            SELECT DISTINCT
                c.chunk_id,
                c.content,
                c.block_type,
                c.language,
                c.metadata
            FROM chunks c
            JOIN relationship_evidence re ON c.chunk_id = re.chunk_id
            WHERE re.relationship_id = ANY($1)
            ORDER BY c.chunk_id
            "#,
        )
        .bind(rel_ids)
        .fetch_all(&self.db_pool)
        .await?;

        let result: Vec<ChunkText> = rows.into_iter().map(|row| ChunkText {
            chunk_id: row.chunk_id,
            content: row.content,
            block_type: row.block_type,
            language: row.language,
            metadata: row.metadata.unwrap_or_else(|| serde_json::json!({})),
        }).collect();

        info!("✅ Fetched {} chunks for {} relationships", result.len(), rel_ids.len());
        Ok(result)
    }
}

/// Internal row type for sqlx mapping
#[derive(sqlx::FromRow)]
struct ChunkRow {
    chunk_id: Uuid,
    content: String,
    block_type: Option<String>,
    language: Option<String>,
    metadata: Option<serde_json::Value>,
}
