//! Models for the decision engine

pub mod query;

// Re-export query types selectively (not ContextBlock which conflicts with legacy)
pub use query::{
    QueryKind, ModalityHint, RetrievalStrategy, RetrievalPlan, GraphQuerySpec, RerankStrategy,
    MemoryQuery, RobotMemoryQuery, TimeRange as QueryTimeRange,
    ContextBlock as NewContextBlock, ContextMetadata,
    MemorySearchResponse, DebugInfo, QueryAnalysisResult, ExtractedEntity,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::services::{VectorRagClient, GraphRagClient, QueryCache};

/// Application state
pub struct AppState {
    pub vector_client: VectorRagClient,
    pub graph_client: GraphRagClient,
    pub cache: RwLock<QueryCache>,
}

/// Retrieval strategy (legacy, kept for backward compatibility)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Strategy {
    Vector,
    Graph,
    Hybrid,
    Auto,
}

impl Default for Strategy {
    fn default() -> Self {
        Strategy::Auto
    }
}

/// Context query request (legacy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextQueryRequest {
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub query: String,
    
    #[serde(default)]
    pub filters: QueryFilters,
    
    #[serde(default)]
    pub strategy: Strategy,
    
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

fn default_top_k() -> usize {
    20
}

/// Time range for filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub from: chrono::DateTime<chrono::Utc>,
    pub to: chrono::DateTime<chrono::Utc>,
}

/// Query filters (legacy)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryFilters {
    #[serde(default)]
    pub source_types: Vec<String>,
    
    #[serde(default)]
    pub repos: Vec<String>,
    
    #[serde(default)]
    pub channels: Vec<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_range: Option<TimeRange>,
    
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Context query response (legacy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextQueryResponse {
    pub strategy_used: Strategy,
    pub blocks: Vec<ContextBlock>,
    pub total_results: usize,
    pub processing_time_ms: u64,
}

/// A single context block (used by vector_client and graph_client)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBlock {
    pub chunk_id: Uuid,
    pub document_id: Uuid,
    pub source: SourceInfo,
    pub content: String,
    pub score: f32,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub source_type: String,
    pub external_id: String,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector: Option<VectorProvenance>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph: Option<GraphProvenance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorProvenance {
    pub similarity: f32,
    pub embedding_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphProvenance {
    pub path: Vec<String>,
    pub distance: usize,
    pub relationship_types: Vec<String>,
}

/// Statistics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResponse {
    pub cache_enabled: bool,
    pub cache_hit_rate: f32,
    pub total_queries: usize,
    pub avg_latency_ms: f32,
    pub strategy_distribution: HashMap<String, usize>,
}
