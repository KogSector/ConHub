use crate::{Plugin, PluginResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
    pub content_type: String,
    pub size: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    pub path: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Sync operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub total_documents: u64,
    pub new_documents: u64,
    pub updated_documents: u64,
    pub deleted_documents: u64,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}

/// Source capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCapabilities {
    pub can_read: bool,
    pub can_write: bool,
    pub can_delete: bool,
    pub supports_real_time: bool,
    pub supports_search: bool,
    pub supports_metadata: bool,
    pub max_file_size: Option<u64>,
    pub supported_formats: Vec<String>,
}

/// Source plugin trait
#[async_trait]
pub trait SourcePlugin: Plugin {
    /// Get source capabilities
    fn capabilities(&self) -> SourceCapabilities;
    
    /// List all documents
    async fn list_documents(&self) -> PluginResult<Vec<Document>>;
    
    /// Get a specific document by ID
    async fn get_document(&self, id: &str) -> PluginResult<Document>;
    
    /// Search documents
    async fn search_documents(&self, query: &str) -> PluginResult<Vec<Document>>;
    
    /// Sync documents from the source
    async fn sync(&self) -> PluginResult<SyncResult>;
    
    /// Get document content
    async fn get_content(&self, id: &str) -> PluginResult<Vec<u8>>;
    
    /// Upload document (if supported)
    async fn upload_document(&self, document: Document, content: Vec<u8>) -> PluginResult<String>;
    
    /// Delete document (if supported)
    async fn delete_document(&self, id: &str) -> PluginResult<()>;
    
    /// Setup real-time sync (if supported)
    async fn setup_realtime_sync(&self) -> PluginResult<()>;
}

/// Source plugin factory
pub trait SourcePluginFactory: Send + Sync {
    fn create(&self) -> Box<dyn SourcePlugin>;
    fn source_type(&self) -> &str;
}