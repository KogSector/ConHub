use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use validator::Validate;

// Re-export SourceVersionKind for public use
pub use crate::execution::row_indexer::SourceVersionKind;

// Missing types needed for compilation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkRecord {
    pub content: String,
    pub embedding: Vec<f32>,
    pub metadata: HashMap<String, String>,
}

impl ChunkRecord {
    pub fn new(content: String, embedding: Vec<f32>, metadata: HashMap<String, String>) -> Self {
        Self {
            content,
            embedding,
            metadata,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentFingerprint {
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationSet {
    pub operation: String,
    pub row_key: String,
    pub data: Option<Vec<ChunkRecord>>,
}

impl MutationSet {
    pub fn deletion(row_key: String) -> Self {
        Self {
            operation: "delete".to_string(),
            row_key,
            data: None,
        }
    }

    pub fn upsert(row_key: String, chunks: Vec<ChunkRecord>) -> Self {
        Self {
            operation: "upsert".to_string(),
            row_key,
            data: Some(chunks),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowSnapshot {
    pub row_id: String,
    pub fingerprint: Option<ContentFingerprint>,
    pub version_kind: SourceVersionKind,
    pub last_mutation_at: DateTime<Utc>,
    pub mutation: Option<MutationSet>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct IndexRepositoryRequest {
    #[validate(url)]
    pub repository_url: String,
    pub branch: Option<String>,
    pub include_patterns: Option<Vec<String>>,
    pub exclude_patterns: Option<Vec<String>>,
    pub language_filters: Option<Vec<String>>,
    pub max_file_size: Option<usize>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct IndexDocumentationRequest {
    #[validate(url)]
    pub documentation_url: String,
    pub doc_type: Option<String>,
    pub crawl_depth: Option<u32>,
    pub follow_links: Option<bool>,
    pub extract_code_blocks: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct IndexUrlRequest {
    #[validate(url)]
    pub url: String,
    pub content_type: Option<String>,
    pub extract_links: Option<bool>,
    pub max_depth: Option<u32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct IndexFileRequest {
    pub file_path: String,
    pub file_type: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub source_type: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub filters: Option<HashMap<String, String>>,
}





#[derive(Debug, Serialize)]
pub struct IndexingResponse {
    pub success: bool,
    pub job_id: String,
    pub message: String,
    pub status: IndexingStatus,
}

#[derive(Debug, Serialize)]
pub struct IndexingResultResponse {
    pub job_id: String,
    pub status: IndexingStatus,
    pub documents_processed: usize,
    pub chunks_created: usize,
    pub embeddings_generated: usize,
    pub processing_time_ms: u64,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total_count: usize,
    pub query: String,
    pub processing_time_ms: u64,
}

#[derive(Debug, Serialize, Clone)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub content: String,
    pub source_type: String,
    pub source_url: String,
    pub score: f32,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub active_jobs: usize,
    pub completed_jobs: usize,
    pub failed_jobs: usize,
    pub queue_size: usize,
}





#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum IndexingStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for IndexingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexingStatus::Pending => write!(f, "pending"),
            IndexingStatus::InProgress => write!(f, "in_progress"),
            IndexingStatus::Completed => write!(f, "completed"),
            IndexingStatus::Failed => write!(f, "failed"),
            IndexingStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SourceType {
    Code,
    Repository,
    Documentation,
    Url,
    File,
}

impl std::fmt::Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceType::Code => write!(f, "code"),
            SourceType::Repository => write!(f, "repository"),
            SourceType::Documentation => write!(f, "documentation"),
            SourceType::Url => write!(f, "url"),
            SourceType::File => write!(f, "file"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndexingJob {
    pub id: String,
    pub source_type: SourceType,
    pub source_url: String,
    pub status: IndexingStatus,
    pub metadata: HashMap<String, String>,
    pub documents_processed: usize,
    pub chunks_created: usize,
    pub embeddings_generated: usize,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

impl IndexingJob {
    pub fn new(source_type: SourceType, source_url: String, metadata: HashMap<String, String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            source_type,
            source_url,
            status: IndexingStatus::Pending,
            metadata,
            documents_processed: 0,
            chunks_created: 0,
            embeddings_generated: 0,
            started_at: Utc::now(),
            completed_at: None,
            error_message: None,
        }
    }
    
    pub fn start(&mut self) {
        self.status = IndexingStatus::InProgress;
        self.started_at = Utc::now();
    }
    
    pub fn complete(&mut self, docs: usize, chunks: usize, embeddings: usize) {
        self.status = IndexingStatus::Completed;
        self.documents_processed = docs;
        self.chunks_created = chunks;
        self.embeddings_generated = embeddings;
        self.completed_at = Some(Utc::now());
    }
    
    pub fn fail(&mut self, error: String) {
        self.status = IndexingStatus::Failed;
        self.error_message = Some(error);
        self.completed_at = Some(Utc::now());
    }
    
    pub fn processing_time_ms(&self) -> u64 {
        if let Some(completed) = self.completed_at {
            (completed - self.started_at).num_milliseconds() as u64
        } else {
            (Utc::now() - self.started_at).num_milliseconds() as u64
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
    pub source_type: String,
    pub source_url: String,
    pub language: Option<String>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub id: String,
    pub document_id: String,
    pub content: String,
    pub index: usize,
    pub metadata: HashMap<String, String>,
}
