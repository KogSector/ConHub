use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Type of data source for chunking strategy selection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    CodeRepo,
    Document,
    Chat,
    Wiki,
    Ticketing,
    Email,
    Web,
    Other,
}

impl SourceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceKind::CodeRepo => "code_repo",
            SourceKind::Document => "document",
            SourceKind::Chat => "chat",
            SourceKind::Wiki => "wiki",
            SourceKind::Ticketing => "ticketing",
            SourceKind::Email => "email",
            SourceKind::Web => "web",
            SourceKind::Other => "other",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "code_repo" => Some(SourceKind::CodeRepo),
            "document" => Some(SourceKind::Document),
            "chat" => Some(SourceKind::Chat),
            "wiki" => Some(SourceKind::Wiki),
            "ticketing" => Some(SourceKind::Ticketing),
            "email" => Some(SourceKind::Email),
            "web" => Some(SourceKind::Web),
            "other" => Some(SourceKind::Other),
            _ => None,
        }
    }
}

/// A single item from a data source (file, doc, thread, etc.) to be chunked
/// This is what flows from data → chunker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceItem {
    /// Stable ID per file/doc/thread (derived from path + commit or hash)
    pub id: Uuid,
    
    /// Connected account / integration ID
    pub source_id: Uuid,
    
    /// Type of source for chunking strategy
    pub source_kind: SourceKind,
    
    /// Content type (mime-ish): "text/markdown", "text/code:rust", etc.
    pub content_type: String,
    
    /// Full text content of this item
    pub content: String,
    
    /// Metadata: repo, path, branch, commit, author, timestamps, etc.
    pub metadata: serde_json::Value,
    
    /// When this item was created/fetched
    pub created_at: Option<DateTime<Utc>>,
}

/// A chunk produced by the chunker service
/// This is what flows from chunker → embedding and chunker → graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// Stable chunk ID (e.g., hash of source_item_id + chunk_index)
    pub chunk_id: Uuid,
    
    /// Reference to the source item this came from
    pub source_item_id: Uuid,
    
    /// Index of this chunk within the source item
    pub chunk_index: u32,
    
    /// The actual text content of this chunk
    pub content: String,
    
    /// Start offset in the original content (bytes or chars)
    pub start_offset: Option<u32>,
    
    /// End offset in the original content
    pub end_offset: Option<u32>,
    
    /// Type of content block: "code", "text", "table", "comment", etc.
    pub block_type: Option<String>,
    
    /// Programming language for code chunks
    pub language: Option<String>,
    
    /// Metadata carried through from source item + chunk-specific info
    pub metadata: serde_json::Value,
}

// ============================================================================
// API Request/Response Types
// ============================================================================

/// Request to start a chunking job
#[derive(Debug, Serialize, Deserialize)]
pub struct StartChunkJobRequest {
    pub source_id: Uuid,
    pub source_kind: SourceKind,
    pub items: Vec<SourceItem>,
}

/// Response when starting a chunking job
#[derive(Debug, Serialize, Deserialize)]
pub struct StartChunkJobResponse {
    pub job_id: Uuid,
    pub accepted: usize,
}

/// Status of a chunking job
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChunkJobStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Response for job status queries
#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkJobStatusResponse {
    pub job_id: Uuid,
    pub status: ChunkJobStatus,
    pub items_total: usize,
    pub items_processed: usize,
    pub chunks_emitted: usize,
    pub error_message: Option<String>,
}

// ============================================================================
// Embedding Service Types
// ============================================================================

/// Single chunk to embed
#[derive(Debug, Serialize, Deserialize)]
pub struct EmbedChunk {
    pub chunk_id: Uuid,
    pub content: String,
    pub metadata: serde_json::Value,
}

/// Request to embed a batch of chunks
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchEmbedChunksRequest {
    pub chunks: Vec<EmbedChunk>,
    pub normalize: bool,
    pub store_in_vector_db: bool,
}

/// Response from batch embedding
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchEmbedChunksResponse {
    pub total_chunks: usize,
    pub successful: usize,
    pub failed: usize,
    pub duration_ms: Option<u64>,
}

// ============================================================================
// Graph Service Types
// ============================================================================

/// Request to ingest chunks into the knowledge graph
#[derive(Debug, Serialize, Deserialize)]
pub struct IngestChunksRequest {
    pub source_id: Uuid,
    pub source_kind: SourceKind,
    pub chunks: Vec<Chunk>,
}

/// Response from chunk ingestion
#[derive(Debug, Serialize, Deserialize)]
pub struct IngestChunksResponse {
    pub total_chunks: usize,
    pub chunks_processed: usize,
    pub entities_created: usize,
    pub relationships_created: usize,
}

// ============================================================================
// Graph Service Types (IDs-only for Option A architecture)
// ============================================================================

/// Chunk metadata for graph observation (no content - graph fetches from Postgres)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkRef {
    /// Stable chunk ID (matches chunks.chunk_id in Postgres and vector DB point ID)
    pub chunk_id: Uuid,
    
    /// Reference to the source item this came from
    pub source_item_id: Uuid,
    
    /// Index of this chunk within the source item
    pub chunk_index: u32,
    
    /// Type of content block: "code", "text", "table", "comment", etc.
    pub block_type: Option<String>,
    
    /// Programming language for code chunks
    pub language: Option<String>,
    
    /// Metadata (provenance, filters) - no content
    pub metadata: serde_json::Value,
}

impl From<Chunk> for ChunkRef {
    fn from(chunk: Chunk) -> Self {
        Self {
            chunk_id: chunk.chunk_id,
            source_item_id: chunk.source_item_id,
            chunk_index: chunk.chunk_index,
            block_type: chunk.block_type,
            language: chunk.language,
            metadata: chunk.metadata,
        }
    }
}

/// Request to observe chunks for graph extraction (IDs only, no content)
/// Graph service will fetch chunk text from Postgres by chunk_id.
#[derive(Debug, Serialize, Deserialize)]
pub struct ObserveChunksRequest {
    pub tenant_id: Uuid,
    pub source_id: Uuid,
    pub source_kind: SourceKind,
    pub chunks: Vec<ChunkRef>,
}

/// Response from chunk observation
#[derive(Debug, Serialize, Deserialize)]
pub struct ObserveChunksResponse {
    pub total_chunks: usize,
    pub chunks_processed: usize,
    pub entities_created: usize,
    pub relationships_created: usize,
    pub evidence_created: usize,
}

// ============================================================================
// Chunk Storage Types (for Postgres chunks table)
// ============================================================================

/// Record for persisting a chunk to the Postgres chunks table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkRecord {
    pub chunk_id: Uuid,
    pub tenant_id: Uuid,
    pub source_item_id: Uuid,
    pub source_id: Uuid,
    pub chunk_index: i32,
    pub content: String,
    pub content_hash: String,
    pub source_kind: String,
    pub block_type: Option<String>,
    pub language: Option<String>,
    pub metadata: serde_json::Value,
}

impl ChunkRecord {
    /// Create a ChunkRecord from a Chunk with additional context
    pub fn from_chunk(
        chunk: &Chunk,
        tenant_id: Uuid,
        source_id: Uuid,
        source_kind: &SourceKind,
    ) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        // Simple content hash (in production, use SHA256)
        let mut hasher = DefaultHasher::new();
        chunk.content.hash(&mut hasher);
        let content_hash = format!("{:x}", hasher.finish());
        
        Self {
            chunk_id: chunk.chunk_id,
            tenant_id,
            source_item_id: chunk.source_item_id,
            source_id,
            chunk_index: chunk.chunk_index as i32,
            content: chunk.content.clone(),
            content_hash,
            source_kind: source_kind.as_str().to_string(),
            block_type: chunk.block_type.clone(),
            language: chunk.language.clone(),
            metadata: chunk.metadata.clone(),
        }
    }
}

/// Response when fetching chunk text by IDs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkText {
    pub chunk_id: Uuid,
    pub content: String,
    pub block_type: Option<String>,
    pub language: Option<String>,
    pub metadata: serde_json::Value,
}

/// Request to fetch chunk texts by IDs (batch)
#[derive(Debug, Serialize, Deserialize)]
pub struct FetchChunksRequest {
    pub chunk_ids: Vec<Uuid>,
}

/// Response with fetched chunk texts
#[derive(Debug, Serialize, Deserialize)]
pub struct FetchChunksResponse {
    pub chunks: Vec<ChunkText>,
    pub missing_ids: Vec<Uuid>,
}
