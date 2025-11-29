//! ConHub Indexers
//! 
//! This crate provides background indexing services that consume data from
//! various sources (Kafka, HTTP, etc.) and index them into ConHub's knowledge layer.
//! 
//! ## Modules
//! 
//! - `robot_memory`: Indexes robot episodes and semantic events from Kafka
//! - `relation_builder`: Extracts relations and builds knowledge graph from episodes

pub mod robot_memory;
pub mod relation_builder;

pub use robot_memory::{
    RobotMemoryIndexer,
    RobotMemoryIndexerConfig,
    EpisodeMessage,
    SemanticEventMessage,
    RobotMemoryDocument,
    IndexerError,
};

pub use relation_builder::{
    RelationBuilder,
    Relation,
    RelationType,
    SemanticFact,
    GraphNode,
    GraphEdge,
    GraphBatch,
};

/// Health check for the indexer service
pub fn health_check() -> bool {
    true
}

/// Get the version of the indexer
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
