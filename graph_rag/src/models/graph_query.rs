use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

/// Multi-source unified query
#[derive(Debug, Deserialize)]
pub struct UnifiedQuery {
    pub query: String,
    pub entity_types: Option<Vec<String>>,
    pub sources: Option<Vec<String>>,
    pub time_range: Option<TimeRange>,
    pub semantic_search: bool,
    pub max_results: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct TimeRange {
    pub start: String,
    pub end: String,
}

/// Unified query response
#[derive(Debug, Serialize)]
pub struct UnifiedQueryResponse {
    pub entities: Vec<EntityResult>,
    pub relationships: Vec<RelationshipResult>,
    pub paths: Vec<PathResult>,
    pub total_count: usize,
}

#[derive(Debug, Serialize)]
pub struct EntityResult {
    pub id: Uuid,
    pub entity_type: String,
    pub source: String,
    pub name: String,
    pub properties: serde_json::Value,
    pub relevance_score: f32,
    pub canonical_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct RelationshipResult {
    pub from_entity: Uuid,
    pub to_entity: Uuid,
    pub relationship_type: String,
    pub properties: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct PathResult {
    pub nodes: Vec<EntityResult>,
    pub edges: Vec<RelationshipResult>,
    pub path_score: f32,
}

/// Cross-source query (e.g., "Who worked on authentication?")
#[derive(Debug, Deserialize)]
pub struct CrossSourceQuery {
    pub question: String,
    pub include_sources: Option<Vec<String>>,
    pub time_range: Option<TimeRange>,
    pub group_by_canonical: bool,
}

/// Response grouped by canonical entities
#[derive(Debug, Serialize)]
pub struct CrossSourceResponse {
    pub canonical_entities: Vec<CanonicalEntityResult>,
    pub timeline: Vec<TimelineEvent>,
}

#[derive(Debug, Serialize)]
pub struct CanonicalEntityResult {
    pub canonical_id: Uuid,
    pub canonical_name: String,
    pub entity_type: String,
    pub activities: Vec<ActivityResult>,
}

#[derive(Debug, Serialize)]
pub struct ActivityResult {
    pub source: String,
    pub entity_type: String,
    pub description: String,
    pub timestamp: String,
    pub entity_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct TimelineEvent {
    pub timestamp: String,
    pub entity_id: Uuid,
    pub event_type: String,
    pub description: String,
    pub participants: Vec<String>,
}

/// Semantic search request
#[derive(Debug, Deserialize)]
pub struct SemanticSearchRequest {
    pub query_text: String,
    pub sources: Option<Vec<String>>,
    pub entity_types: Option<Vec<String>>,
    pub top_k: Option<usize>,
    pub similarity_threshold: Option<f32>,
}

/// Semantic search response
#[derive(Debug, Serialize)]
pub struct SemanticSearchResponse {
    pub results: Vec<SemanticSearchResult>,
}

#[derive(Debug, Serialize)]
pub struct SemanticSearchResult {
    pub entity_id: Uuid,
    pub entity_type: String,
    pub source: String,
    pub name: String,
    pub similarity_score: f32,
    pub snippet: Option<String>,
    pub related_entities: Vec<Uuid>,
}

/// Graph traversal request
#[derive(Debug, Deserialize)]
pub struct GraphTraversalRequest {
    pub start_entity_id: Uuid,
    pub end_entity_id: Option<Uuid>,
    pub relationship_types: Option<Vec<String>>,
    pub max_depth: u32,
    pub algorithm: TraversalAlgorithm,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraversalAlgorithm {
    Bfs,
    Dfs,
    ShortestPath,
    AllPaths,
}

/// Graph traversal response
#[derive(Debug, Serialize)]
pub struct GraphTraversalResponse {
    pub paths: Vec<PathResult>,
    pub visited_nodes: usize,
}
