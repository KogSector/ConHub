pub mod data;
pub mod integrations;
pub mod embedding_client;
pub mod ingestion;
pub mod chunker_client;
pub mod graph_rag_ingestion;
pub mod connector_cache;
pub mod kafka_client;
pub mod auth_client;

// Zilliz Cloud vector store client
pub mod zilliz_client;
pub mod vector_store;

// Legacy modules (will be deprecated/removed in Graph RAG migration)
#[deprecated(note = "Use chunker service instead")]
pub mod embedding_pipeline;
#[deprecated(note = "Graph service owns entity/relationship extraction")]
pub mod relationship;

pub use embedding_client::*;
pub use ingestion::*;
pub use chunker_client::ChunkerClient;
pub use graph_rag_ingestion::GraphRagIngestionService;
pub use zilliz_client::{ZillizClient, ZillizConfig, ZillizSearchResult};
pub use vector_store::{VectorStoreService, CodeSearchResult, create_vector_store_service};
pub use kafka_client::KafkaClient;
pub use auth_client::{AuthClient, AuthClientError, OAuthTokenResponse, OAuthStatusResponse};

// Legacy exports (deprecated)
#[allow(deprecated)]
pub use embedding_pipeline::*;
#[allow(deprecated)]
pub use relationship::RelationshipService;
