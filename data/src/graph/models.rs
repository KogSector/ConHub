// Simplified graph models for data service
// Full models from knowledge-graph are preserved but streamlined for integration

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

/// Universal entity types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EntityType {
    Person,
    CodeEntity,
    Document,
    Conversation,
    Project,
    Concept,
    Message,
    Commit,
    File,
    Repository,
}

/// Source of the entity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EntitySource {
    GitHub,
    Slack,
    Notion,
    GoogleDrive,
    Dropbox,
    LocalFile,
    URL,
}

/// Base entity structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: Uuid,
    pub entity_type: EntityType,
    pub source: EntitySource,
    pub source_id: String,
    pub name: String,
    pub content: Option<String>,
    pub properties: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Relationship between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: Uuid,
    pub relationship_type: String,
    pub from_entity: Uuid,
    pub to_entity: Uuid,
    pub source: EntitySource,
    pub confidence_score: f32,
    pub created_at: DateTime<Utc>,
}

/// Canonical entity (resolved across sources)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalEntity {
    pub id: Uuid,
    pub entity_type: EntityType,
    pub canonical_name: String,
    pub source_entities: Vec<Uuid>,
    pub confidence_score: f32,
}

/// Cross-source query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossSourceQuery {
    pub topic: String,
    pub sources: Vec<EntitySource>,
    pub entity_types: Vec<EntityType>,
}

/// Graph statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStatistics {
    pub total_entities: usize,
    pub total_relationships: usize,
    pub entities_by_type: HashMap<String, usize>,
    pub entities_by_source: HashMap<String, usize>,
}

// Conversion helpers
impl From<Entity> for conhub_database::graph::GraphEntity {
    fn from(entity: Entity) -> Self {
        Self {
            id: entity.id,
            entity_type: format!("{:?}", entity.entity_type),
            source: format!("{:?}", entity.source),
            source_id: entity.source_id,
            name: entity.name,
            content: entity.content,
            properties: serde_json::to_value(&entity.properties).unwrap_or_default(),
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}

impl From<Relationship> for conhub_database::graph::GraphRelationship {
    fn from(rel: Relationship) -> Self {
        Self {
            id: rel.id,
            relationship_type: rel.relationship_type,
            from_entity_id: rel.from_entity,
            to_entity_id: rel.to_entity,
            source: format!("{:?}", rel.source),
            confidence_score: rel.confidence_score,
            properties: serde_json::Value::Object(serde_json::Map::new()),
            created_at: rel.created_at,
        }
    }
}
