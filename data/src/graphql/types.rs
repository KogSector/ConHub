use async_graphql::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

/// Connector Type enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Enum, Eq, PartialEq)]
pub enum ConnectorTypeGql {
    /// GitHub repository connector
    Github,
    /// Google Drive connector
    GoogleDrive,
    /// Dropbox connector
    Dropbox,
    /// Notion connector
    Notion,
    /// Local file system
    LocalFile,
    /// Web URL scraper
    WebUrl,
}

/// Connection status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Enum, Eq, PartialEq)]
pub enum ConnectionStatus {
    /// Connection is active and working
    Connected,
    /// Connection is being established
    Connecting,
    /// Connection is being synced
    Syncing,
    /// Connection encountered an error
    Error,
    /// Connection is disconnected
    Disconnected,
}

/// Document processing status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Enum, Eq, PartialEq)]
pub enum DocumentStatus {
    /// Document is queued for processing
    Pending,
    /// Document is being processed
    Processing,
    /// Document is being embedded
    Embedding,
    /// Document is successfully indexed
    Indexed,
    /// Document processing failed
    Failed,
}

/// Connected Account GraphQL type
#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex)]
pub struct ConnectedAccount {
    pub id: ID,
    pub user_id: ID,
    pub connector_type: ConnectorTypeGql,
    pub account_name: String,
    pub account_identifier: String,
    pub status: ConnectionStatus,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub document_count: i32,
}

#[ComplexObject]
impl ConnectedAccount {
    /// Get documents for this connected account
    async fn documents(&self, ctx: &Context<'_>) -> Result<Vec<SourceDocument>> {
        // Implementation will fetch documents from database
        Ok(vec![])
    }
    
    /// Get sync statistics
    async fn sync_stats(&self, ctx: &Context<'_>) -> Result<SyncStats> {
        Ok(SyncStats {
            total_documents: self.document_count,
            indexed_documents: 0,
            pending_documents: 0,
            failed_documents: 0,
            last_sync: self.last_sync_at,
        })
    }
}

/// Source Document GraphQL type
#[derive(Debug, Clone, SimpleObject)]
pub struct SourceDocument {
    pub id: ID,
    pub source_id: ID,
    pub connector_type: ConnectorTypeGql,
    pub external_id: String,
    pub name: String,
    pub path: Option<String>,
    pub content_type: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<i64>,
    pub url: Option<String>,
    pub is_folder: bool,
    pub status: DocumentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub indexed_at: Option<DateTime<Utc>>,
}

/// Sync Statistics
#[derive(Debug, Clone, SimpleObject)]
pub struct SyncStats {
    pub total_documents: i32,
    pub indexed_documents: i32,
    pub pending_documents: i32,
    pub failed_documents: i32,
    pub last_sync: Option<DateTime<Utc>>,
}

/// Embedding Queue Entry
#[derive(Debug, Clone, SimpleObject)]
pub struct EmbeddingQueueEntry {
    pub id: ID,
    pub document_id: ID,
    pub status: DocumentStatus,
    pub retry_count: i32,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

/// Connection Request Input
#[derive(Debug, Clone, InputObject)]
pub struct ConnectSourceInput {
    pub connector_type: ConnectorTypeGql,
    pub account_name: String,
    pub credentials: Option<Json<serde_json::Value>>,
    pub settings: Option<Json<serde_json::Value>>,
}

/// OAuth Callback Input
#[derive(Debug, Clone, InputObject)]
pub struct OAuthCallbackInput {
    pub connector_type: ConnectorTypeGql,
    pub code: String,
    pub state: String,
}

/// Sync Request Input
#[derive(Debug, Clone, InputObject)]
pub struct SyncSourceInput {
    pub account_id: ID,
    pub force_full_sync: Option<bool>,
    pub filters: Option<Json<serde_json::Value>>,
}

/// Upload Local File Input
#[derive(Debug, Clone, InputObject)]
pub struct UploadFileInput {
    pub name: String,
    pub content_type: String,
    pub size: i64,
    /// Base64 encoded file content
    pub content: String,
    pub tags: Option<Vec<String>>,
}

/// Connect Result
#[derive(Debug, Clone, SimpleObject)]
pub struct ConnectResult {
    pub success: bool,
    pub message: String,
    pub account: Option<ConnectedAccount>,
    /// OAuth authorization URL if OAuth flow is required
    pub authorization_url: Option<String>,
}

/// Sync Result
#[derive(Debug, Clone, SimpleObject)]
pub struct SyncResult {
    pub success: bool,
    pub message: String,
    pub documents_synced: i32,
    pub documents_queued_for_embedding: i32,
}

/// Disconnect Result
#[derive(Debug, Clone, SimpleObject)]
pub struct DisconnectResult {
    pub success: bool,
    pub message: String,
}

/// Available Connector Info
#[derive(Debug, Clone, SimpleObject)]
pub struct ConnectorInfo {
    pub connector_type: ConnectorTypeGql,
    pub name: String,
    pub description: String,
    pub requires_oauth: bool,
    pub supports_webhooks: bool,
    pub supported_file_types: Vec<String>,
}

/// Subscription Event Types
#[derive(Debug, Clone, Serialize, Deserialize, Union)]
pub enum DataSourceEvent {
    ConnectionStatusChanged(ConnectionStatusEvent),
    DocumentProcessed(DocumentProcessedEvent),
    SyncProgress(SyncProgressEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct ConnectionStatusEvent {
    pub account_id: ID,
    pub status: ConnectionStatus,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct DocumentProcessedEvent {
    pub document_id: ID,
    pub status: DocumentStatus,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct SyncProgressEvent {
    pub account_id: ID,
    pub total: i32,
    pub processed: i32,
    pub percentage: f64,
}
