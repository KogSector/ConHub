use async_trait::async_trait;
use uuid::Uuid;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    RedirectUrl, Scope, TokenResponse, TokenUrl,
    basic::BasicClient,
    reqwest::async_http_client,
};

use super::traits::{Connector, ConnectorFactory, WebhookConnector};
use super::types::*;
use super::error::ConnectorError;

/// Google Drive connector
pub struct GoogleDriveConnector {
    name: String,
    client: Client,
    oauth_client: Option<BasicClient>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DriveFile {
    id: String,
    name: String,
    #[serde(rename = "mimeType")]
    mime_type: String,
    #[serde(rename = "createdTime")]
    created_time: Option<String>,
    #[serde(rename = "modifiedTime")]
    modified_time: Option<String>,
    size: Option<String>,
    #[serde(rename = "webViewLink")]
    web_view_link: Option<String>,
    parents: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DriveFileList {
    files: Vec<DriveFile>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

impl GoogleDriveConnector {
    pub fn new() -> Self {
        Self {
            name: "Google Drive".to_string(),
            client: Client::new(),
            oauth_client: None,
        }
    }
    
    pub fn factory() -> GoogleDriveConnectorFactory {
        GoogleDriveConnectorFactory
    }
    
    fn init_oauth_client(&mut self, config: &ConnectorConfig) -> Result<(), ConnectorError> {
        let client_id = config.credentials.get("client_id")
            .ok_or_else(|| ConnectorError::InvalidConfiguration(
                "Google Drive client_id is required".to_string()
            ))?;
        
        let client_secret = config.credentials.get("client_secret")
            .ok_or_else(|| ConnectorError::InvalidConfiguration(
                "Google Drive client_secret is required".to_string()
            ))?;
        
        let redirect_url = config.settings.get("redirect_url")
            .and_then(|v| v.as_str())
            .unwrap_or("http://localhost:3000/auth/google/callback");
        
        let client = BasicClient::new(
            ClientId::new(client_id.clone()),
            Some(ClientSecret::new(client_secret.clone())),
            AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
                .map_err(|e| ConnectorError::InvalidConfiguration(e.to_string()))?,
            Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
                .map_err(|e| ConnectorError::InvalidConfiguration(e.to_string()))?)
        )
        .set_redirect_uri(
            RedirectUrl::new(redirect_url.to_string())
                .map_err(|e| ConnectorError::InvalidConfiguration(e.to_string()))?
        );
        
        self.oauth_client = Some(client);
        Ok(())
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
    
    async fn list_files(
        &self,
        access_token: &str,
        query: Option<&str>,
        page_token: Option<&str>,
    ) -> Result<DriveFileList, ConnectorError> {
        let mut url = "https://www.googleapis.com/drive/v3/files?pageSize=100&fields=files(id,name,mimeType,createdTime,modifiedTime,size,webViewLink,parents),nextPageToken".to_string();
        
        if let Some(q) = query {
            url.push_str(&format!("&q={}", urlencoding::encode(q)));
        }
        
        if let Some(token) = page_token {
            url.push_str(&format!("&pageToken={}", token));
        }
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(ConnectorError::HttpError(
                format!("Google Drive API error: {}", response.status())
            ));
        }
        
        let file_list: DriveFileList = response.json().await?;
        Ok(file_list)
    }
    
    async fn get_file_content(
        &self,
        access_token: &str,
        file_id: &str,
    ) -> Result<Vec<u8>, ConnectorError> {
        let url = format!("https://www.googleapis.com/drive/v3/files/{}?alt=media", file_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(ConnectorError::HttpError(
                format!("Google Drive API error: {}", response.status())
            ));
        }
        
        let content = response.bytes().await?.to_vec();
        Ok(content)
    }
    
    async fn export_google_doc(
        &self,
        access_token: &str,
        file_id: &str,
        mime_type: &str,
    ) -> Result<Vec<u8>, ConnectorError> {
        // Export Google Docs as text/plain or text/html
        let export_mime = if mime_type == "application/vnd.google-apps.document" {
            "text/plain"
        } else if mime_type == "application/vnd.google-apps.spreadsheet" {
            "text/csv"
        } else if mime_type == "application/vnd.google-apps.presentation" {
            "text/plain"
        } else {
            return Err(ConnectorError::UnsupportedOperation(
                format!("Cannot export mime type: {}", mime_type)
            ));
        };
        
        let url = format!(
            "https://www.googleapis.com/drive/v3/files/{}/export?mimeType={}",
            file_id,
            urlencoding::encode(export_mime)
        );
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(ConnectorError::HttpError(
                format!("Google Drive API error: {}", response.status())
            ));
        }
        
        let content = response.bytes().await?.to_vec();
        Ok(content)
    }
    
    async fn recursively_list_files(
        &self,
        access_token: &str,
        folder_id: Option<&str>,
        documents: &mut Vec<DocumentMetadata>,
    ) -> Result<(), ConnectorError> {
        let query = match folder_id {
            Some(id) => format!("'{}' in parents and trashed=false", id),
            None => "trashed=false".to_string(),
        };
        
        let mut page_token: Option<String> = None;
        
        loop {
            let file_list = self.list_files(
                access_token,
                Some(&query),
                page_token.as_deref(),
            ).await?;
            
            for file in file_list.files {
                let is_folder = file.mime_type == "application/vnd.google-apps.folder";
                
                documents.push(DocumentMetadata {
                    external_id: file.id.clone(),
                    name: file.name.clone(),
                    path: None,
                    mime_type: Some(file.mime_type.clone()),
                    size: file.size.as_ref().and_then(|s| s.parse().ok()),
                    created_at: file.created_time.as_ref()
                        .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc)),
                    modified_at: file.modified_time.as_ref()
                        .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc)),
                    permissions: None,
                    url: file.web_view_link,
                    parent_id: file.parents.as_ref().and_then(|p| p.first().cloned()),
                    is_folder,
                    metadata: None,
                });
                
                // Recursively list folder contents
                if is_folder {
                    self.recursively_list_files(access_token, Some(&file.id), documents).await?;
                }
            }
            
            page_token = file_list.next_page_token;
            if page_token.is_none() {
                break;
            }
        }
        
        Ok(())
    }
    
    fn determine_content_type(&self, mime_type: &str) -> ContentType {
        match mime_type {
            "text/plain" => ContentType::Text,
            "text/markdown" => ContentType::Markdown,
            "text/html" => ContentType::Html,
            "application/pdf" => ContentType::Pdf,
            "application/vnd.google-apps.document" => ContentType::Text,
            "application/vnd.google-apps.spreadsheet" => ContentType::Text,
            "application/vnd.google-apps.presentation" => ContentType::Text,
            m if m.starts_with("text/") => ContentType::Text,
            m if m.starts_with("image/") => ContentType::Image,
            m if m.starts_with("video/") => ContentType::Video,
            m if m.starts_with("audio/") => ContentType::Audio,
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
impl Connector for GoogleDriveConnector {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn connector_type(&self) -> ConnectorType {
        ConnectorType::GoogleDrive
    }
    
    fn validate_config(&self, config: &ConnectorConfig) -> Result<(), ConnectorError> {
        if config.credentials.get("client_id").is_none() {
            return Err(ConnectorError::InvalidConfiguration(
                "Google Drive client_id is required".to_string()
            ));
        }
        
        if config.credentials.get("client_secret").is_none() {
            return Err(ConnectorError::InvalidConfiguration(
                "Google Drive client_secret is required".to_string()
            ));
        }
        
        Ok(())
    }
    
    async fn authenticate(&self, config: &ConnectorConfig) -> Result<Option<String>, ConnectorError> {
        let mut connector = Self::new();
        connector.init_oauth_client(config)?;
        
        let client = connector.oauth_client.as_ref()
            .ok_or_else(|| ConnectorError::AuthenticationFailed("OAuth client not initialized".to_string()))?;
        
        let (auth_url, _csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("https://www.googleapis.com/auth/drive.readonly".to_string()))
            .add_scope(Scope::new("https://www.googleapis.com/auth/userinfo.email".to_string()))
            .url();
        
        Ok(Some(auth_url.to_string()))
    }
    
    async fn complete_oauth(&self, callback_data: OAuthCallbackData) -> Result<OAuthCredentials, ConnectorError> {
        let client = self.oauth_client.as_ref()
            .ok_or_else(|| ConnectorError::AuthenticationFailed("OAuth client not initialized".to_string()))?;
        
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
            scope: None,
        })
    }
    
    async fn connect(&mut self, account: &ConnectedAccount) -> Result<(), ConnectorError> {
        info!("🔌 Connecting to Google Drive for account: {}", account.account_name);
        
        // Verify access token
        let access_token = self.get_access_token(account).await?;
        
        // Test connection by listing files (limit 1)
        self.list_files(&access_token, None, None).await?;
        
        info!("✅ Successfully connected to Google Drive");
        Ok(())
    }
    
    async fn check_connection(&self, account: &ConnectedAccount) -> Result<bool, ConnectorError> {
        let access_token = self.get_access_token(account).await?;
        self.list_files(&access_token, None, None).await.is_ok().into()
    }
    
    async fn list_documents(
        &self,
        account: &ConnectedAccount,
        _filters: Option<SyncFilters>,
    ) -> Result<Vec<DocumentMetadata>, ConnectorError> {
        let access_token = self.get_access_token(account).await?;
        
        let mut all_documents = Vec::new();
        
        info!("📁 Listing Google Drive files");
        
        self.recursively_list_files(&access_token, None, &mut all_documents).await?;
        
        info!("✅ Listed {} files from Google Drive", all_documents.len());
        
        Ok(all_documents)
    }
    
    async fn get_document_content(
        &self,
        account: &ConnectedAccount,
        document_id: &str,
    ) -> Result<DocumentContent, ConnectorError> {
        let access_token = self.get_access_token(account).await?;
        
        // First, get file metadata
        let file_list = self.list_files(
            &access_token,
            Some(&format!("id='{}'", document_id)),
            None,
        ).await?;
        
        let file = file_list.files.first()
            .ok_or_else(|| ConnectorError::DocumentNotFound(document_id.to_string()))?;
        
        // Get file content
        let content = if file.mime_type.starts_with("application/vnd.google-apps.") {
            // Export Google Workspace document
            self.export_google_doc(&access_token, document_id, &file.mime_type).await?
        } else {
            // Download regular file
            self.get_file_content(&access_token, document_id).await?
        };
        
        let content_type = self.determine_content_type(&file.mime_type);
        
        Ok(DocumentContent {
            metadata: DocumentMetadata {
                external_id: file.id.clone(),
                name: file.name.clone(),
                path: None,
                mime_type: Some(file.mime_type.clone()),
                size: file.size.as_ref().and_then(|s| s.parse().ok()),
                created_at: file.created_time.as_ref()
                    .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                modified_at: file.modified_time.as_ref()
                    .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                permissions: None,
                url: file.web_view_link.clone(),
                parent_id: file.parents.as_ref().and_then(|p| p.first().cloned()),
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
        
        info!("🔄 Starting Google Drive sync for account: {}", account.account_name);
        
        let documents = self.list_documents(account, request.filters.clone()).await?;
        
        let mut documents_for_embedding = Vec::new();
        let mut errors = Vec::new();
        
        // Process each document
        for doc in &documents {
            // Skip folders
            if doc.is_folder {
                continue;
            }
            
            // Skip binary files that we can't process
            if let Some(ref mime) = doc.mime_type {
                if mime.starts_with("image/") || mime.starts_with("video/") || mime.starts_with("audio/") {
                    continue;
                }
            }
            
            match self.get_document_content(account, &doc.external_id).await {
                Ok(content) => {
                    let content_str = String::from_utf8_lossy(&content.content).to_string();
                    let chunks = self.chunk_content(&content_str, &doc.name);
                    
                    documents_for_embedding.push(DocumentForEmbedding {
                        id: Uuid::new_v4(),
                        source_id: account.id,
                        connector_type: ConnectorType::GoogleDrive,
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
                Err(e) => {
                    error!("Failed to get content for {}: {}", doc.name, e);
                    errors.push(format!("Failed to get {}: {}", doc.name, e));
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
        
        info!("✅ Google Drive sync completed: {:?}", result);
        
        Ok((result, documents_for_embedding))
    }
    
    async fn incremental_sync(
        &self,
        account: &ConnectedAccount,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<DocumentMetadata>, ConnectorError> {
        let access_token = self.get_access_token(account).await?;
        
        // Query for files modified since the given date
        let query = format!(
            "modifiedTime > '{}' and trashed=false",
            since.to_rfc3339()
        );
        
        let mut all_documents = Vec::new();
        let mut page_token: Option<String> = None;
        
        loop {
            let file_list = self.list_files(
                &access_token,
                Some(&query),
                page_token.as_deref(),
            ).await?;
            
            for file in file_list.files {
                let is_folder = file.mime_type == "application/vnd.google-apps.folder";
                
                all_documents.push(DocumentMetadata {
                    external_id: file.id.clone(),
                    name: file.name.clone(),
                    path: None,
                    mime_type: Some(file.mime_type.clone()),
                    size: file.size.as_ref().and_then(|s| s.parse().ok()),
                    created_at: file.created_time.as_ref()
                        .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc)),
                    modified_at: file.modified_time.as_ref()
                        .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc)),
                    permissions: None,
                    url: file.web_view_link,
                    parent_id: file.parents.as_ref().and_then(|p| p.first().cloned()),
                    is_folder,
                    metadata: None,
                });
            }
            
            page_token = file_list.next_page_token;
            if page_token.is_none() {
                break;
            }
        }
        
        Ok(all_documents)
    }
    
    async fn disconnect(&mut self, _account: &ConnectedAccount) -> Result<(), ConnectorError> {
        info!("📁 Disconnected from Google Drive");
        Ok(())
    }
    
    async fn refresh_credentials(&self, account: &ConnectedAccount) -> Result<OAuthCredentials, ConnectorError> {
        let refresh_token = account.credentials
            .get("refresh_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConnectorError::AuthenticationFailed(
                "No refresh token found".to_string()
            ))?;
        
        let client = self.oauth_client.as_ref()
            .ok_or_else(|| ConnectorError::AuthenticationFailed("OAuth client not initialized".to_string()))?;
        
        let token_result = client
            .exchange_refresh_token(&oauth2::RefreshToken::new(refresh_token.to_string()))
            .request_async(async_http_client)
            .await
            .map_err(|e| ConnectorError::AuthenticationFailed(e.to_string()))?;
        
        Ok(OAuthCredentials {
            access_token: token_result.access_token().secret().clone(),
            refresh_token: Some(refresh_token.to_string()),
            token_type: "Bearer".to_string(),
            expires_in: token_result.expires_in().map(|d| d.as_secs() as i64),
            scope: None,
        })
    }
}

pub struct GoogleDriveConnectorFactory;

impl ConnectorFactory for GoogleDriveConnectorFactory {
    fn create(&self) -> Box<dyn Connector> {
        Box::new(GoogleDriveConnector::new())
    }
    
    fn connector_type(&self) -> ConnectorType {
        ConnectorType::GoogleDrive
    }
    
    fn supports_oauth(&self) -> bool {
        true
    }
    
    fn supports_webhooks(&self) -> bool {
        true
    }
}

#[async_trait]
impl WebhookConnector for GoogleDriveConnector {
    async fn register_webhook(
        &self,
        account: &ConnectedAccount,
        webhook_url: &str,
    ) -> Result<String, ConnectorError> {
        let access_token = self.get_access_token(account).await?;
        
        // TODO: Implement Google Drive Push Notifications (Webhooks)
        info!("Registering webhook at: {}", webhook_url);
        
        Ok(Uuid::new_v4().to_string())
    }
    
    async fn handle_webhook(
        &self,
        _account: &ConnectedAccount,
        _webhook_id: &str,
        _payload: serde_json::Value,
    ) -> Result<Vec<DocumentMetadata>, ConnectorError> {
        // TODO: Parse Google Drive webhook payload and return changed files
        Ok(Vec::new())
    }
    
    async fn unregister_webhook(
        &self,
        _account: &ConnectedAccount,
        _webhook_id: &str,
    ) -> Result<(), ConnectorError> {
        // TODO: Implement webhook removal
        Ok(())
    }
}
