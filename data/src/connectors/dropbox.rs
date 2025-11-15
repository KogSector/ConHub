use async_trait::async_trait;
use uuid::Uuid;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};
use std::collections::HashMap;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    RedirectUrl, Scope, TokenResponse, TokenUrl,
    basic::BasicClient,
    reqwest::async_http_client,
};

use super::traits::{Connector, ConnectorFactory, WebhookConnector};
use super::types::*;
use std::env;
use super::error::ConnectorError;

/// Dropbox API connector
pub struct DropboxConnector {
    name: String,
    client: Client,
    oauth_client: Option<BasicClient>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DropboxFile {
    #[serde(rename = ".tag")]
    tag: String,
    name: String,
    id: String,
    path_lower: String,
    path_display: String,
    size: Option<u64>,
    server_modified: Option<String>,
    content_hash: Option<String>,
    rev: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DropboxListFolderResult {
    entries: Vec<DropboxFile>,
    cursor: String,
    has_more: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct DropboxListFolderRequest {
    path: String,
    recursive: bool,
    include_media_info: bool,
    include_deleted: bool,
    include_has_explicit_shared_members: bool,
}

impl DropboxConnector {
    pub fn new() -> Self {
        Self {
            name: "Dropbox".to_string(),
            client: Client::new(),
            oauth_client: None,
        }
    }
    
    pub fn factory() -> DropboxConnectorFactory {
        DropboxConnectorFactory
    }
    
    async fn get_access_token(&self, account: &ConnectedAccount) -> Result<String, ConnectorError> {
        account.credentials
            .get("access_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ConnectorError::AuthenticationFailed(
                "No access token found".to_string()
            ))
    }
    
    async fn list_folder(
        &self,
        access_token: &str,
        path: &str,
        recursive: bool,
    ) -> Result<Vec<DropboxFile>, ConnectorError> {
        let request_body = DropboxListFolderRequest {
            path: if path.is_empty() { "".to_string() } else { path.to_string() },
            recursive,
            include_media_info: false,
            include_deleted: false,
            include_has_explicit_shared_members: false,
        };
        
        let mut all_entries = Vec::new();
        let mut cursor: Option<String> = None;
        
        loop {
            let url = if cursor.is_some() {
                "https://api.dropboxapi.com/2/files/list_folder/continue"
            } else {
                "https://api.dropboxapi.com/2/files/list_folder"
            };
            
            let body = if let Some(ref cursor_val) = cursor {
                serde_json::json!({ "cursor": cursor_val })
            } else {
                serde_json::to_value(&request_body)?
            };
            
            let response = self.client
                .post(url)
                .header("Authorization", format!("Bearer {}", access_token))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await?;
            
            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                return Err(ConnectorError::HttpError(
                    format!("Dropbox API error {}: {}", status, error_text)
                ));
            }
            
            let result: DropboxListFolderResult = response.json().await?;
            all_entries.extend(result.entries);
            
            if result.has_more {
                cursor = Some(result.cursor);
            } else {
                break;
            }
        }
        
        Ok(all_entries)
    }
    
    async fn download_file(&self, access_token: &str, path: &str) -> Result<Vec<u8>, ConnectorError> {
        let response = self.client
            .post("https://content.dropboxapi.com/2/files/download")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Dropbox-API-Arg", serde_json::to_string(&serde_json::json!({ "path": path }))?)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(ConnectorError::HttpError(
                format!("Dropbox download error {}: {}", status, error_text)
            ));
        }
        
        let content = response.bytes().await?.to_vec();
        Ok(content)
    }
    
    fn should_include_file(&self, file: &DropboxFile, filters: &Option<SyncFilters>) -> bool {
        // Only process files, not folders
        if file.tag != "file" {
            return false;
        }
        
        if let Some(filters) = filters {
            // Check include paths
            if let Some(ref include_paths) = filters.include_paths {
                if !include_paths.iter().any(|pattern| file.path_display.contains(pattern)) {
                    return false;
                }
            }
            
            // Check exclude paths
            if let Some(ref exclude_paths) = filters.exclude_paths {
                if exclude_paths.iter().any(|pattern| file.path_display.contains(pattern)) {
                    return false;
                }
            }
            
            // Check file types
            if let Some(ref file_types) = filters.file_types {
                let extension = std::path::Path::new(&file.name)
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("");
                
                if !file_types.iter().any(|ft| ft == extension || ft == "*") {
                    return false;
                }
            }
            
            // Check max file size
            if let Some(max_size) = filters.max_file_size {
                if let Some(size) = file.size {
                    if size > max_size as u64 {
                        return false;
                    }
                }
            }
        }
        
        true
    }
    
    fn determine_content_type(&self, file_name: &str) -> ContentType {
        let extension = std::path::Path::new(file_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        match extension.to_lowercase().as_str() {
            "txt" | "md" | "markdown" => ContentType::Text,
            "html" | "htm" => ContentType::Html,
            "pdf" => ContentType::Pdf,
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" => ContentType::Image,
            "mp4" | "avi" | "mov" | "wmv" => ContentType::Video,
            "mp3" | "wav" | "flac" | "aac" => ContentType::Audio,
            "zip" | "rar" | "7z" | "tar" | "gz" => ContentType::Archive,
            "rs" | "py" | "js" | "ts" | "java" | "cpp" | "c" | "go" => ContentType::Code,
            _ => ContentType::Binary,
        }
    }
    
    fn chunk_content(&self, content: &str, file_name: &str) -> Vec<DocumentChunk> {
        const CHUNK_SIZE: usize = 1000;
        const CHUNK_OVERLAP: usize = 200;
        
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
                    "file_name": file_name,
                    "length": chunk_content.len(),
                })),
            });
            
            chunk_number += 1;
            start = end.saturating_sub(CHUNK_OVERLAP);
            
            if start + CHUNK_SIZE >= content_len && end == content_len {
                break;
            }
        }
        
        chunks
    }
}

#[async_trait]
impl Connector for DropboxConnector {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn connector_type(&self) -> ConnectorType {
        ConnectorType::Dropbox
    }
    
    fn validate_config(&self, config: &ConnectorConfigAuth) -> Result<(), ConnectorError> {
        if config.credentials.get("client_id").is_none() {
            return Err(ConnectorError::InvalidConfiguration(
                "Dropbox client_id is required".to_string()
            ));
        }
        
        if config.credentials.get("client_secret").is_none() {
            return Err(ConnectorError::InvalidConfiguration(
                "Dropbox client_secret is required".to_string()
            ));
        }
        
        Ok(())
    }
    
    async fn authenticate(&self, config: &ConnectorConfigAuth) -> Result<Option<String>, ConnectorError> {
        let client_id = config.credentials.get("client_id")
            .cloned()
            .or_else(|| env::var("DROPBOX_CLIENT_ID").ok())
            .ok_or_else(|| ConnectorError::InvalidConfiguration("Dropbox client_id is required".to_string()))?;
        
        let client_secret = config.credentials.get("client_secret")
            .cloned()
            .or_else(|| env::var("DROPBOX_CLIENT_SECRET").ok())
            .ok_or_else(|| ConnectorError::InvalidConfiguration("Dropbox client_secret is required".to_string()))?;
        
        let redirect_url = config.settings.get("redirect_url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| env::var("DROPBOX_REDIRECT_URL").ok())
            .unwrap_or_else(|| "http://localhost:3000/auth/dropbox/callback".to_string());
        
        let client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new("https://www.dropbox.com/oauth2/authorize".to_string())
                .map_err(|e| ConnectorError::InvalidConfiguration(e.to_string()))?,
            Some(TokenUrl::new("https://api.dropboxapi.com/oauth2/token".to_string())
                .map_err(|e| ConnectorError::InvalidConfiguration(e.to_string()))?)
        )
        .set_redirect_uri(
            RedirectUrl::new(redirect_url)
                .map_err(|e| ConnectorError::InvalidConfiguration(e.to_string()))?
        );
        
        let (auth_url, _csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .url();
        
        Ok(Some(auth_url.to_string()))
    }
    
    async fn complete_oauth(&self, callback_data: OAuthCallbackData) -> Result<OAuthCredentials, ConnectorError> {
        let client_id = env::var("DROPBOX_CLIENT_ID")
            .map_err(|_| ConnectorError::InvalidConfiguration("DROPBOX_CLIENT_ID not set".to_string()))?;
        let client_secret = env::var("DROPBOX_CLIENT_SECRET")
            .map_err(|_| ConnectorError::InvalidConfiguration("DROPBOX_CLIENT_SECRET not set".to_string()))?;
        let redirect_url = env::var("DROPBOX_REDIRECT_URL")
            .unwrap_or_else(|_| "http://localhost:3000/auth/dropbox/callback".to_string());

        let client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new("https://www.dropbox.com/oauth2/authorize".to_string())
                .map_err(|e| ConnectorError::InvalidConfiguration(e.to_string()))?,
            Some(TokenUrl::new("https://api.dropboxapi.com/oauth2/token".to_string())
                .map_err(|e| ConnectorError::InvalidConfiguration(e.to_string()))?)
        )
        .set_redirect_uri(
            RedirectUrl::new(redirect_url)
                .map_err(|e| ConnectorError::InvalidConfiguration(e.to_string()))?
        );
        
        let token_result = client
            .exchange_code(AuthorizationCode::new(callback_data.code))
            .request_async(async_http_client)
            .await
            .map_err(|e| ConnectorError::AuthenticationFailed(e.to_string()))?;
        
        Ok(OAuthCredentials {
            access_token: token_result.access_token().secret().clone(),
            refresh_token: token_result.refresh_token().map(|t| t.secret().clone()),
            token_type: "Bearer".to_string(),
            expires_in: token_result.expires_in().map(|d| d.as_secs() as i64),
            expires_at: None,
            scope: None,
            metadata: HashMap::new(),
        })
    }
    
    async fn connect(&mut self, account: &ConnectedAccount) -> Result<(), ConnectorError> {
        info!("🔌 Connecting to Dropbox for account: {}", account.account_name);
        
        let access_token = self.get_access_token(account).await?;
        
        // Test connection by listing root folder
        self.list_folder(&access_token, "", false).await?;
        
        info!("✅ Successfully connected to Dropbox");
        Ok(())
    }
    
    async fn check_connection(&self, account: &ConnectedAccount) -> Result<bool, ConnectorError> {
        let access_token = self.get_access_token(account).await?;
        Ok(self.list_folder(&access_token, "", false).await.is_ok())
    }
    
    async fn list_documents(
        &self,
        account: &ConnectedAccount,
        filters: Option<SyncFilters>,
    ) -> Result<Vec<DocumentMetadata>, ConnectorError> {
        let access_token = self.get_access_token(account).await?;
        
        // Get folder paths from account metadata or use root
        let folder_paths = account.metadata
            .as_ref()
            .and_then(|m| m.get("folder_paths"))
            .and_then(|fp| fp.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_else(|| vec![""]);
        
        let mut all_documents = Vec::new();
        
        for folder_path in folder_paths {
            info!("📁 Listing files in Dropbox folder: {}", folder_path);
            
            match self.list_folder(&access_token, folder_path, true).await {
                Ok(files) => {
                    for file in files {
                        if !self.should_include_file(&file, &filters) {
                            continue;
                        }
                        
                        all_documents.push(DocumentMetadata {
                            external_id: file.id.clone(),
                            name: file.name.clone(),
                            path: Some(file.path_display.clone()),
                            mime_type: mime_guess::from_path(&file.name).first().map(|m| m.to_string()),
                            size: file.size.map(|s| s as i64),
                            created_at: None,
                            modified_at: file.server_modified.as_ref()
                                .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                                .map(|dt| dt.with_timezone(&chrono::Utc)),
                            permissions: None,
                            url: Some(format!("https://www.dropbox.com/home{}", file.path_display)),
                            parent_id: None,
                            is_folder: file.tag == "folder",
                            metadata: Some(serde_json::json!({
                                "content_hash": file.content_hash,
                                "rev": file.rev,
                                "path_lower": file.path_lower,
                            })),
                        });
                    }
                }
                Err(e) => {
                    error!("Failed to list Dropbox folder {}: {}", folder_path, e);
                }
            }
        }
        
        Ok(all_documents)
    }
    
    async fn get_document_content(
        &self,
        account: &ConnectedAccount,
        document_id: &str,
    ) -> Result<DocumentContent, ConnectorError> {
        let access_token = self.get_access_token(account).await?;
        
        // For Dropbox, document_id is the file path
        let content = self.download_file(&access_token, document_id).await?;
        
        let file_name = document_id.split('/').last().unwrap_or(document_id);
        let content_type = self.determine_content_type(file_name);
        
        Ok(DocumentContent {
            metadata: DocumentMetadata {
                external_id: document_id.to_string(),
                name: file_name.to_string(),
                path: Some(document_id.to_string()),
                mime_type: mime_guess::from_path(file_name).first().map(|m| m.to_string()),
                size: Some(content.len() as i64),
                created_at: None,
                modified_at: Some(chrono::Utc::now()),
                permissions: None,
                url: Some(format!("https://www.dropbox.com/home{}", document_id)),
                parent_id: None,
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
        request: &SyncRequestWithFilters,
    ) -> Result<(SyncResult, Vec<DocumentForEmbedding>), ConnectorError> {
        let start_time = std::time::Instant::now();
        
        info!("🔄 Starting Dropbox sync for account: {}", account.account_name);
        
        let documents = self.list_documents(account, request.filters.clone()).await?;
        
        let mut documents_for_embedding = Vec::new();
        let mut errors = Vec::new();
        
        for doc in &documents {
            if doc.is_folder {
                continue;
            }
            
            // Skip large files (> 10MB)
            if let Some(size) = doc.size {
                if size > 10_485_760 {
                    continue;
                }
            }
            
            match doc.path.as_ref() {
                Some(path) => {
                    match self.get_document_content(account, path).await {
                        Ok(content) => {
                            let content_str = String::from_utf8_lossy(&content.content).to_string();
                            
                            // Only process text-based content
                            if matches!(content.content_type, ContentType::Text | ContentType::Code | ContentType::Markdown | ContentType::Html) {
                                let chunks = self.chunk_content(&content_str, &doc.name);
                                
                                documents_for_embedding.push(DocumentForEmbedding {
                                    id: Uuid::new_v4(),
                                    source_id: account.id,
                                    connector_type: ConnectorType::Dropbox,
                                    external_id: doc.external_id.clone(),
                                    name: doc.name.clone(),
                                    path: doc.path.clone(),
                                    content: content_str,
                                    content_type: content.content_type,
                                    metadata: serde_json::json!({
                                        "url": doc.url,
                                        "size": doc.size,
                                        "mime_type": doc.mime_type,
                                    }),
                                    chunks: Some(chunks),
                                });
                            }
                        }
                        Err(e) => {
                            error!("Failed to get content for {}: {}", doc.name, e);
                            errors.push(format!("Failed to get {}: {}", doc.name, e));
                        }
                    }
                }
                None => {
                    warn!("Document has no path: {}", doc.name);
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
        
        info!("✅ Dropbox sync completed: {:?}", result);
        
        Ok((result, documents_for_embedding))
    }
    
    async fn incremental_sync(
        &self,
        account: &ConnectedAccount,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<DocumentMetadata>, ConnectorError> {
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
        info!("📁 Disconnected from Dropbox");
        Ok(())
    }
    
    async fn refresh_credentials(&self, _account: &ConnectedAccount) -> Result<OAuthCredentials, ConnectorError> {
        Err(ConnectorError::UnsupportedOperation(
            "Dropbox credential refresh not implemented".to_string()
        ))
    }
}

pub struct DropboxConnectorFactory;

impl ConnectorFactory for DropboxConnectorFactory {
    fn create(&self) -> Box<dyn Connector> {
        Box::new(DropboxConnector::new())
    }
    
    fn connector_type(&self) -> ConnectorType {
        ConnectorType::Dropbox
    }
    
    fn supports_oauth(&self) -> bool {
        true
    }
    
    fn supports_webhooks(&self) -> bool {
        true
    }
}

#[async_trait]
impl WebhookConnector for DropboxConnector {
    async fn register_webhook(
        &self,
        _account: &ConnectedAccount,
        webhook_url: &str,
    ) -> Result<String, ConnectorError> {
        info!("Registering Dropbox webhook at: {}", webhook_url);
        Ok(Uuid::new_v4().to_string())
    }
    
    async fn handle_webhook(
        &self,
        _account: &ConnectedAccount,
        _webhook_id: &str,
        _payload: serde_json::Value,
    ) -> Result<Vec<DocumentMetadata>, ConnectorError> {
        Ok(Vec::new())
    }
    
    async fn unregister_webhook(
        &self,
        _account: &ConnectedAccount,
        _webhook_id: &str,
    ) -> Result<(), ConnectorError> {
        Ok(())
    }
}
