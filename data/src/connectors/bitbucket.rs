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

/// Bitbucket API connector
pub struct BitbucketConnector {
    name: String,
    client: Client,
    oauth_client: Option<BasicClient>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BitbucketRepo {
    uuid: String,
    name: String,
    full_name: String,
    description: Option<String>,
    #[serde(rename = "is_private")]
    private: bool,
    links: BitbucketLinks,
}

#[derive(Debug, Serialize, Deserialize)]
struct BitbucketLinks {
    html: BitbucketLink,
    clone: Vec<BitbucketCloneLink>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BitbucketLink {
    href: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BitbucketCloneLink {
    name: String,
    href: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BitbucketRepoList {
    values: Vec<BitbucketRepo>,
    next: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BitbucketFile {
    path: String,
    #[serde(rename = "type")]
    file_type: String,
    size: Option<i64>,
    commit: Option<BitbucketCommit>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BitbucketCommit {
    hash: String,
    date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BitbucketFileList {
    values: Vec<BitbucketFile>,
    next: Option<String>,
}

impl BitbucketConnector {
    pub fn new() -> Self {
        Self {
            name: "Bitbucket".to_string(),
            client: Client::new(),
            oauth_client: None,
        }
    }
    
    pub fn factory() -> BitbucketConnectorFactory {
        BitbucketConnectorFactory
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
    
    async fn list_repositories(&self, access_token: &str) -> Result<Vec<BitbucketRepo>, ConnectorError> {
        let mut repos = Vec::new();
        let mut url = "https://api.bitbucket.org/2.0/repositories?role=member&pagelen=100".to_string();
        
        loop {
            let response = self.client
                .get(&url)
                .header("Authorization", format!("Bearer {}", access_token))
                .header("Accept", "application/json")
                .send()
                .await?;
            
            if !response.status().is_success() {
                return Err(ConnectorError::HttpError(
                    format!("Bitbucket API error: {}", response.status())
                ));
            }
            
            let repo_list: BitbucketRepoList = response.json().await?;
            repos.extend(repo_list.values);
            
            match repo_list.next {
                Some(next_url) => url = next_url,
                None => break,
            }
        }
        
        Ok(repos)
    }
    
    async fn list_repo_files(
        &self,
        access_token: &str,
        workspace: &str,
        repo_slug: &str,
        path: &str,
    ) -> Result<Vec<BitbucketFile>, ConnectorError> {
        let mut files = Vec::new();
        let mut url = format!(
            "https://api.bitbucket.org/2.0/repositories/{}/{}/src/HEAD/{}?pagelen=100",
            workspace, repo_slug, path
        );
        
        loop {
            let response = self.client
                .get(&url)
                .header("Authorization", format!("Bearer {}", access_token))
                .header("Accept", "application/json")
                .send()
                .await?;
            
            if !response.status().is_success() {
                if response.status().as_u16() == 404 {
                    // Empty repository or path doesn't exist
                    break;
                }
                return Err(ConnectorError::HttpError(
                    format!("Bitbucket API error: {}", response.status())
                ));
            }
            
            let file_list: BitbucketFileList = response.json().await?;
            files.extend(file_list.values);
            
            match file_list.next {
                Some(next_url) => url = next_url,
                None => break,
            }
        }
        
        Ok(files)
    }
    
    async fn get_file_content(
        &self,
        access_token: &str,
        workspace: &str,
        repo_slug: &str,
        path: &str,
    ) -> Result<Vec<u8>, ConnectorError> {
        let url = format!(
            "https://api.bitbucket.org/2.0/repositories/{}/{}/src/HEAD/{}",
            workspace, repo_slug, path
        );
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(ConnectorError::HttpError(
                format!("Bitbucket API error: {}", response.status())
            ));
        }
        
        let content = response.bytes().await?.to_vec();
        Ok(content)
    }
    
    fn recursively_list_files<'a>(
        &'a self,
        access_token: &'a str,
        workspace: &'a str,
        repo_slug: &'a str,
        path: &'a str,
        documents: &'a mut Vec<DocumentMetadata>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ConnectorError>> + Send + 'a>> {
        Box::pin(async move {
            let files = self.list_repo_files(access_token, workspace, repo_slug, path).await?;
            
            for file in files {
                if file.file_type == "commit_directory" {
                    // Recursively list directory
                    self.recursively_list_files(
                        access_token,
                        workspace,
                        repo_slug,
                        &file.path,
                        documents,
                    ).await?;
                } else if file.file_type == "commit_file" {
                    documents.push(DocumentMetadata {
                        external_id: file.commit.as_ref()
                            .map(|c| c.hash.clone())
                            .unwrap_or_else(|| file.path.clone()),
                        name: file.path.split('/').last().unwrap_or(&file.path).to_string(),
                        path: Some(file.path.clone()),
                        mime_type: mime_guess::from_path(&file.path).first().map(|m| m.to_string()),
                        size: file.size,
                        created_at: None,
                        modified_at: file.commit.as_ref()
                            .and_then(|c| c.date.as_ref())
                            .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
                            .map(|dt| dt.with_timezone(&chrono::Utc)),
                        permissions: None,
                        url: Some(format!(
                            "https://bitbucket.org/{}/{}/src/HEAD/{}",
                            workspace, repo_slug, file.path
                        )),
                        parent_id: Some(path.to_string()),
                        is_folder: false,
                        metadata: Some(serde_json::json!({
                            "commit_hash": file.commit.as_ref().map(|c| &c.hash),
                        })),
                    });
                }
            }
            
            Ok(())
        })
    }
    
    fn chunk_content(&self, content: &str, file_path: &str) -> Vec<DocumentChunk> {
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
                    "file_path": file_path,
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
impl Connector for BitbucketConnector {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn connector_type(&self) -> ConnectorType {
        ConnectorType::Bitbucket
    }
    
    fn validate_config(&self, config: &ConnectorConfigAuth) -> Result<(), ConnectorError> {
        if config.credentials.get("client_id").is_none() {
            return Err(ConnectorError::InvalidConfiguration(
                "Bitbucket client_id is required".to_string()
            ));
        }
        
        if config.credentials.get("client_secret").is_none() {
            return Err(ConnectorError::InvalidConfiguration(
                "Bitbucket client_secret is required".to_string()
            ));
        }
        
        Ok(())
    }
    
    async fn authenticate(&self, config: &ConnectorConfigAuth) -> Result<Option<String>, ConnectorError> {
        let client_id = config.credentials.get("client_id")
            .cloned()
            .or_else(|| env::var("BITBUCKET_CLIENT_ID").ok())
            .ok_or_else(|| ConnectorError::InvalidConfiguration("Bitbucket client_id is required".to_string()))?;
        
        let client_secret = config.credentials.get("client_secret")
            .cloned()
            .or_else(|| env::var("BITBUCKET_CLIENT_SECRET").ok())
            .ok_or_else(|| ConnectorError::InvalidConfiguration("Bitbucket client_secret is required".to_string()))?;
        
        let redirect_url = config.settings.get("redirect_url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| env::var("BITBUCKET_REDIRECT_URL").ok())
            .unwrap_or_else(|| "http://localhost:3000/auth/bitbucket/callback".to_string());
        
        let client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new("https://bitbucket.org/site/oauth2/authorize".to_string())
                .map_err(|e| ConnectorError::InvalidConfiguration(e.to_string()))?,
            Some(TokenUrl::new("https://bitbucket.org/site/oauth2/access_token".to_string())
                .map_err(|e| ConnectorError::InvalidConfiguration(e.to_string()))?)
        )
        .set_redirect_uri(
            RedirectUrl::new(redirect_url)
                .map_err(|e| ConnectorError::InvalidConfiguration(e.to_string()))?
        );
        
        let (auth_url, _csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("repositories".to_string()))
            .url();
        
        Ok(Some(auth_url.to_string()))
    }
    
    async fn complete_oauth(&self, callback_data: OAuthCallbackData) -> Result<OAuthCredentials, ConnectorError> {
        let client_id = env::var("BITBUCKET_CLIENT_ID")
            .map_err(|_| ConnectorError::InvalidConfiguration("BITBUCKET_CLIENT_ID not set".to_string()))?;
        let client_secret = env::var("BITBUCKET_CLIENT_SECRET")
            .map_err(|_| ConnectorError::InvalidConfiguration("BITBUCKET_CLIENT_SECRET not set".to_string()))?;
        let redirect_url = env::var("BITBUCKET_REDIRECT_URL")
            .unwrap_or_else(|_| "http://localhost:3000/auth/bitbucket/callback".to_string());

        let client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new("https://bitbucket.org/site/oauth2/authorize".to_string())
                .map_err(|e| ConnectorError::InvalidConfiguration(e.to_string()))?,
            Some(TokenUrl::new("https://bitbucket.org/site/oauth2/access_token".to_string())
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
        info!("🔌 Connecting to Bitbucket for account: {}", account.account_name);
        
        let access_token = self.get_access_token(account).await?;
        
        // Test connection by fetching user repos
        self.list_repositories(&access_token).await?;
        
        info!("✅ Successfully connected to Bitbucket");
        Ok(())
    }
    
    async fn check_connection(&self, account: &ConnectedAccount) -> Result<bool, ConnectorError> {
        let access_token = self.get_access_token(account).await?;
        Ok(self.list_repositories(&access_token).await.is_ok())
    }
    
    async fn list_documents(
        &self,
        account: &ConnectedAccount,
        filters: Option<SyncFilters>,
    ) -> Result<Vec<DocumentMetadata>, ConnectorError> {
        let access_token = self.get_access_token(account).await?;
        
        let repo_filter = filters.as_ref()
            .and_then(|f| f.include_paths.as_ref())
            .map(|paths| paths.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        
        let repos = self.list_repositories(&access_token).await?;
        
        let mut all_documents = Vec::new();
        
        for repo in repos {
            if let Some(ref filter) = repo_filter {
                if !filter.iter().any(|f| repo.full_name.contains(f)) {
                    continue;
                }
            }
            
            info!("📦 Listing files in repository: {}", repo.full_name);
            
            let parts: Vec<&str> = repo.full_name.split('/').collect();
            if parts.len() != 2 {
                warn!("Invalid repository name: {}", repo.full_name);
                continue;
            }
            
            let (workspace, repo_slug) = (parts[0], parts[1]);
            
            match self.recursively_list_files(
                &access_token,
                workspace,
                repo_slug,
                "",
                &mut all_documents,
            ).await {
                Ok(_) => info!("✅ Listed files in {}", repo.full_name),
                Err(e) => error!("Failed to list files in {}: {}", repo.full_name, e),
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
        
        // document_id should be in format: workspace/repo_slug/path
        let parts: Vec<&str> = document_id.splitn(3, '/').collect();
        if parts.len() < 3 {
            return Err(ConnectorError::InvalidConfiguration(
                "Invalid document ID format".to_string()
            ));
        }
        
        let (workspace, repo_slug, path) = (parts[0], parts[1], parts[2]);
        
        let content = self.get_file_content(&access_token, workspace, repo_slug, path).await?;
        
        Ok(DocumentContent {
            metadata: DocumentMetadata {
                external_id: document_id.to_string(),
                name: path.split('/').last().unwrap_or(path).to_string(),
                path: Some(path.to_string()),
                mime_type: mime_guess::from_path(path).first().map(|m| m.to_string()),
                size: Some(content.len() as i64),
                created_at: None,
                modified_at: None,
                permissions: None,
                url: Some(format!("https://bitbucket.org/{}/{}/src/HEAD/{}", workspace, repo_slug, path)),
                parent_id: None,
                is_folder: false,
                metadata: None,
            },
            content,
            content_type: ContentType::Code,
        })
    }
    
    async fn sync(
        &self,
        account: &ConnectedAccount,
        request: &SyncRequestWithFilters,
    ) -> Result<(SyncResult, Vec<DocumentForEmbedding>), ConnectorError> {
        let start_time = std::time::Instant::now();
        
        info!("🔄 Starting Bitbucket sync for account: {}", account.account_name);
        
        let documents = self.list_documents(account, request.filters.clone()).await?;
        
        let mut documents_for_embedding = Vec::new();
        let mut errors = Vec::new();
        
        for doc in &documents {
            if doc.is_folder {
                continue;
            }
            
            // Skip binary files
            if let Some(ref mime) = doc.mime_type {
                if mime.starts_with("image/") || mime.starts_with("video/") || mime.starts_with("audio/") {
                    continue;
                }
            }
            
            match doc.path.as_ref() {
                Some(path) => {
                    match self.get_document_content(account, path).await {
                        Ok(content) => {
                            let content_str = String::from_utf8_lossy(&content.content).to_string();
                            let chunks = self.chunk_content(&content_str, path);
                            
                            documents_for_embedding.push(DocumentForEmbedding {
                                id: Uuid::new_v4(),
                                source_id: account.id,
                                connector_type: ConnectorType::Bitbucket,
                                external_id: doc.external_id.clone(),
                                name: doc.name.clone(),
                                path: doc.path.clone(),
                                content: content_str,
                                content_type: ContentType::Code,
                                metadata: serde_json::json!({
                                    "url": doc.url,
                                    "size": doc.size,
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
        
        info!("✅ Bitbucket sync completed: {:?}", result);
        
        Ok((result, documents_for_embedding))
    }
    
    async fn incremental_sync(
        &self,
        _account: &ConnectedAccount,
        _since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<DocumentMetadata>, ConnectorError> {
        warn!("Incremental sync not implemented for Bitbucket");
        Ok(Vec::new())
    }
    
    async fn disconnect(&mut self, _account: &ConnectedAccount) -> Result<(), ConnectorError> {
        info!("📁 Disconnected from Bitbucket");
        Ok(())
    }
    
    async fn refresh_credentials(&self, _account: &ConnectedAccount) -> Result<OAuthCredentials, ConnectorError> {
        Err(ConnectorError::UnsupportedOperation(
            "Bitbucket credential refresh not implemented".to_string()
        ))
    }
}

pub struct BitbucketConnectorFactory;

impl ConnectorFactory for BitbucketConnectorFactory {
    fn create(&self) -> Box<dyn Connector> {
        Box::new(BitbucketConnector::new())
    }
    
    fn connector_type(&self) -> ConnectorType {
        ConnectorType::Bitbucket
    }
    
    fn supports_oauth(&self) -> bool {
        true
    }
    
    fn supports_webhooks(&self) -> bool {
        true
    }
}

#[async_trait]
impl WebhookConnector for BitbucketConnector {
    async fn register_webhook(
        &self,
        _account: &ConnectedAccount,
        webhook_url: &str,
    ) -> Result<String, ConnectorError> {
        info!("Registering Bitbucket webhook at: {}", webhook_url);
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
