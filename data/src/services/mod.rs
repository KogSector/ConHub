pub mod data;
pub mod integrations;
pub mod embedding_client;
pub mod ingestion;
pub mod chunker_client;
pub mod graph_rag_ingestion;

// Legacy modules (will be deprecated/removed in Graph RAG migration)
#[deprecated(note = "Use chunker service instead")]
pub mod embedding_pipeline;
#[deprecated(note = "Embedding service owns vector DB operations")]
pub mod qdrant_client;
#[deprecated(note = "Embedding service owns vector DB operations")]
pub mod vector_store;
#[deprecated(note = "Graph service owns entity/relationship extraction")]
pub mod relationship;

pub use embedding_client::*;
pub use ingestion::*;
pub use chunker_client::ChunkerClient;
pub use graph_rag_ingestion::GraphRagIngestionService;

// Legacy exports (deprecated)
#[allow(deprecated)]
pub use embedding_pipeline::*;
#[allow(deprecated)]
pub use qdrant_client::QdrantClient;
#[allow(deprecated)]
pub use vector_store::{VectorStoreService, CodeSearchResult, create_vector_store_service};
#[allow(deprecated)]
pub use relationship::RelationshipService;
