use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use conhub_plugins::{
    Plugin, PluginConfig, PluginMetadata, PluginStatus, PluginResult,
    sources::{Document, SourceCapabilities, SourcePlugin, SyncResult},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleDriveConfig {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub folder_id: Option<String>, // Optional: specific folder to sync
    pub include_shared: Option<bool>, // Include shared files
    pub file_types: Option<Vec<String>>, // Filter by file types
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleDriveFile {
    id: String,
    name: String,
    #[serde(rename = "mimeType")]
    mime_type: String,
    size: Option<String>,
    #[serde(rename = "modifiedTime")]
    modified_time: String,
    #[serde(rename = "createdTime")]
    created_time: String,
    #[serde(rename = "webViewLink")]
    web_view_link: Option<String>,
    #[serde(rename = "webContentLink")]
    web_content_link: Option<String>,
    parents: Option<Vec<String>>,
    shared: Option<bool>,
    owners: Option<Vec<GoogleDriveUser>>,
    #[serde(rename = "lastModifyingUser")]
    last_modifying_user: Option<GoogleDriveUser>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleDriveUser {
    #[serde(rename = "displayName")]
    display_name: String,
    #[serde(rename = "emailAddress")]
    email_address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleDriveFileList {
    files: Vec<GoogleDriveFile>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

pub struct GoogleDrivePlugin {
    config: Arc<RwLock<Option<GoogleDriveConfig>>>,
    client: Client,
    status: Arc<RwLock<PluginStatus>>,
}

impl GoogleDrivePlugin {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(None)),
            client: Client::new(),
            status: Arc::new(RwLock::new(PluginStatus::Inactive)),
        }
    }

    async fn get_config(&self) -> Result<GoogleDriveConfig> {
        let config_guard = self.config.read().await;
        config_guard
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow!("Plugin not configured"))
    }

    async fn make_api_request(&self, endpoint: &str, query_params: Option<&[(&str, &str)]>) -> Result<Value> {
        let config = self.get_config().await?;
        let url = format!("https://www.googleapis.com/drive/v3/{}", endpoint);
        
        let mut request = self.client.get(&url)
            .bearer_auth(&config.access_token);
        
        if let Some(params) = query_params {
            request = request.query(params);
        }
        
        let response = request.send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Google Drive API error {}: {}", status, error_text));
        }
        
        let json: Value = response.json().await?;
        Ok(json)
    }

    async fn download_file_content(&self, file_id: &str) -> Result<Vec<u8>> {
        let config = self.get_config().await?;
        let url = format!("https://www.googleapis.com/drive/v3/files/{}", file_id);
        
        let response = self.client.get(&url)
            .bearer_auth(&config.access_token)
            .query(&[("alt", "media")])
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Failed to download file {}: {} - {}", file_id, status, error_text));
        }
        
        let content = response.bytes().await?;
        Ok(content.to_vec())
    }

    fn convert_drive_file_to_document(&self, file: GoogleDriveFile) -> Result<Document> {
        let size = file.size.as_ref()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        
        let modified_time = DateTime::parse_from_rfc3339(&file.modified_time)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        
        let created_time = DateTime::parse_from_rfc3339(&file.created_time)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        
        let mut metadata = HashMap::new();
        metadata.insert("mime_type".to_string(), file.mime_type.clone());
        metadata.insert("google_drive_id".to_string(), file.id.clone());
        
        if let Some(web_view_link) = &file.web_view_link {
            metadata.insert("web_view_link".to_string(), web_view_link.clone());
        }
        
        if let Some(web_content_link) = &file.web_content_link {
            metadata.insert("web_content_link".to_string(), web_content_link.clone());
        }
        
        if let Some(shared) = file.shared {
            metadata.insert("shared".to_string(), shared.to_string());
        }
        
        if let Some(owners) = &file.owners {
            if let Some(owner) = owners.first() {
                metadata.insert("owner".to_string(), owner.display_name.clone());
                if let Some(email) = &owner.email_address {
                    metadata.insert("owner_email".to_string(), email.clone());
                }
            }
        }
        
        // Determine if this is a Google Workspace document
        let is_google_doc = file.mime_type.starts_with("application/vnd.google-apps.");
        let file_extension = if is_google_doc {
            match file.mime_type.as_str() {
                "application/vnd.google-apps.document" => "gdoc",
                "application/vnd.google-apps.spreadsheet" => "gsheet",
                "application/vnd.google-apps.presentation" => "gslides",
                "application/vnd.google-apps.form" => "gform",
                "application/vnd.google-apps.drawing" => "gdraw",
                _ => "gdoc",
            }
        } else {
            // Extract extension from filename
            file.name.split('.').last().unwrap_or("unknown")
        };
        
        let mut doc_metadata = HashMap::new();
        doc_metadata.insert("file_type".to_string(), serde_json::Value::String(file_extension.to_string()));
        doc_metadata.insert("size".to_string(), serde_json::Value::Number(serde_json::Number::from(size)));
        doc_metadata.insert("created_at".to_string(), serde_json::Value::String(created_time.to_rfc3339()));
        doc_metadata.insert("modified_at".to_string(), serde_json::Value::String(modified_time.to_rfc3339()));
        doc_metadata.insert("author".to_string(), serde_json::Value::String(
            file.owners
                .and_then(|owners| owners.first().map(|o| o.display_name.clone()))
                .unwrap_or_else(|| "Unknown".to_string())
        ));
        doc_metadata.insert("tags".to_string(), serde_json::Value::Array(vec![serde_json::Value::String("google_drive".to_string())]));
        doc_metadata.insert("source".to_string(), serde_json::Value::String("google_drive".to_string()));
        
        // Add any additional metadata
        for (key, value) in metadata {
            doc_metadata.insert(key, serde_json::Value::String(value));
        }

        Ok(Document {
            id: file.id.clone(),
            title: file.name,
            content: String::new(), // Content will be loaded separately if needed
            content_type: file.mime_type,
            size,
            created_at: created_time,
            modified_at: modified_time,
            path: format!("/google-drive/{}", file.id),
            metadata: doc_metadata,
        })
    }

    fn list_files_recursive<'a>(&'a self, folder_id: Option<&'a str>, page_token: Option<&'a str>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<GoogleDriveFile>>> + Send + 'a>> {
        Box::pin(async move {
        let mut query = "trashed=false".to_string();
        
        if let Some(folder) = folder_id {
            query.push_str(&format!(" and '{}' in parents", folder));
        }
        
        let config = self.get_config().await?;
        
        // Apply file type filters if specified
        if let Some(file_types) = &config.file_types {
            if !file_types.is_empty() {
                let mime_conditions: Vec<String> = file_types
                    .iter()
                    .map(|ft| format!("mimeType contains '{}'", ft))
                    .collect();
                query.push_str(&format!(" and ({})", mime_conditions.join(" or ")));
            }
        }
        
        let mut params = vec![
            ("q", query.as_str()),
            ("fields", "nextPageToken,files(id,name,mimeType,size,modifiedTime,createdTime,webViewLink,webContentLink,parents,shared,owners,lastModifyingUser)"),
            ("pageSize", "1000"),
        ];
        
        if let Some(token) = page_token {
            params.push(("pageToken", token));
        }
        
        let response = self.make_api_request("files", Some(&params)).await?;
        let file_list: GoogleDriveFileList = serde_json::from_value(response)?;
        
        let mut all_files = file_list.files;
        
        // Handle pagination
        if let Some(next_token) = file_list.next_page_token {
            let mut next_files = self.list_files_recursive(folder_id, Some(&next_token)).await?;
            all_files.append(&mut next_files);
        }
        
        Ok(all_files)
        })
    }
}

#[async_trait]
impl Plugin for GoogleDrivePlugin {
    fn metadata(&self) -> &PluginMetadata {
        static METADATA: std::sync::OnceLock<PluginMetadata> = std::sync::OnceLock::new();
        METADATA.get_or_init(|| PluginMetadata {
            id: "google-drive".to_string(),
            name: "Google Drive".to_string(),
            version: "0.1.0".to_string(),
            description: "Google Drive integration for document management".to_string(),
            author: "ConHub Team".to_string(),
            plugin_type: conhub_plugins::PluginType::Source,
            capabilities: vec![
                "oauth2".to_string(),
                "real_time_sync".to_string(),
                "shared_files".to_string(),
                "folder_sync".to_string(),
            ],
            config_schema: Some(json!({
                "type": "object",
                "properties": {
                    "access_token": {
                        "type": "string",
                        "description": "Google Drive OAuth2 access token",
                        "required": true
                    },
                    "refresh_token": {
                        "type": "string",
                        "description": "OAuth2 refresh token for token renewal"
                    },
                    "client_id": {
                        "type": "string",
                        "description": "Google OAuth2 client ID"
                    },
                    "client_secret": {
                        "type": "string",
                        "description": "Google OAuth2 client secret"
                    },
                    "folder_id": {
                        "type": "string",
                        "description": "Specific folder ID to sync (optional)"
                    },
                    "include_shared": {
                        "type": "boolean",
                        "description": "Include shared files in sync",
                        "default": false
                    },
                    "file_types": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "File types to include (e.g., ['pdf', 'docx'])"
                    }
                },
                "required": ["access_token"]
            })),
        })
    }

    async fn initialize(&mut self, config: PluginConfig) -> Result<(), conhub_plugins::error::PluginError> {
        info!("Initializing Google Drive plugin");
        
        let google_drive_config: GoogleDriveConfig = serde_json::from_value(
            config.settings.get("google_drive").unwrap_or(&json!({})).clone()
        ).map_err(|e| conhub_plugins::error::PluginError::ConfigurationError(e.to_string()))?;
        
        let mut config_guard = self.config.write().await;
        *config_guard = Some(google_drive_config);
        
        let mut status_guard = self.status.write().await;
        *status_guard = PluginStatus::Inactive;
        
        info!("Google Drive plugin initialized successfully");
        Ok(())
    }

    async fn start(&mut self) -> Result<(), conhub_plugins::error::PluginError> {
        info!("Starting Google Drive plugin");
        
        let config_guard = self.config.read().await;
        if config_guard.is_none() {
            return Err(conhub_plugins::error::PluginError::InitializationFailed("Plugin not initialized".to_string()));
        }
        
        let mut status_guard = self.status.write().await;
        *status_guard = PluginStatus::Active;
        
        info!("Google Drive plugin started successfully");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), conhub_plugins::error::PluginError> {
        info!("Stopping Google Drive plugin");
        
        let mut status_guard = self.status.write().await;
        *status_guard = PluginStatus::Inactive;
        
        info!("Google Drive plugin stopped");
        Ok(())
    }

    fn status(&self) -> PluginStatus {
        match self.status.try_read() {
            Ok(status) => status.clone(),
            Err(_) => PluginStatus::Error("Failed to read status".to_string()),
        }
    }

    async fn health_check(&self) -> Result<bool, conhub_plugins::error::PluginError> {
        match self.get_config().await {
            Ok(config) => {
                // Test API connectivity
                let response = self.client
                    .get("https://www.googleapis.com/drive/v3/about")
                    .bearer_auth(&config.access_token)
                    .query(&[("fields", "user")])
                    .send()
                    .await;
                
                match response {
                    Ok(resp) => Ok(resp.status().is_success()),
                    Err(e) => Err(conhub_plugins::error::PluginError::NetworkError(e.to_string())),
                }
            }
            Err(e) => Err(conhub_plugins::error::PluginError::ConfigurationError(e.to_string())),
        }
    }

    fn validate_config(&self, config: &PluginConfig) -> Result<(), conhub_plugins::error::PluginError> {
        let _: GoogleDriveConfig = serde_json::from_value(
            config.settings.get("google_drive").unwrap_or(&json!({})).clone()
        ).map_err(|e| conhub_plugins::error::PluginError::ValidationError(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl SourcePlugin for GoogleDrivePlugin {
    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities {
            can_read: true,
            can_write: true,
            can_delete: true,
            supports_real_time: false,
            supports_search: true,
            supports_metadata: true,
            max_file_size: Some(5 * 1024 * 1024 * 1024), // 5GB limit for Google Drive
            supported_formats: vec![
                "application/pdf".to_string(),
                "text/plain".to_string(),
                "application/msword".to_string(),
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
                "application/vnd.google-apps.document".to_string(),
                "application/vnd.google-apps.spreadsheet".to_string(),
                "application/vnd.google-apps.presentation".to_string(),
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/gif".to_string(),
            ],
        }
    }

    async fn list_documents(&self) -> PluginResult<Vec<Document>> {
        info!("Listing Google Drive documents");
        
        let config = self.get_config().await.map_err(|e| conhub_plugins::error::PluginError::from(e))?;
        let folder_id = config.folder_id.as_deref();
        
        let files = self.list_files_recursive(folder_id, None).await.map_err(|e| conhub_plugins::error::PluginError::from(e))?;
        
        let documents: Result<Vec<Document>> = files
            .into_iter()
            .map(|file| self.convert_drive_file_to_document(file))
            .collect();
        
        let docs = documents.map_err(|e| conhub_plugins::error::PluginError::from(e))?;
        info!("Listed {} documents from Google Drive", docs.len());
        Ok(docs)
    }

    async fn get_document(&self, document_id: &str) -> PluginResult<Document> {
        info!("Getting document {} from Google Drive", document_id);
        
        let response = self.make_api_request(
            &format!("files/{}", document_id),
            Some(&[("fields", "id,name,mimeType,size,modifiedTime,createdTime,webViewLink,webContentLink,parents,shared,owners,lastModifyingUser")])
        ).await.map_err(|e| conhub_plugins::error::PluginError::from(e))?;
        
        let file: GoogleDriveFile = serde_json::from_value(response).map_err(|e| anyhow::anyhow!(e))?;
        let mut document = self.convert_drive_file_to_document(file).map_err(|e| conhub_plugins::error::PluginError::from(e))?;
        
        // Download content for text-based files
        let mime_type = document.metadata.get("mime_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        if document.content_type == "text/plain" || 
           document.content_type == "application/vnd.google-apps.document" ||
           mime_type.starts_with("text/") {
            
            match self.download_file_content(document_id).await {
                Ok(content) => {
                    document.content = String::from_utf8_lossy(&content).to_string();
                }
                Err(e) => {
                    warn!("Failed to download content for document {}: {}", document_id, e);
                }
            }
        }
        
        info!("Retrieved document {} from Google Drive", document_id);
        Ok(document)
    }

    async fn search_documents(&self, query: &str) -> PluginResult<Vec<Document>> {
        info!("Searching Google Drive documents with query: {}", query);
        
        let search_query = format!("trashed=false and fullText contains '{}'", query);
        let page_size = "100".to_string();
        let params = vec![
            ("q", search_query.as_str()),
            ("fields", "files(id,name,mimeType,size,modifiedTime,createdTime,webViewLink,webContentLink,parents,shared,owners,lastModifyingUser)"),
            ("pageSize", &page_size),
        ];
        
        let response = self.make_api_request("files", Some(&params)).await.map_err(|e| conhub_plugins::error::PluginError::from(e))?;
        let file_list: GoogleDriveFileList = serde_json::from_value(response).map_err(|e| anyhow::anyhow!(e))?;
        
        let documents: Result<Vec<Document>> = file_list.files
            .into_iter()
            .map(|file| self.convert_drive_file_to_document(file))
            .collect();
        
        let docs = documents.map_err(|e| conhub_plugins::error::PluginError::from(e))?;
        info!("Found {} documents matching query '{}'", docs.len(), query);
        Ok(docs)
    }

    async fn sync(&self) -> PluginResult<SyncResult> {
        info!("Starting Google Drive sync");
        
        let config = self.get_config().await.map_err(|e| conhub_plugins::error::PluginError::from(e))?;
        let folder_id = config.folder_id.as_deref();
        
        let files = self.list_files_recursive(folder_id, None).await.map_err(|e| conhub_plugins::error::PluginError::from(e))?;
        
        let mut sync_result = SyncResult {
            total_documents: files.len() as u64,
            new_documents: 0,
            updated_documents: 0,
            deleted_documents: 0,
            errors: Vec::new(),
            duration_ms: 0,
        };
        
        for file in files {
            match self.convert_drive_file_to_document(file) {
                Ok(_document) => {
                    sync_result.new_documents += 1;
                    // Here you would typically save the document to your database
                }
                Err(e) => {
                    sync_result.errors.push(format!("Failed to process file: {}", e));
                }
            }
        }
        
        info!(
            "Google Drive sync completed: {} new documents, {} errors",
            sync_result.new_documents, sync_result.errors.len()
        );
        
        Ok(sync_result)
    }

    async fn upload_document(&self, document: Document, content: Vec<u8>) -> PluginResult<String> {
        info!("Uploading document to Google Drive: {}", document.title);
        
        let config = self.get_config().await?;
        
        // Create file metadata
        let mut metadata = json!({
            "name": document.title
        });
        
        // Set parent folder if specified
        if let Some(folder_id) = &config.folder_id {
            metadata["parents"] = json!([folder_id]);
        }
        
        // Create multipart upload
        let form = reqwest::multipart::Form::new()
            .text("metadata", metadata.to_string())
            .text("media", document.content);
        
        let response = self.client
            .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart")
            .bearer_auth(&config.access_token)
            .multipart(form)
            .send()
            .await
            .map_err(|e| conhub_plugins::error::PluginError::from(anyhow::anyhow!(e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(conhub_plugins::error::PluginError::from(anyhow!("Failed to upload document: {} - {}", status, error_text)));
        }
        
        let result: Value = response.json().await.map_err(|e| anyhow::anyhow!(e))?;
        let file_id = result["id"].as_str()
            .ok_or_else(|| anyhow!("No file ID returned from upload"))?;
        
        info!("Document uploaded to Google Drive with ID: {}", file_id);
        Ok(file_id.to_string())
    }

    async fn delete_document(&self, document_id: &str) -> PluginResult<()> {
        info!("Deleting document {} from Google Drive", document_id);
        
        let config = self.get_config().await.map_err(|e| conhub_plugins::error::PluginError::from(e))?;
        let url = format!("https://www.googleapis.com/drive/v3/files/{}", document_id);
        
        let response = self.client
            .delete(&url)
            .bearer_auth(&config.access_token)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(conhub_plugins::error::PluginError::from(anyhow!("Failed to delete document: {} - {}", status, error_text)));
        }
        
        info!("Document {} deleted from Google Drive", document_id);
        Ok(())
    }

    async fn get_content(&self, document_id: &str) -> PluginResult<Vec<u8>> {
        info!("Getting content for document {} from Google Drive", document_id);
        self.download_file_content(document_id).await.map_err(|e| conhub_plugins::error::PluginError::from(e))
    }

    async fn setup_realtime_sync(&self) -> PluginResult<()> {
        warn!("Real-time sync is not yet implemented for Google Drive");
        Err(conhub_plugins::error::PluginError::from(anyhow!("Real-time sync not implemented")))
    }
}

// Export the plugin factory function
#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin + Send + Sync> {
    Box::new(GoogleDrivePlugin::new())
}