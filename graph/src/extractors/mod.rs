pub mod github;
pub mod slack;
pub mod notion;
pub mod document;

use crate::models::{Entity, Relationship};
use crate::errors::GraphResult;
use async_trait::async_trait;

#[async_trait]
pub trait EntityExtractor: Send + Sync {
    async fn extract_entities(&self, raw_data: serde_json::Value) -> GraphResult<Vec<Entity>>;
    async fn extract_relationships(&self, entities: &[Entity]) -> GraphResult<Vec<Relationship>>;
}
