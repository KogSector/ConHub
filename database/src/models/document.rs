use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SourceDocument {
    pub id: Uuid,
    pub source_id: Uuid,
    pub connector_type: String,
    pub external_id: String,
    pub name: String,
    pub path: Option<String>,
    pub content_type: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<i64>,
    pub url: Option<String>,
    pub is_folder: bool,
    pub metadata: Option<serde_json::Value>,
    pub indexed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DocumentChunk {
    pub id: Uuid,
    pub document_id: Uuid,
    pub chunk_number: i32,
    pub content: String,
    pub start_offset: i32,
    pub end_offset: i32,
    pub metadata: Option<serde_json::Value>,
    pub embedding_vector: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmbeddingQueueItem {
    pub id: Uuid,
    pub document_id: Uuid,
    pub status: String,
    pub retry_count: i32,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocumentInput {
    pub source_id: Uuid,
    pub connector_type: String,
    pub external_id: String,
    pub name: String,
    pub path: Option<String>,
    pub content_type: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<i64>,
    pub url: Option<String>,
    pub is_folder: bool,
    pub metadata: Option<serde_json::Value>,
}

impl super::Model for SourceDocument {
    type Id = Uuid;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
