use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Document from connector to be embedded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentForEmbedding {
    pub id: Uuid,
    pub source_id: Uuid,
    pub connector_type: String,
    pub external_id: String,
    pub name: String,
    pub path: Option<String>,
    pub content: String,
    pub content_type: String,
    pub metadata: serde_json::Value,
    pub chunks: Option<Vec<DocumentChunk>>,
}

/// A chunk of a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub chunk_number: usize,
    pub content: String,
    pub start_offset: usize,
    pub end_offset: usize,
    pub metadata: Option<serde_json::Value>,
}

/// Request to process documents in batch
#[derive(Debug, Deserialize)]
pub struct BatchEmbedRequest {
    pub documents: Vec<DocumentForEmbedding>,
    #[serde(default)]
    pub normalize: bool,
    #[serde(default)]
    pub store_in_vector_db: bool,
}

/// Result of embedding a single document
#[derive(Debug, Serialize)]
pub struct DocumentEmbedResult {
    pub id: Uuid,
    pub name: String,
    pub status: EmbedStatus,
    pub chunks_processed: usize,
    pub error: Option<String>,
}

/// Status of embedding operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmbedStatus {
    Success,
    Failed,
    PartialSuccess,
}

/// Response for batch embedding
#[derive(Debug, Serialize)]
pub struct BatchEmbedResponse {
    pub total_documents: usize,
    pub successful: usize,
    pub failed: usize,
    pub results: Vec<DocumentEmbedResult>,
    pub duration_ms: u64,
}

/// Embedded chunk ready for storage
#[derive(Debug, Clone, Serialize)]
pub struct EmbeddedChunk {
    pub document_id: Uuid,
    pub chunk_number: usize,
    pub content: String,
    pub embedding: Vec<f32>,
    pub metadata: serde_json::Value,
}
