use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

/// Represents the type of connector
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorType {
    LocalFile,
    GitHub,
    GitLab,
    Bitbucket,
    GoogleDrive,
    Dropbox,
    OneDrive,
    Notion,
    Slack,
    UrlScraper,
}

impl ConnectorType {
    pub fn as_str(&self) -> &str {
        match self {
            ConnectorType::LocalFile => "local_file",
            ConnectorType::GitHub => "github",
            ConnectorType::GitLab => "gitlab",
            ConnectorType::Bitbucket => "bitbucket",
            ConnectorType::GoogleDrive => "google_drive",
            ConnectorType::Dropbox => "dropbox",
            ConnectorType::OneDrive => "onedrive",
            ConnectorType::Notion => "notion",
            ConnectorType::Slack => "slack",
            ConnectorType::UrlScraper => "url_scraper",
        }
    }
}

/// Status of a connected account
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Syncing,
    Error(String),
    PendingAuth,
}

/// Represents a connected external account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectedAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub connector_type: ConnectorType,
    pub account_name: String,
    pub account_identifier: String, // email, username, etc.
    pub credentials: serde_json::Value, // Encrypted OAuth tokens, API keys, etc.
    pub status: ConnectionStatus,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

/// Metadata about a document from an external source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub external_id: String,
    pub name: String,
    pub path: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<i64>,
    pub created_at: Option<DateTime<Utc>>,
    pub modified_at: Option<DateTime<Utc>>,
    pub permissions: Option<serde_json::Value>,
    pub url: Option<String>,
    pub parent_id: Option<String>,
    pub is_folder: bool,
    pub metadata: Option<serde_json::Value>,
}

/// Content of a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContent {
    pub metadata: DocumentMetadata,
    pub content: Vec<u8>,
    pub content_type: ContentType,
}

/// Type of content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    Text,
    Binary,
    Code,
    Markdown,
    Html,
    Pdf,
    Image,
    Video,
    Audio,
    Archive,
    Unknown,
}

impl ContentType {
    pub fn to_string(&self) -> String {
        match self {
            ContentType::Text => "text".to_string(),
            ContentType::Binary => "binary".to_string(),
            ContentType::Code => "code".to_string(),
            ContentType::Markdown => "markdown".to_string(),
            ContentType::Html => "html".to_string(),
            ContentType::Pdf => "pdf".to_string(),
            ContentType::Image => "image".to_string(),
            ContentType::Video => "video".to_string(),
            ContentType::Audio => "audio".to_string(),
            ContentType::Archive => "archive".to_string(),
            ContentType::Unknown => "unknown".to_string(),
        }
    }
}

/// Result of a sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub total_documents: usize,
    pub new_documents: usize,
    pub updated_documents: usize,
    pub deleted_documents: usize,
    pub failed_documents: usize,
    pub sync_duration_ms: u64,
    pub errors: Vec<String>,
}

/// Configuration for a connector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorConfig {
    pub user_id: Uuid,
    pub connector_type: ConnectorType,
    pub credentials: HashMap<String, String>,
    pub settings: HashMap<String, serde_json::Value>,
}

/// Request to connect a new data source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectRequest {
    pub connector_type: ConnectorType,
    pub account_name: Option<String>,
    pub credentials: HashMap<String, String>,
    pub settings: Option<HashMap<String, serde_json::Value>>,
}

/// Sync request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    pub account_id: Uuid,
    pub incremental: bool, // If true, only sync changes since last sync
    pub filters: Option<SyncFilters>,
}

/// Filters for sync operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncFilters {
    pub include_paths: Option<Vec<String>>,
    pub exclude_paths: Option<Vec<String>>,
    pub file_types: Option<Vec<String>>,
    pub max_file_size: Option<i64>,
}

/// Document to be embedded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentForEmbedding {
    pub id: Uuid,
    pub source_id: Uuid,
    pub connector_type: ConnectorType,
    pub external_id: String,
    pub name: String,
    pub path: Option<String>,
    pub content: String,
    pub content_type: ContentType,
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

/// OAuth callback data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthCallbackData {
    pub code: String,
    pub state: String,
}

/// OAuth credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthCredentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: Option<i64>,
    pub expires_at: Option<DateTime<Utc>>,
    pub scope: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Sync request with filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequestWithFilters {
    pub force_full_sync: bool,
    pub filters: Option<SyncFilters>,
}

/// Connector config for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorConfigAuth {
    pub connector_type: ConnectorType,
    pub credentials: HashMap<String, String>,
    pub settings: HashMap<String, serde_json::Value>,
}
