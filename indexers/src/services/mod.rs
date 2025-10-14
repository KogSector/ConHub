pub mod code;
pub mod document;
pub mod web;
pub mod embedding;
pub mod chunking;

use crate::models::StatusResponse;

pub trait IndexingService: Send + Sync {
    async fn get_stats(&self) -> StatusResponse;
}
