//! Query analysis and classification models
//! 
//! This module provides types for analyzing queries and selecting
//! the optimal retrieval strategy for AI agents.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// QUERY KIND - What type of question is being asked?
// ============================================================================

/// Classification of query intent
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum QueryKind {
    /// Simple fact lookup: "What is X?", "Where is Y?"
    FactLookup,
    
    /// Episodic/temporal query: "What happened?", "When did X occur?"
    EpisodicLookup,
    
    /// Topology/relationship query: "Who owns?", "What depends on?"
    TopologyQuestion,
    
    /// Explanation: "How does X work?", "Explain Y"
    Explainer,
    
    /// How-to/procedural: "How do I?", "Steps to?"
    HowTo,
    
    /// Troubleshooting: "Why did X fail?", "Debug Y"
    Troubleshooting,
    
    /// Task support: "What should I do next?", "Recommend action"
    TaskSupport,
    
    /// Comparison: "Difference between X and Y?"
    Comparison,
    
    /// Aggregation: "How many?", "List all?", "Summary of?"
    Aggregation,
    
    /// Generic/unclassified
    Generic,
}

impl Default for QueryKind {
    fn default() -> Self {
        Self::Generic
    }
}

// ============================================================================
// MODALITY HINT - What kind of content should we search?
// ============================================================================

/// Hint about what content modality to prioritize
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModalityHint {
    /// Source code
    Code,
    
    /// Documentation, markdown, wikis
    Docs,
    
    /// Chat messages (Slack, Discord, etc.)
    Chat,
    
    /// Tickets, issues, PRs
    Tickets,
    
    /// Robot episodic memory
    RobotEpisodic,
    
    /// Robot semantic facts
    RobotSemantic,
    
    /// Mixed/unknown
    Mixed,
}

impl Default for ModalityHint {
    fn default() -> Self {
        Self::Mixed
    }
}

// ============================================================================
// RETRIEVAL STRATEGY
// ============================================================================

/// High-level retrieval strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalStrategy {
    /// Use only vector similarity search
    VectorOnly,
    
    /// Use only graph traversal
    GraphOnly,
    
    /// Use both, then fuse results
    Hybrid,
    
    /// Vector first, then graph expansion from results
    VectorThenGraph,
    
    /// Graph first, then vector to enrich
    GraphThenVector,
}

impl Default for RetrievalStrategy {
    fn default() -> Self {
        Self::VectorOnly
    }
}

// ============================================================================
// RETRIEVAL PLAN - Detailed execution plan
// ============================================================================

/// Detailed plan for executing a retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalPlan {
    /// Primary strategy
    pub strategy: RetrievalStrategy,
    
    /// Vector collections to search
    pub vector_collections: Vec<String>,
    
    /// Graph query specifications
    pub graph_queries: Vec<GraphQuerySpec>,
    
    /// Maximum tokens in final context
    pub max_tokens: u32,
    
    /// Maximum number of context blocks
    pub max_blocks: u32,
    
    /// Whether to include provenance info
    pub include_provenance: bool,
    
    /// Re-ranking strategy
    pub rerank: RerankStrategy,
}

impl Default for RetrievalPlan {
    fn default() -> Self {
        Self {
            strategy: RetrievalStrategy::VectorOnly,
            vector_collections: vec!["default".to_string()],
            graph_queries: vec![],
            max_tokens: 8000,
            max_blocks: 20,
            include_provenance: true,
            rerank: RerankStrategy::ScoreBased,
        }
    }
}

/// Graph query specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQuerySpec {
    /// Starting node types
    pub start_node_types: Vec<String>,
    
    /// Edge types to traverse
    pub edge_types: Vec<String>,
    
    /// Maximum hops
    pub max_hops: u32,
    
    /// Filter conditions
    pub filters: HashMap<String, serde_json::Value>,
}

/// Re-ranking strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RerankStrategy {
    /// Simple score-based sorting
    ScoreBased,
    
    /// Reciprocal Rank Fusion (for hybrid)
    ReciprocalRankFusion,
    
    /// Diversity-aware (MMR-like)
    DiversityAware,
    
    /// Recency-biased
    RecencyBiased,
}

// ============================================================================
// MEMORY QUERY - Input for memory search
// ============================================================================

/// A query to the memory/knowledge system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQuery {
    /// Tenant ID (required for isolation)
    pub tenant_id: Uuid,
    
    /// User ID making the query
    pub user_id: Uuid,
    
    /// The natural language query
    pub query: String,
    
    /// Optional source filters
    #[serde(default)]
    pub sources: Vec<String>,
    
    /// Optional time range filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_range: Option<TimeRange>,
    
    /// Additional filters (repo, channel, tags, etc.)
    #[serde(default)]
    pub filters: HashMap<String, serde_json::Value>,
    
    /// Maximum context blocks to return
    #[serde(default = "default_max_blocks")]
    pub max_blocks: u32,
    
    /// Maximum tokens in response
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    
    /// Force a specific strategy (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_strategy: Option<RetrievalStrategy>,
    
    /// Include debug/provenance info
    #[serde(default)]
    pub include_debug: bool,
}

fn default_max_blocks() -> u32 {
    20
}

fn default_max_tokens() -> u32 {
    8000
}

/// Time range for filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

// ============================================================================
// ROBOT MEMORY QUERY - Specialized for robot context
// ============================================================================

/// Query specifically for robot memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotMemoryQuery {
    /// Robot ID
    pub robot_id: Uuid,
    
    /// Tenant ID
    pub tenant_id: Uuid,
    
    /// The natural language query
    pub query: String,
    
    /// Time range filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_range: Option<TimeRange>,
    
    /// Location filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    
    /// Include episodic memory
    #[serde(default = "default_true")]
    pub include_episodic: bool,
    
    /// Include semantic facts
    #[serde(default = "default_true")]
    pub include_semantic: bool,
    
    /// Maximum blocks
    #[serde(default = "default_max_blocks")]
    pub max_blocks: u32,
}

fn default_true() -> bool {
    true
}

// ============================================================================
// CONTEXT BLOCK - Output unit
// ============================================================================

/// A single block of context returned from memory search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBlock {
    /// Unique ID of this block
    pub id: Uuid,
    
    /// Source document/chunk ID
    pub source_id: String,
    
    /// The actual text content
    pub text: String,
    
    /// Source type (code, docs, episode, fact, etc.)
    pub source_type: String,
    
    /// Relevance score (0.0 - 1.0)
    pub score: f32,
    
    /// Token count (approximate)
    pub token_count: u32,
    
    /// Metadata for provenance and filtering
    pub metadata: ContextMetadata,
}

/// Metadata attached to a context block
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextMetadata {
    /// Source system (github, slack, robot, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    
    /// Repository or container
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    
    /// File path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    
    /// Timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
    
    /// Robot ID (if robot memory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub robot_id: Option<Uuid>,
    
    /// Episode number (if episodic)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode_number: Option<i64>,
    
    /// Location (if robot memory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    
    /// Additional key-value pairs
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, serde_json::Value>,
}

// ============================================================================
// MEMORY SEARCH RESPONSE
// ============================================================================

/// Response from memory search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResponse {
    /// The context blocks
    pub blocks: Vec<ContextBlock>,
    
    /// Total number of results (before truncation)
    pub total_results: u32,
    
    /// Query kind that was detected
    pub query_kind: QueryKind,
    
    /// Strategy that was used
    pub strategy_used: RetrievalStrategy,
    
    /// Processing time in milliseconds
    pub took_ms: u64,
    
    /// Debug information (if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<DebugInfo>,
}

/// Debug information for troubleshooting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugInfo {
    /// Detected modality hint
    pub modality_hint: ModalityHint,
    
    /// Collections that were searched
    pub collections_searched: Vec<String>,
    
    /// Graph queries executed
    pub graph_queries_executed: usize,
    
    /// Vector results count (before fusion)
    pub vector_results: usize,
    
    /// Graph results count (before fusion)
    pub graph_results: usize,
}

// ============================================================================
// QUERY ANALYSIS RESULT
// ============================================================================

/// Result of analyzing a query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAnalysisResult {
    /// Detected query kind
    pub kind: QueryKind,
    
    /// Detected modality hint
    pub modality: ModalityHint,
    
    /// Confidence in the classification (0.0 - 1.0)
    pub confidence: f32,
    
    /// Extracted entities (if any)
    pub entities: Vec<ExtractedEntity>,
    
    /// Suggested retrieval plan
    pub suggested_plan: RetrievalPlan,
}

/// An entity extracted from the query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    /// Entity text
    pub text: String,
    
    /// Entity type (repo, file, person, location, object, etc.)
    pub entity_type: String,
    
    /// Confidence
    pub confidence: f32,
}
