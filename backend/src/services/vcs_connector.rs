use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use chrono::Utc;

use crate::models::{
    VcsType, VcsProvider, RepositoryInfo, RepositoryCredentials, 
    CredentialType, RepositoryConfig, RepositorySyncStatus
};
use crate::services::vcs_detector::{VcsDetector, CloneUrls};

/// Result type for VCS operations
pub type VcsResult<T> = Result<T, VcsError>;

/// VCS operation errors
#[derive(Debug, thiserror::Error)]
pub enum VcsError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),
    
    #[error("Unsupported VCS type: {0:?}")]
    UnsupportedVcs(VcsType),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Invalid repository URL: {0}")]
    InvalidUrl(String),
    
    #[error("VCS operation failed: {0}")]
    OperationFailed(String),
}

/// Repository metadata retrieved from VCS
#[derive(Debug, Clone)]
pub struct RepositoryMetadata {
    pub name: String,
    pub description: Option<String>,
    pub is_private: bool,
    pub default_branch: String,
    pub branches: Vec<String>,
    pub tags: Vec<String>,
    pub language: Option<String>,
    pub size_kb: Option<u64>,
    pub stars: Option<u32>,
    pub forks: Option<u32>,
    pub clone_urls: CloneUrls,
}

/// File content from repository
#[derive(Debug, Clone)]
pub struct FileContent {
    pub path: String,
    pub content: String,
    pub sha: String,
    pub size: u64,
    pub url: String,
}

/// Repository branch information
#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub sha: String,
    pub protected: bool,
    pub is_default: bool,
}

/// VCS connector trait for different version control systems
#[async_trait]
pub trait VcsConnector: Send + Sync {
    /// Test connection with the provided credentials
    async fn test_connection(&self, credentials: &RepositoryCredentials) -> VcsResult<bool>;
    
    /// Get repository metadata
    async fn get_repository_metadata(
        &self, 
        url: &str, 
        credentials: &RepositoryCredentials
    ) -> VcsResult<RepositoryMetadata>;
    
    /// List branches in the repository
    async fn list_branches(
        &self, 
        url: &str, 
        credentials: &RepositoryCredentials
    ) -> VcsResult<Vec<BranchInfo>>;
    
    /// Get file contents from repository
    async fn get_file_content(
        &self,
        url: &str,
        path: &str,
        branch: &str,
        credentials: &RepositoryCredentials,
    ) -> VcsResult<FileContent>;
    
    /// List files in repository directory
    async fn list_files(
        &self,
        url: &str,
        path: &str,
        branch: &str,
        credentials: &RepositoryCredentials,
        recursive: bool,
    ) -> VcsResult<Vec<String>>;
    
    /// Clone or pull repository for local access
    async fn sync_repository(
        &self,
        repo_info: &RepositoryInfo,
        local_path: &str,
    ) -> VcsResult<()>;
    
    /// Setup webhook for repository (if supported)
    async fn setup_webhook(
        &self,
        url: &str,
        webhook_url: &str,
        secret: &str,
        credentials: &RepositoryCredentials,
    ) -> VcsResult<String>;
}

/// Git connector implementation
pub struct GitConnector {
    client: Client,
}

impl GitConnector {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
    
    /// Get GitHub API base URL
    fn get_api_base_url(&self, provider: &VcsProvider, url: &str) -> VcsResult<String> {
        match provider {
            VcsProvider::GitHub => Ok("https://api.github.com".to_string()),
            VcsProvider::GitLab => Ok("https://gitlab.com/api/v4".to_string()),
            VcsProvider::Bitbucket => Ok("https://api.bitbucket.org/2.0".to_string()),
            VcsProvider::Azure => Ok("https://dev.azure.com".to_string()),
            VcsProvider::SelfHosted => {
                // Try to construct API URL for self-hosted instances
                if url.contains("gitlab") {
                    let base = url.split("/").take(3).collect::<Vec<_>>().join("/");
                    Ok(format!("{}/api/v4", base))
                } else {
                    // Generic Git API (might not be available)
                    Err(VcsError::UnsupportedVcs(VcsType::Git))
                }
            }
            _ => Err(VcsError::UnsupportedVcs(VcsType::Git)),
        }
    }
    
    /// Make authenticated API request
    async fn make_api_request(
        &self,
        url: &str,
        credentials: &RepositoryCredentials,
    ) -> VcsResult<Value> {
        let mut request = self.client.get(url);
        
        // Add authentication headers based on credential type
        match &credentials.credential_type {
            CredentialType::PersonalAccessToken { token } => {
                request = request.header("Authorization", format!("token {}", token));
            }
            CredentialType::OAuthToken { access_token, .. } => {
                request = request.header("Authorization", format!("Bearer {}", access_token));
            }
            CredentialType::UsernamePassword { username, password } => {
                request = request.basic_auth(username, Some(password));
            }
            CredentialType::AppPassword { username, app_password } => {
                request = request.basic_auth(username, Some(app_password));
            }
            _ => {}
        }
        
        let response = request.send().await
            .map_err(|e| VcsError::NetworkError(e.to_string()))?;
        
        if response.status().is_success() {
            let json = response.json::<Value>().await
                .map_err(|e| VcsError::OperationFailed(e.to_string()))?;
            Ok(json)
        } else {
            match response.status().as_u16() {
                401 => Err(VcsError::AuthenticationFailed("Invalid credentials".to_string())),
                403 => Err(VcsError::PermissionDenied("Access denied".to_string())),
                404 => Err(VcsError::RepositoryNotFound("Repository not found".to_string())),
                _ => Err(VcsError::OperationFailed(format!("HTTP {}", response.status()))),
            }
        }
    }
}

#[async_trait]
impl VcsConnector for GitConnector {
    async fn test_connection(&self, credentials: &RepositoryCredentials) -> VcsResult<bool> {
        // Test with GitHub user endpoint
        match &credentials.credential_type {
            CredentialType::PersonalAccessToken { token } => {
                let response = self.client
                    .get("https://api.github.com/user")
                    .header("Authorization", format!("token {}", token))
                    .header("User-Agent", "ConHub")
                    .send()
                    .await
                    .map_err(|e| VcsError::NetworkError(e.to_string()))?;
                
                Ok(response.status().is_success())
            }
            CredentialType::OAuthToken { access_token, .. } => {
                let response = self.client
                    .get("https://api.github.com/user")
                    .header("Authorization", format!("Bearer {}", access_token))
                    .header("User-Agent", "ConHub")
                    .send()
                    .await
                    .map_err(|e| VcsError::NetworkError(e.to_string()))?;
                
                Ok(response.status().is_success())
            }
            _ => Ok(false), // Other credential types not supported for testing
        }
    }
    
    async fn get_repository_metadata(
        &self,
        url: &str,
        credentials: &RepositoryCredentials,
    ) -> VcsResult<RepositoryMetadata> {
        let (vcs_type, provider) = VcsDetector::detect_from_url(url)
            .map_err(|e| VcsError::InvalidUrl(e))?;
        
        let (owner, repo) = VcsDetector::extract_repo_info(url)
            .map_err(|e| VcsError::InvalidUrl(e))?;
        
        let api_base = self.get_api_base_url(&provider, url)?;
        
        let api_url = match provider {
            VcsProvider::GitHub => format!("{}/repos/{}/{}", api_base, owner, repo),
            VcsProvider::GitLab => format!("{}/projects/{}%2F{}", api_base, owner, repo),
            VcsProvider::Bitbucket => format!("{}/repositories/{}/{}", api_base, owner, repo),
            _ => return Err(VcsError::UnsupportedVcs(vcs_type)),
        };
        
        let repo_data = self.make_api_request(&api_url, credentials).await?;
        
        // Parse response based on provider
        let metadata = match provider {
            VcsProvider::GitHub => {
                RepositoryMetadata {
                    name: repo_data["name"].as_str().unwrap_or(&repo).to_string(),
                    description: repo_data["description"].as_str().map(|s| s.to_string()),
                    is_private: repo_data["private"].as_bool().unwrap_or(false),
                    default_branch: repo_data["default_branch"].as_str().unwrap_or("main").to_string(),
                    branches: vec![], // Will be populated separately
                    tags: vec![],     // Will be populated separately
                    language: repo_data["language"].as_str().map(|s| s.to_string()),
                    size_kb: repo_data["size"].as_u64(),
                    stars: repo_data["stargazers_count"].as_u64().map(|n| n as u32),
                    forks: repo_data["forks_count"].as_u64().map(|n| n as u32),
                    clone_urls: VcsDetector::generate_clone_urls(url, &provider)
                        .map_err(|e| VcsError::InvalidUrl(e))?,
                }
            }
            _ => return Err(VcsError::UnsupportedVcs(vcs_type)),
        };
        
        Ok(metadata)
    }
    
    async fn list_branches(
        &self,
        url: &str,
        credentials: &RepositoryCredentials,
    ) -> VcsResult<Vec<BranchInfo>> {
        let (vcs_type, provider) = VcsDetector::detect_from_url(url)
            .map_err(|e| VcsError::InvalidUrl(e))?;
        
        let (owner, repo) = VcsDetector::extract_repo_info(url)
            .map_err(|e| VcsError::InvalidUrl(e))?;
        
        let api_base = self.get_api_base_url(&provider, url)?;
        
        let api_url = match provider {
            VcsProvider::GitHub => format!("{}/repos/{}/{}/branches", api_base, owner, repo),
            VcsProvider::GitLab => format!("{}/projects/{}%2F{}/repository/branches", api_base, owner, repo),
            VcsProvider::Bitbucket => format!("{}/repositories/{}/{}/refs/branches", api_base, owner, repo),
            _ => return Err(VcsError::UnsupportedVcs(vcs_type)),
        };
        
        let branches_data = self.make_api_request(&api_url, credentials).await?;
        
        let mut branches = Vec::new();
        
        match provider {
            VcsProvider::GitHub => {
                if let Some(branch_array) = branches_data.as_array() {
                    for branch in branch_array {
                        if let Some(name) = branch["name"].as_str() {
                            branches.push(BranchInfo {
                                name: name.to_string(),
                                sha: branch["commit"]["sha"].as_str().unwrap_or("").to_string(),
                                protected: branch["protected"].as_bool().unwrap_or(false),
                                is_default: false, // Will be set based on repo default branch
                            });
                        }
                    }
                }
            }
            _ => return Err(VcsError::UnsupportedVcs(vcs_type)),
        }
        
        Ok(branches)
    }
    
    async fn get_file_content(
        &self,
        url: &str,
        path: &str,
        branch: &str,
        credentials: &RepositoryCredentials,
    ) -> VcsResult<FileContent> {
        // Implementation for getting file content from Git repositories
        // This would involve API calls to get file contents
        Err(VcsError::OperationFailed("Not implemented yet".to_string()))
    }
    
    async fn list_files(
        &self,
        url: &str,
        path: &str,
        branch: &str,
        credentials: &RepositoryCredentials,
        recursive: bool,
    ) -> VcsResult<Vec<String>> {
        // Implementation for listing files in Git repositories
        Err(VcsError::OperationFailed("Not implemented yet".to_string()))
    }
    
    async fn sync_repository(
        &self,
        repo_info: &RepositoryInfo,
        local_path: &str,
    ) -> VcsResult<()> {
        // Implementation for cloning/pulling Git repositories
        Err(VcsError::OperationFailed("Not implemented yet".to_string()))
    }
    
    async fn setup_webhook(
        &self,
        url: &str,
        webhook_url: &str,
        secret: &str,
        credentials: &RepositoryCredentials,
    ) -> VcsResult<String> {
        // Implementation for setting up webhooks
        Err(VcsError::OperationFailed("Not implemented yet".to_string()))
    }
}

/// SVN connector implementation
pub struct SvnConnector {
    client: Client,
}

impl SvnConnector {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl VcsConnector for SvnConnector {
    async fn test_connection(&self, credentials: &RepositoryCredentials) -> VcsResult<bool> {
        // SVN connection testing would require different approach
        Err(VcsError::UnsupportedVcs(VcsType::Subversion))
    }
    
    async fn get_repository_metadata(
        &self,
        url: &str,
        credentials: &RepositoryCredentials,
    ) -> VcsResult<RepositoryMetadata> {
        Err(VcsError::UnsupportedVcs(VcsType::Subversion))
    }
    
    async fn list_branches(
        &self,
        url: &str,
        credentials: &RepositoryCredentials,
    ) -> VcsResult<Vec<BranchInfo>> {
        Err(VcsError::UnsupportedVcs(VcsType::Subversion))
    }
    
    async fn get_file_content(
        &self,
        url: &str,
        path: &str,
        branch: &str,
        credentials: &RepositoryCredentials,
    ) -> VcsResult<FileContent> {
        Err(VcsError::UnsupportedVcs(VcsType::Subversion))
    }
    
    async fn list_files(
        &self,
        url: &str,
        path: &str,
        branch: &str,
        credentials: &RepositoryCredentials,
        recursive: bool,
    ) -> VcsResult<Vec<String>> {
        Err(VcsError::UnsupportedVcs(VcsType::Subversion))
    }
    
    async fn sync_repository(
        &self,
        repo_info: &RepositoryInfo,
        local_path: &str,
    ) -> VcsResult<()> {
        Err(VcsError::UnsupportedVcs(VcsType::Subversion))
    }
    
    async fn setup_webhook(
        &self,
        url: &str,
        webhook_url: &str,
        secret: &str,
        credentials: &RepositoryCredentials,
    ) -> VcsResult<String> {
        Err(VcsError::UnsupportedVcs(VcsType::Subversion))
    }
}

/// Factory for creating VCS connectors
pub struct VcsConnectorFactory;

impl VcsConnectorFactory {
    pub fn create_connector(vcs_type: &VcsType) -> Box<dyn VcsConnector> {
        match vcs_type {
            VcsType::Git => Box::new(GitConnector::new()),
            VcsType::Subversion => Box::new(SvnConnector::new()),
            VcsType::Mercurial => Box::new(SvnConnector::new()), // Placeholder
            VcsType::Bazaar => Box::new(SvnConnector::new()),    // Placeholder
            VcsType::Perforce => Box::new(SvnConnector::new()),  // Placeholder
            VcsType::Unknown => Box::new(GitConnector::new()),   // Default to Git
        }
    }
}