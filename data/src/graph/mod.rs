// Graph module for knowledge graph functionality integrated into data service
// This replaces the standalone knowledge-graph microservice

pub mod models;

pub use models::*;

// Re-export database graph types
pub use conhub_database::graph::{GraphDb, Neo4jGraphDb, GraphEntity, GraphRelationship};
