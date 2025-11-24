use crate::extractors::EntityExtractor;
use crate::models::{Entity, Relationship};
use crate::errors::GraphResult;
use async_trait::async_trait;

pub struct DocumentExtractor;

#[async_trait]
impl EntityExtractor for DocumentExtractor {
    async fn extract_entities(&self, _raw_data: serde_json::Value) -> GraphResult<Vec<Entity>> {
        Ok(Vec::new())
    }
    
    async fn extract_relationships(&self, _entities: &[Entity]) -> GraphResult<Vec<Relationship>> {
        Ok(Vec::new())
    }
}
