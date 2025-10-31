use conhub_plugins::{
    Plugin, PluginConfig, PluginMetadata, PluginStatus, PluginType, PluginResult,
    sources::{SourcePlugin, SourcePluginFactory, Document, SyncResult, SourceCapabilities},
    error::PluginError,
};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{info, error, warn};

pub struct DropboxPlugin {
    metadata: PluginMetadata,
    status: PluginStatus,
    config: Option<PluginConfig>,
    client: Option<reqwest::Client>,
    access_token: Option<String>,
}

impl DropboxPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "dropbox".to_string(),
                name: "Dropbox Source".to_string(),
                version: "1.0.0".to_string(),
                description: "Dropbox file source integration".to_string(),
                author: "ConHub Team".to_string(),
                plugin_type: PluginType::Source,
                capabilities: vec![
                    "read".to_string(),
                    "search".to_string(),
                    "sync".to_string(),
                ],
                config_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "access_token": {
                            "type": "string",
                            "description": "Dropbox API access token"
                        },
                        "sync_interval_minutes": {
                            "type": "number",
                            "description": "Sync interval in minutes",
                            "default": 30
                        }
                    },
                    "required": ["access_token"]
                })),
            },
            status: PluginStatus::Inactive,
            config: None,
            client: None,
            access_token: None,
        }
    }
}

#[async_trait]
impl Plugin for DropboxPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn initialize(&mut self, config: PluginConfig) -> PluginResult<()> {
        info!("[Dropbox Plugin] Initializing...");
        self.status = PluginStatus::Loading;

        // Validate configuration
        self.validate_config(&config)?;

        // Extract access token
        let access_token = config.settings.get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PluginError::ConfigurationError("Missing access_token".to_string()))?;

        self.access_token = Some(access_token.to_string());
        self.client = Some(reqwest::Client::new());
        self.config = Some(config);

        info!("[Dropbox Plugin] Initialized successfully");
        Ok(())
    }

    async fn start(&mut self) -> PluginResult<()> {
        info!("[Dropbox Plugin] Starting...");
        
        // Test connection
        if let Err(e) = self.health_check().await {
            self.status = PluginStatus::Error(format!("Health check failed: {}", e));
            return Err(e);
        }

        self.status = PluginStatus::Active;
        info!("[Dropbox Plugin] Started successfully");
        Ok(())
    }

    async fn stop(&mut self) -> PluginResult<()> {
        info!("[Dropbox Plugin] Stopping...");
        self.status = PluginStatus::Inactive;
        self.client = None;
        info!("[Dropbox Plugin] Stopped");
        Ok(())
    }

    fn status(&self) -> PluginStatus {
        self.status.clone()
    }

    async fn health_check(&self) -> PluginResult<bool> {
        let client = self.client.as_ref()
            .ok_or_else(|| PluginError::RuntimeError("Client not initialized".to_string()))?;
        
        let access_token = self.access_token.as_ref()
            .ok_or_else(|| PluginError::RuntimeError("Access token not set".to_string()))?;

        // Test Dropbox API connection
        let response = client
            .post("https://api.dropboxapi.com/2/users/get_current_account")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| PluginError::NetworkError(e.to_string()))?;

        if response.status().is_success() {
            Ok(true)
        } else {
            Err(PluginError::AuthenticationError("Invalid access token".to_string()))
        }
    }

    fn validate_config(&self, config: &PluginConfig) -> PluginResult<()> {
        if !config.settings.contains_key("access_token") {
            return Err(PluginError::ValidationError("Missing access_token".to_string()));
        }

        if let Some(token) = config.settings.get("access_token").and_then(|v| v.as_str()) {
            if token.is_empty() {
                return Err(PluginError::ValidationError("Empty access_token".to_string()));
            }
        }

        Ok(())
    }
}

#[async_trait]
impl SourcePlugin for DropboxPlugin {
    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities {
            can_read: true,
            can_write: false,
            can_delete: false,
            supports_real_time: false,
            supports_search: true,
            supports_metadata: true,
            max_file_size: Some(150 * 1024 * 1024), // 150MB
            supported_formats: vec![
                "txt".to_string(),
                "pdf".to_string(),
                "doc".to_string(),
                "docx".to_string(),
                "md".to_string(),
            ],
        }
    }

    async fn list_documents(&self) -> PluginResult<Vec<Document>> {
        let client = self.client.as_ref()
            .ok_or_else(|| PluginError::RuntimeError("Client not initialized".to_string()))?;
        
        let access_token = self.access_token.as_ref()
            .ok_or_else(|| PluginError::RuntimeError("Access token not set".to_string()))?;

        // Call Dropbox API to list files
        let response = client
            .post("https://api.dropboxapi.com/2/files/list_folder")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "path": "",
                "recursive": true
            }))
            .send()
            .await
            .map_err(|e| PluginError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(PluginError::RuntimeError("Failed to list files".to_string()));
        }

        let data: Value = response.json().await
            .map_err(|e| PluginError::RuntimeError(e.to_string()))?;

        let mut documents = Vec::new();
        
        if let Some(entries) = data.get("entries").and_then(|v| v.as_array()) {
            for entry in entries {
                if let Some(doc) = self.parse_dropbox_entry(entry) {
                    documents.push(doc);
                }
            }
        }

        Ok(documents)
    }

    async fn get_document(&self, id: &str) -> PluginResult<Document> {
        // Implementation would fetch specific document metadata
        Err(PluginError::RuntimeError("Not implemented".to_string()))
    }

    async fn search_documents(&self, query: &str) -> PluginResult<Vec<Document>> {
        let client = self.client.as_ref()
            .ok_or_else(|| PluginError::RuntimeError("Client not initialized".to_string()))?;
        
        let access_token = self.access_token.as_ref()
            .ok_or_else(|| PluginError::RuntimeError("Access token not set".to_string()))?;

        // Call Dropbox search API
        let response = client
            .post("https://api.dropboxapi.com/2/files/search_v2")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "query": query,
                "options": {
                    "path": "",
                    "max_results": 100
                }
            }))
            .send()
            .await
            .map_err(|e| PluginError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(PluginError::RuntimeError("Search failed".to_string()));
        }

        let data: Value = response.json().await
            .map_err(|e| PluginError::RuntimeError(e.to_string()))?;

        let mut documents = Vec::new();
        
        if let Some(matches) = data.get("matches").and_then(|v| v.as_array()) {
            for match_entry in matches {
                if let Some(metadata) = match_entry.get("metadata").and_then(|m| m.get("metadata")) {
                    if let Some(doc) = self.parse_dropbox_entry(metadata) {
                        documents.push(doc);
                    }
                }
            }
        }

        Ok(documents)
    }

    async fn sync(&self) -> PluginResult<SyncResult> {
        let start_time = std::time::Instant::now();
        
        // Get all documents
        let documents = self.list_documents().await?;
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(SyncResult {
            total_documents: documents.len() as u64,
            new_documents: documents.len() as u64, // Simplified
            updated_documents: 0,
            deleted_documents: 0,
            errors: vec![],
            duration_ms,
        })
    }

    async fn get_content(&self, id: &str) -> PluginResult<Vec<u8>> {
        // Implementation would download file content
        Err(PluginError::RuntimeError("Not implemented".to_string()))
    }

    async fn upload_document(&self, _document: Document, _content: Vec<u8>) -> PluginResult<String> {
        Err(PluginError::RuntimeError("Upload not supported".to_string()))
    }

    async fn delete_document(&self, _id: &str) -> PluginResult<()> {
        Err(PluginError::RuntimeError("Delete not supported".to_string()))
    }

    async fn setup_realtime_sync(&self) -> PluginResult<()> {
        Err(PluginError::RuntimeError("Real-time sync not supported".to_string()))
    }
}

impl DropboxPlugin {
    fn parse_dropbox_entry(&self, entry: &Value) -> Option<Document> {
        let tag = entry.get(".tag")?.as_str()?;
        if tag != "file" {
            return None;
        }

        let id = entry.get("id")?.as_str()?.to_string();
        let name = entry.get("name")?.as_str()?.to_string();
        let path = entry.get("path_display")?.as_str()?.to_string();
        let size = entry.get("size")?.as_u64().unwrap_or(0);
        
        let modified_at = entry.get("client_modified")?.as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        let mut metadata = HashMap::new();
        metadata.insert("dropbox_id".to_string(), serde_json::Value::String(id.clone()));
        metadata.insert("path".to_string(), serde_json::Value::String(path.clone()));

        Some(Document {
            id,
            title: name,
            content: String::new(), // Content loaded separately
            content_type: "application/octet-stream".to_string(),
            size,
            created_at: modified_at, // Dropbox doesn't provide created_at
            modified_at,
            path,
            metadata,
        })
    }
}

/// Plugin factory for creating Dropbox plugin instances
pub struct DropboxPluginFactory;

impl DropboxPluginFactory {
    pub fn new() -> Self {
        Self
    }
}

impl SourcePluginFactory for DropboxPluginFactory {
    fn create(&self) -> Box<dyn SourcePlugin> {
        Box::new(DropboxPlugin::new())
    }

    fn source_type(&self) -> &str {
        "dropbox"
    }
}