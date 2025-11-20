use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use conhub_models::chunking::ChunkJobStatus;

use crate::services::embedding_client::EmbeddingClient;
use crate::services::graph_client::GraphClient;

/// Application state shared across handlers
pub struct AppState {
    pub embedding_client: EmbeddingClient,
    pub graph_client: GraphClient,
    pub max_concurrent_jobs: usize,
    pub jobs: RwLock<HashMap<Uuid, JobHandle>>,
}

/// Handle for tracking an active chunking job
#[derive(Debug, Clone)]
pub struct JobHandle {
    pub job_id: Uuid,
    pub source_id: Uuid,
    pub status: ChunkJobStatus,
    pub items_total: usize,
    pub items_processed: usize,
    pub chunks_emitted: usize,
    pub error_message: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl JobHandle {
    pub fn new(job_id: Uuid, source_id: Uuid, items_total: usize) -> Self {
        Self {
            job_id,
            source_id,
            status: ChunkJobStatus::Pending,
            items_total,
            items_processed: 0,
            chunks_emitted: 0,
            error_message: None,
            created_at: chrono::Utc::now(),
        }
    }
}
