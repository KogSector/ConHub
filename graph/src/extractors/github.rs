use crate::models::{Entity, EntityType, DataSource, Relationship, RelationshipType};
use crate::errors::GraphResult;
use crate::extractors::EntityExtractor;
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

pub struct GitHubExtractor;

#[async_trait]
impl EntityExtractor for GitHubExtractor {
    async fn extract_entities(&self, raw_data: serde_json::Value) -> GraphResult<Vec<Entity>> {
        let mut entities = Vec::new();
        
        // Extract repository
        if let Some(repo_name) = raw_data.get("name").and_then(|v| v.as_str()) {
            let mut props = HashMap::new();
            props.insert("full_name".to_string(), serde_json::json!(repo_name));
            
            entities.push(Entity::new(
                EntityType::Repository,
                DataSource::GitHub,
                repo_name.to_string(),
                repo_name.to_string(),
                props,
            ));
        }
        
        Ok(entities)
    }
    
    async fn extract_relationships(&self, entities: &[Entity]) -> GraphResult<Vec<Relationship>> {
        Ok(Vec::new())
    }
}
