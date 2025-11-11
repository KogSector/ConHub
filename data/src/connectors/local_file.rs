use async_trait::async_trait;
use uuid::Uuid;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncReadExt;
use walkdir::WalkDir;
use mime_guess::from_path;
use tracing::{info, warn, error};

use super::traits::{Connector, ConnectorFactory};
use super::types::*;
use super::error::ConnectorError;

/// Local file system connector
pub struct LocalFileConnector {
    name: String,
}

impl LocalFileConnector {
    pub fn new() -> Self {
        Self {
            name: "Local File System".to_string(),
        }
    }
    
    pub fn factory() -> LocalFileConnectorFactory {
        LocalFileConnectorFactory
    }
    
    /// Process uploaded file
    async fn process_uploaded_file(
        &self,
        file_path: &Path,
        user_id: Uuid,
    ) -> Result<DocumentForEmbedding, ConnectorError> {
        let metadata = fs::metadata(file_path).await?;
        
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let mime_type = from_path(file_path)
            .first()
            .map(|m| m.to_string());
        
        let content_type = self.determine_content_type(&mime_type);
        
        // Read file content
        let mut file = fs::File::open(file_path).await?;
        let mut content = Vec::new();
        file.read_to_end(&mut content).await?;
        
        let content_str = match content_type {
            ContentType::Text | ContentType::Code | ContentType::Markdown | ContentType::Html => {
                String::from_utf8_lossy(&content).to_string()
            }
            ContentType::Pdf => {
                // TODO: Extract text from PDF
                warn!("PDF text extraction not yet implemented");
                String::from_utf8_lossy(&content).to_string()
            }
            _ => {
                // For binary files, just store metadata
                format!("Binary file: {}", file_name)
            }
        };
        
        // Chunk the content
        let chunks = self.chunk_content(&content_str)?;
        
        Ok(DocumentForEmbedding {
            id: Uuid::new_v4(),
            source_id: user_id, // Using user_id as source for local files
            connector_type: ConnectorType::LocalFile,
            external_id: file_name.clone(),
            name: file_name.clone(),
            path: Some(file_path.to_string_lossy().to_string()),
            content: content_str,
            content_type,
            metadata: serde_json::json!({
                "size": metadata.len(),
                "mime_type": mime_type,
                "modified": metadata.modified().ok(),
            }),
            chunks: Some(chunks),
        })
    }
    
    fn determine_content_type(&self, mime_type: &Option<String>) -> ContentType {
        match mime_type.as_ref().map(|s| s.as_str()) {
            Some("text/plain") => ContentType::Text,
            Some("text/markdown") => ContentType::Markdown,
            Some("text/html") => ContentType::Html,
            Some(m) if m.starts_with("text/") => ContentType::Text,
            Some(m) if m.starts_with("application/") && m.contains("pdf") => ContentType::Pdf,
            Some(m) if m.starts_with("image/") => ContentType::Image,
            Some(m) if m.starts_with("video/") => ContentType::Video,
            Some(m) if m.starts_with("audio/") => ContentType::Audio,
            _ => ContentType::Binary,
        }
    }
    
    fn chunk_content(&self, content: &str) -> Result<Vec<DocumentChunk>, ConnectorError> {
        const CHUNK_SIZE: usize = 1000; // characters
        const CHUNK_OVERLAP: usize = 200; // characters
        
        let mut chunks = Vec::new();
        let content_len = content.len();
        let mut chunk_number = 0;
        let mut start = 0;
        
        while start < content_len {
            let end = (start + CHUNK_SIZE).min(content_len);
            let chunk_content = &content[start..end];
            
            chunks.push(DocumentChunk {
                chunk_number,
                content: chunk_content.to_string(),
                start_offset: start,
                end_offset: end,
                metadata: Some(serde_json::json!({
                    "length": chunk_content.len(),
                })),
            });
            
            chunk_number += 1;
            start = end.saturating_sub(CHUNK_OVERLAP);
            
            if start + CHUNK_SIZE >= content_len && end == content_len {
                break;
            }
        }
        
        Ok(chunks)
    }
}

#[async_trait]
impl Connector for LocalFileConnector {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn connector_type(&self) -> ConnectorType {
        ConnectorType::LocalFile
    }
    
    fn validate_config(&self, _config: &ConnectorConfig) -> Result<(), ConnectorError> {
        // Local file connector doesn't need special configuration
        Ok(())
    }
    
    async fn authenticate(&self, _config: &ConnectorConfig) -> Result<Option<String>, ConnectorError> {
        // No authentication needed for local files
        Ok(None)
    }
    
    async fn complete_oauth(&self, _callback_data: OAuthCallbackData) -> Result<OAuthCredentials, ConnectorError> {
        Err(ConnectorError::UnsupportedOperation(
            "Local file connector does not support OAuth".to_string()
        ))
    }
    
    async fn connect(&mut self, _account: &ConnectedAccount) -> Result<(), ConnectorError> {
        info!("📁 Connected to local file system");
        Ok(())
    }
    
    async fn check_connection(&self, _account: &ConnectedAccount) -> Result<bool, ConnectorError> {
        // Local file system is always available
        Ok(true)
    }
    
    async fn list_documents(
        &self,
        account: &ConnectedAccount,
        filters: Option<SyncFilters>,
    ) -> Result<Vec<DocumentMetadata>, ConnectorError> {
        // For local files, we list from a configured directory
        let base_path = account.metadata
            .as_ref()
            .and_then(|m| m.get("base_path"))
            .and_then(|p| p.as_str())
            .ok_or_else(|| ConnectorError::InvalidConfiguration(
                "base_path not configured".to_string()
            ))?;
        
        let mut documents = Vec::new();
        
        for entry in WalkDir::new(base_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            
            let path = entry.path();
            
            // Apply filters
            if let Some(ref filters) = filters {
                if let Some(ref exclude_paths) = filters.exclude_paths {
                    if exclude_paths.iter().any(|p| path.to_string_lossy().contains(p)) {
                        continue;
                    }
                }
                
                if let Some(ref include_paths) = filters.include_paths {
                    if !include_paths.iter().any(|p| path.to_string_lossy().contains(p)) {
                        continue;
                    }
                }
            }
            
            let metadata = entry.metadata().map_err(|e| ConnectorError::IoError(e.to_string()))?;
            
            documents.push(DocumentMetadata {
                external_id: path.to_string_lossy().to_string(),
                name: path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                path: Some(path.to_string_lossy().to_string()),
                mime_type: from_path(path).first().map(|m| m.to_string()),
                size: Some(metadata.len() as i64),
                created_at: metadata.created().ok().map(|t| chrono::DateTime::from(t)),
                modified_at: metadata.modified().ok().map(|t| chrono::DateTime::from(t)),
                permissions: None,
                url: None,
                parent_id: path.parent()
                    .and_then(|p| p.to_str())
                    .map(|s| s.to_string()),
                is_folder: false,
                metadata: None,
            });
        }
        
        Ok(documents)
    }
    
    async fn get_document_content(
        &self,
        _account: &ConnectedAccount,
        document_id: &str,
    ) -> Result<DocumentContent, ConnectorError> {
        let path = Path::new(document_id);
        
        if !path.exists() {
            return Err(ConnectorError::DocumentNotFound(document_id.to_string()));
        }
        
        let metadata = fs::metadata(path).await?;
        let mime_type = from_path(path).first().map(|m| m.to_string());
        let content_type = self.determine_content_type(&mime_type);
        
        let mut file = fs::File::open(path).await?;
        let mut content = Vec::new();
        file.read_to_end(&mut content).await?;
        
        Ok(DocumentContent {
            metadata: DocumentMetadata {
                external_id: document_id.to_string(),
                name: path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                path: Some(path.to_string_lossy().to_string()),
                mime_type: mime_type.clone(),
                size: Some(metadata.len() as i64),
                created_at: metadata.created().ok().map(|t| chrono::DateTime::from(t)),
                modified_at: metadata.modified().ok().map(|t| chrono::DateTime::from(t)),
                permissions: None,
                url: None,
                parent_id: path.parent()
                    .and_then(|p| p.to_str())
                    .map(|s| s.to_string()),
                is_folder: false,
                metadata: None,
            },
            content,
            content_type,
        })
    }
    
    async fn sync(
        &self,
        account: &ConnectedAccount,
        request: &SyncRequest,
    ) -> Result<(SyncResult, Vec<DocumentForEmbedding>), ConnectorError> {
        let start_time = std::time::Instant::now();
        
        info!("🔄 Starting local file sync for account: {}", account.id);
        
        // List all documents
        let documents = self.list_documents(account, request.filters.clone()).await?;
        
        let mut documents_for_embedding = Vec::new();
        let mut errors = Vec::new();
        
        for doc in &documents {
            match self.process_uploaded_file(
                Path::new(&doc.external_id),
                account.user_id,
            ).await {
                Ok(doc_embedding) => {
                    documents_for_embedding.push(doc_embedding);
                }
                Err(e) => {
                    error!("Failed to process file {}: {}", doc.name, e);
                    errors.push(format!("Failed to process {}: {}", doc.name, e));
                }
            }
        }
        
        let sync_duration = start_time.elapsed().as_millis() as u64;
        
        let result = SyncResult {
            total_documents: documents.len(),
            new_documents: documents_for_embedding.len(),
            updated_documents: 0,
            deleted_documents: 0,
            failed_documents: errors.len(),
            sync_duration_ms: sync_duration,
            errors,
        };
        
        info!("✅ Local file sync completed: {:?}", result);
        
        Ok((result, documents_for_embedding))
    }
    
    async fn incremental_sync(
        &self,
        account: &ConnectedAccount,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<DocumentMetadata>, ConnectorError> {
        // For local files, check modification times
        let all_docs = self.list_documents(account, None).await?;
        
        Ok(all_docs.into_iter()
            .filter(|doc| {
                doc.modified_at
                    .map(|m| m > since)
                    .unwrap_or(false)
            })
            .collect())
    }
    
    async fn disconnect(&mut self, _account: &ConnectedAccount) -> Result<(), ConnectorError> {
        info!("📁 Disconnected from local file system");
        Ok(())
    }
    
    async fn refresh_credentials(&self, _account: &ConnectedAccount) -> Result<OAuthCredentials, ConnectorError> {
        Err(ConnectorError::UnsupportedOperation(
            "Local file connector does not use credentials".to_string()
        ))
    }
}

pub struct LocalFileConnectorFactory;

impl ConnectorFactory for LocalFileConnectorFactory {
    fn create(&self) -> Box<dyn Connector> {
        Box::new(LocalFileConnector::new())
    }
    
    fn connector_type(&self) -> ConnectorType {
        ConnectorType::LocalFile
    }
    
    fn supports_oauth(&self) -> bool {
        false
    }
    
    fn supports_webhooks(&self) -> bool {
        false
    }
}
