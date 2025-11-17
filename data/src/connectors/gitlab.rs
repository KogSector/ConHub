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

/// GitLab API connector
pub struct GitLabConnector {
    name: String,
    client: Client,
    oauth_client: Option<BasicClient>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitLabProject {
    id: i64,
    name: String,
    name_with_namespace: String,
    description: Option<String>,
    web_url: String,
    http_url_to_repo: String,
    ssh_url_to_repo: String,
    default_branch: Option<String>,
    visibility: String,
    permissions: Option<GitLabProjectPermissions>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitLabProjectPermissions {
    project_access: Option<GitLabAccessLevel>,
    group_access: Option<GitLabAccessLevel>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitLabAccessLevel {
    access_level: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitLabFile {
    id: String,
    name: String,
    #[serde(rename = "type")]
    file_type: String,
    path: String,
    mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitLabFileContent {
    file_name: String,
    file_path: String,
    size: i64,
    encoding: String,
    content: String,
    content_sha256: String,
    ref_name: String,
    blob_id: String,
    commit_id: String,
}

impl GitLabConnector {
    pub fn new() -> Self {
        Self {
            name: "GitLab".to_string(),
            client: Client::new(),
            oauth_client: None,
        }
    }

    fn auth_header(token: &str) -> String {
        format!("Bearer {}", token)
    }
    
    pub fn factory() -> GitLabConnectorFactory {
        GitLabConnectorFactory
    }
    
    fn init_oauth_client(&mut self, config: &ConnectorConfigAuth) -> Result<(), ConnectorError> {
        let client_id = config.credentials.get("client_id")
            .ok_or_else(|| ConnectorError::InvalidConfiguration(
                "GitLab client_id is required".to_string()
            ))?;
        
        let client_secret = config.credentials.get("client_secret")
            .ok_or_else(|| ConnectorError::InvalidConfiguration(
                "GitLab client_secret is required".to_string()
            ))?;
        
        let redirect_url = config.settings.get("redirect_url")
            .and_then(|v| v.as_str())
            .unwrap_or("http://localhost:3000/auth/gitlab/callback");
        
        let gitlab_url = config.settings.get("gitlab_url")
            .and_then(|v| v.as_str())
            .unwrap_or("https://gitlab.com");
        
        let client = BasicClient::new(
            ClientId::new(client_id.clone()),
            Some(ClientSecret::new(client_secret.clone())),
            AuthUrl::new(format!("{}/oauth/authorize", gitlab_url))
                .map_err(|e| ConnectorError::InvalidConfiguration(e.to_string()))?,
            Some(TokenUrl::new(format!("{}/oauth/token", gitlab_url))
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
    
    fn get_gitlab_url(&self, account: &ConnectedAccount) -> String {
        account.settings
            .get("gitlab_url")
            .and_then(|v| v.as_str())
            .unwrap_or("https://gitlab.com")
            .to_string()
    }
    
    /// Validate user has read access to a project
    async fn validate_read_access(&self, access_token: &str, project_id: i64, gitlab_url: &str) -> Result<bool, ConnectorError> {
        let url = format!("{}/api/v4/projects/{}", gitlab_url, project_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", Self::auth_header(access_token))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Ok(false);
        }
        
        let project: GitLabProject = response.json().await?;
        
        // Check if user has at least reporter (read) access
        // Access levels: 10=Guest, 20=Reporter, 30=Developer, 40=Maintainer, 50=Owner
        let has_access = if let Some(permissions) = project.permissions {
            let project_access = permissions.project_access
                .map(|a| a.access_level >= 20)
                .unwrap_or(false);
            let group_access = permissions.group_access
                .map(|a| a.access_level >= 20)
                .unwrap_or(false);
            
            project_access || group_access || project.visibility == "public"
        } else {
            project.visibility == "public"
        };
        
        Ok(has_access)
    }
    
    async fn list_projects(&self, access_token: &str, gitlab_url: &str) -> Result<Vec<GitLabProject>, ConnectorError> {
        let url = format!("{}/api/v4/projects?membership=true&per_page=100", gitlab_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", Self::auth_header(access_token))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(ConnectorError::HttpError(
                format!("GitLab API error: {}", response.status())
            ));
        }
        
        let projects: Vec<GitLabProject> = response.json().await?;
        Ok(projects)
    }
    
    async fn list_project_files(
        &self,
        access_token: &str,
        gitlab_url: &str,
        project_id: i64,
        path: &str,
        ref_name: &str,
    ) -> Result<Vec<GitLabFile>, ConnectorError> {
        let encoded_path = urlencoding::encode(path);
        let url = format!(
            "{}/api/v4/projects/{}/repository/tree?path={}&ref={}&per_page=100",
            gitlab_url, project_id, encoded_path, ref_name
        );
        
        let response = self.client
            .get(&url)
            .header("Authorization", Self::auth_header(access_token))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(ConnectorError::HttpError(
                format!("GitLab API error: {}", response.status())
            ));
        }
        
        let files: Vec<GitLabFile> = response.json().await?;
        Ok(files)
    }
    
    async fn get_file_content(
        &self,
        access_token: &str,
        gitlab_url: &str,
        project_id: i64,
        file_path: &str,
        ref_name: &str,
    ) -> Result<Vec<u8>, ConnectorError> {
        let encoded_path = urlencoding::encode(file_path);
        let url = format!(
            "{}/api/v4/projects/{}/repository/files/{}?ref={}",
            gitlab_url, project_id, encoded_path, ref_name
        );
        
        let response = self.client
            .get(&url)
            .header("Authorization", Self::auth_header(access_token))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(ConnectorError::HttpError(
                format!("GitLab API error: {}", response.status())
            ));
        }
        
        let file: GitLabFileContent = response.json().await?;
        
        // Decode base64 content
        let decoded = base64::decode(&file.content.replace("\n", ""))
            .map_err(|e| ConnectorError::SerializationError(e.to_string()))?;
        
        Ok(decoded)
    }
}

#[async_trait]
impl Connector for GitLabConnector {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn connector_type(&self) -> ConnectorType {
        ConnectorType::GitLab
    }
    
    async fn init(&mut self, config: ConnectorConfigAuth) -> Result<(), ConnectorError> {
        self.init_oauth_client(&config)?;
        info!("GitLab connector initialized");
        Ok(())
    }
    
    async fn connect(&self, account: &ConnectedAccount) -> Result<(), ConnectorError> {
        let access_token = self.get_access_token(account).await?;
        let gitlab_url = self.get_gitlab_url(account);
        
        // Test connection by fetching user's projects
        self.list_projects(&access_token, &gitlab_url).await?;
        
        info!("Successfully connected to GitLab");
        Ok(())
    }
    
    async fn disconnect(&self, _account: &ConnectedAccount) -> Result<(), ConnectorError> {
        info!("Disconnected from GitLab");
        Ok(())
    }
    
    async fn fetch_data(&self, account: &ConnectedAccount, source: &DataSource) -> Result<Vec<Document>, ConnectorError> {
        let access_token = self.get_access_token(account).await?;
        let gitlab_url = self.get_gitlab_url(account);
        
        let project_id = source.external_id.parse::<i64>()
            .map_err(|_| ConnectorError::InvalidConfiguration("Invalid project ID".to_string()))?;
        
        // Validate read access before fetching
        if !self.validate_read_access(&access_token, project_id, &gitlab_url).await? {
            return Err(ConnectorError::AuthenticationFailed(
                "User does not have read access to this project".to_string()
            ));
        }
        
        let ref_name = source.settings.get("branch")
            .and_then(|v| v.as_str())
            .unwrap_or("main");
        
        let mut documents = Vec::new();
        
        // Fetch files from project
        let files = self.list_project_files(&access_token, &gitlab_url, project_id, "", ref_name).await?;
        
        for file in files {
            if file.file_type == "blob" {
                // Skip non-text files
                if !self.is_text_file(&file.path) {
                    continue;
                }
                
                match self.get_file_content(&access_token, &gitlab_url, project_id, &file.path, ref_name).await {
                    Ok(content) => {
                        if let Ok(text_content) = String::from_utf8(content) {
                            let mut metadata = HashMap::new();
                            metadata.insert("project_id".to_string(), serde_json::json!(project_id));
                            metadata.insert("file_path".to_string(), serde_json::json!(file.path));
                            metadata.insert("branch".to_string(), serde_json::json!(ref_name));
                            
                            documents.push(Document {
                                id: Uuid::new_v4(),
                                source_id: source.id,
                                external_id: file.id.clone(),
                                name: file.name.clone(),
                                content: text_content,
                                content_type: "text/plain".to_string(),
                                path: file.path.clone(),
                                size: 0,
                                metadata,
                                created_at: chrono::Utc::now(),
                                updated_at: chrono::Utc::now(),
                            });
                        }
                    }
                    Err(e) => {
                        warn!("Failed to fetch file {}: {}", file.path, e);
                    }
                }
            }
        }
        
        info!("Fetched {} documents from GitLab project {}", documents.len(), project_id);
        Ok(documents)
    }
    
    async fn validate_credentials(&self, credentials: &HashMap<String, String>) -> Result<bool, ConnectorError> {
        let access_token = credentials.get("access_token")
            .ok_or_else(|| ConnectorError::AuthenticationFailed("No access token provided".to_string()))?;
        
        let gitlab_url = credentials.get("gitlab_url").map(|s| s.as_str()).unwrap_or("https://gitlab.com");
        
        let url = format!("{}/api/v4/user", gitlab_url);
        let response = self.client
            .get(&url)
            .header("Authorization", Self::auth_header(access_token))
            .send()
            .await?;
        
        Ok(response.status().is_success())
    }
    
    async fn get_oauth_url(&self, state: &str) -> Result<String, ConnectorError> {
        let client = self.oauth_client.as_ref()
            .ok_or_else(|| ConnectorError::InvalidConfiguration("OAuth client not initialized".to_string()))?;
        
        let (auth_url, _csrf_token) = client
            .authorize_url(|| CsrfToken::new(state.to_string()))
            .add_scope(Scope::new("read_user".to_string()))
            .add_scope(Scope::new("read_api".to_string()))
            .add_scope(Scope::new("read_repository".to_string()))
            .url();
        
        Ok(auth_url.to_string())
    }
    
    async fn exchange_code(&self, code: &str) -> Result<HashMap<String, String>, ConnectorError> {
        let client = self.oauth_client.as_ref()
            .ok_or_else(|| ConnectorError::InvalidConfiguration("OAuth client not initialized".to_string()))?;
        
        let token_result = client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(async_http_client)
            .await
            .map_err(|e| ConnectorError::AuthenticationFailed(format!("Token exchange failed: {}", e)))?;
        
        let mut credentials = HashMap::new();
        credentials.insert("access_token".to_string(), token_result.access_token().secret().clone());
        
        if let Some(refresh_token) = token_result.refresh_token() {
            credentials.insert("refresh_token".to_string(), refresh_token.secret().clone());
        }
        
        Ok(credentials)
    }
}

impl GitLabConnector {
    fn is_text_file(&self, path: &str) -> bool {
        let text_extensions = vec![
            "rs", "go", "py", "js", "ts", "jsx", "tsx", "java", "c", "cpp", "h", "hpp",
            "cs", "php", "rb", "swift", "kt", "scala", "r", "m", "mm",
            "txt", "md", "json", "yaml", "yml", "toml", "xml", "html", "css", "scss",
            "sh", "bash", "zsh", "fish", "sql", "prisma", "graphql", "proto"
        ];
        
        if let Some(ext) = path.split('.').last() {
            text_extensions.contains(&ext.to_lowercase().as_str())
        } else {
            false
        }
    }
}

pub struct GitLabConnectorFactory;

#[async_trait]
impl ConnectorFactory for GitLabConnectorFactory {
    fn connector_type(&self) -> ConnectorType {
        ConnectorType::GitLab
    }
    
    async fn create(&self, config: ConnectorConfigAuth) -> Result<Box<dyn Connector>, ConnectorError> {
        let mut connector = GitLabConnector::new();
        connector.init(config).await?;
        Ok(Box::new(connector))
    }
}

#[async_trait]
impl WebhookConnector for GitLabConnector {
    async fn setup_webhook(&self, _account: &ConnectedAccount, _source: &DataSource, _webhook_url: &str) -> Result<String, ConnectorError> {
        // GitLab webhook implementation would go here
        Err(ConnectorError::NotImplemented("GitLab webhooks not yet implemented".to_string()))
    }
    
    async fn remove_webhook(&self, _account: &ConnectedAccount, _webhook_id: &str) -> Result<(), ConnectorError> {
        Ok(())
    }
    
    async fn handle_webhook(&self, _payload: &[u8], _signature: Option<&str>) -> Result<Vec<Document>, ConnectorError> {
        Err(ConnectorError::NotImplemented("GitLab webhooks not yet implemented".to_string()))
    }
}
