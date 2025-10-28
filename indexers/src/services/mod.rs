pub mod code;
pub mod document;
pub mod web;
pub mod embedding;
pub mod chunking;
pub mod state;
pub mod evaluation;
pub mod qdrant;
pub mod fusion;

use crate::models::StatusResponse;

pub trait IndexingService: Send + Sync {
    async fn get_stats(&self) -> StatusResponse;
}
