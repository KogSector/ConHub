pub mod neo4j;

pub use neo4j::Neo4jGraphDb;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Graph database trait for knowledge graph operations
#[async_trait]
pub trait GraphDb: Send + Sync {
    /// Initialize database with indexes and constraints
    async fn initialize(&self) -> Result<()>;
    
    /// Insert entity into graph
    async fn insert_entity(&self, entity: &GraphEntity) -> Result<()>;
    
    /// Update existing entity
    async fn update_entity(&self, entity: &GraphEntity) -> Result<()>;
    
    /// Find entity by ID
    async fn find_entity(&self, id: Uuid) -> Result<Option<GraphEntity>>;
    
    /// Insert relationship into graph
    async fn insert_relationship(&self, relationship: &GraphRelationship) -> Result<()>;
    
    /// Insert canonical entity
    async fn insert_canonical_entity(&self, canonical: &CanonicalEntity) -> Result<()>;
    
    /// Batch insert entities
    async fn batch_insert_entities(&self, entities: &[GraphEntity]) -> Result<usize>;
    
    /// Batch insert relationships
    async fn batch_insert_relationships(&self, relationships: &[GraphRelationship]) -> Result<usize>;
    
    /// Find paths between entities
    async fn find_paths(&self, from_id: Uuid, to_id: Uuid, max_hops: usize) -> Result<Vec<EntityPath>>;
    
    /// Get graph statistics
    async fn get_statistics(&self) -> Result<GraphStatistics>;
}

/// Simplified entity for graph storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEntity {
    pub id: Uuid,
    pub entity_type: String,
    pub source: String,
    pub source_id: String,
    pub name: String,
    pub content: Option<String>,
    pub properties: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Simplified relationship for graph storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphRelationship {
    pub id: Uuid,
    pub relationship_type: String,
    pub from_entity_id: Uuid,
    pub to_entity_id: Uuid,
    pub source: String,
    pub confidence_score: f32,
    pub properties: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Canonical entity (resolved across sources)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalEntity {
    pub id: Uuid,
    pub entity_type: String,
    pub canonical_name: String,
    pub properties: serde_json::Value,
    pub confidence_score: f32,
    pub source_entity_count: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Path between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityPath {
    pub entities: Vec<GraphEntity>,
    pub relationships: Vec<GraphRelationship>,
    pub total_hops: usize,
}

/// Graph statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStatistics {
    pub total_entities: usize,
    pub total_relationships: usize,
    pub entities_by_type: std::collections::HashMap<String, usize>,
    pub entities_by_source: std::collections::HashMap<String, usize>,
}
