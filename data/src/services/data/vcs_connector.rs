use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use serde::{Serialize, Deserialize};
use base64::{Engine as _, engine::general_purpose};
use std::collections::HashSet;

use conhub_config::{
    VcsType, VcsProvider, RepositoryInfo, RepositoryCredentials, 
    CredentialType
};
use crate::services::data::vcs_detector::{VcsDetector, CloneUrls};


pub type VcsResult<T> = Result<T, VcsError>;


#[derive(Debug, thiserror::Error)]
pub enum VcsError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Invalid credentials: {0}")]
    #[allow(dead_code)]
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


#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
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


#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FileContent {
    pub path: String,
    pub content: String,
    pub sha: String,
    pub size: u64,
    pub url: String,
}


#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BranchInfo {
    pub name: String,
    pub sha: String,
    pub protected: bool,
    pub is_default: bool,
}


#[async_trait]
pub trait VcsConnector: Send + Sync {
    
    async fn test_connection(&self, credentials: &RepositoryCredentials) -> VcsResult<bool>;
    
    
    async fn get_repository_metadata(
        &self, 
        url: &str, 
        credentials: &RepositoryCredentials
    ) -> VcsResult<RepositoryMetadata>;
    
    
    async fn list_branches(
        &self, 
        url: &str, 
        credentials: &RepositoryCredentials
    ) -> VcsResult<Vec<BranchInfo>>;
    
    
    #[allow(dead_code)]
    async fn get_file_content(
        &self,
        url: &str,
        path: &str,
        branch: &str,
        credentials: &RepositoryCredentials,
    ) -> VcsResult<FileContent>;
    
    
    #[allow(dead_code)]
    async fn list_files(
        &self,
        url: &str,
        path: &str,
        branch: &str,
        credentials: &RepositoryCredentials,
        recursive: bool,
    ) -> VcsResult<Vec<String>>;
    
    
    #[allow(dead_code)]
    async fn sync_repository(
        &self,
        repo_info: &RepositoryInfo,
        local_path: &str,
    ) -> VcsResult<()>;
    
    
    async fn setup_webhook(
        &self,
        url: &str,
        webhook_url: &str,
        secret: &str,
        credentials: &RepositoryCredentials,
    ) -> VcsResult<String>;
    
    /// Get file extensions present in the repository
    async fn get_file_extensions(
        &self,
        url: &str,
        branch: &str,
        credentials: &RepositoryCredentials,
    ) -> VcsResult<Vec<String>>;
}


pub struct GitConnector {
    client: Client,
}

impl GitConnector {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
    
    
    fn get_api_base_url(&self, provider: &VcsProvider, url: &str) -> VcsResult<String> {
        match provider {
            VcsProvider::GitHub => Ok("https://api.github.com".to_string()),
            VcsProvider::GitLab => {
                if url.contains("gitlab.com") {
                    Ok("https://gitlab.com/api/v4".to_string())
                } else {
                    
                    let base = url.split("/").take(3).collect::<Vec<_>>().join("/");
                    Ok(format!("{}/api/v4", base))
                }
            },
            VcsProvider::Bitbucket => Ok("https://api.bitbucket.org/2.0".to_string()),
            VcsProvider::Azure => Ok("https://dev.azure.com".to_string()),
            VcsProvider::SelfHosted => {
                if url.contains("gitlab") {
                    let base = url.split("/").take(3).collect::<Vec<_>>().join("/");
                    Ok(format!("{}/api/v4", base))
                } else {
                    Err(VcsError::UnsupportedVcs(VcsType::Git))
                }
            }
            _ => Err(VcsError::UnsupportedVcs(VcsType::Git)),
        }
    }
    
    
    async fn make_api_request(
        &self,
        url: &str,
        credentials: &RepositoryCredentials,
    ) -> VcsResult<Value> {
        tracing::info!("[VCS_CONNECTOR] make_api_request to: {}", url);
        tracing::info!("[VCS_CONNECTOR] Credential type: {:?}", std::mem::discriminant(&credentials.credential_type));
        
        let mut request = self.client.get(url)
            .header("User-Agent", "ConHub")
            .header("Accept", "application/json");
        
        match &credentials.credential_type {
            CredentialType::PersonalAccessToken { token } => {
                tracing::info!("[VCS_CONNECTOR] Using PersonalAccessToken (length: {})", token.len());
                
                if url.contains("gitlab") {
                    tracing::info!("[VCS_CONNECTOR] Using GitLab Bearer auth");
                    request = request.header("Authorization", format!("Bearer {}", token));
                } else if token.starts_with("github_pat_") {
                    tracing::info!("[VCS_CONNECTOR] Using Bearer auth for fine-grained GitHub token");
                    request = request.header("Authorization", format!("Bearer {}", token));
                } else if token.starts_with("ghp_") {
                    tracing::info!("[VCS_CONNECTOR] Using token auth for classic GitHub token");
                    request = request.header("Authorization", format!("token {}", token));
                } else {
                    tracing::info!("[VCS_CONNECTOR] Using Bearer auth for unknown token type");
                    request = request.header("Authorization", format!("Bearer {}", token));
                }
            }
            CredentialType::UsernamePassword { username, password } => {
                tracing::info!("[VCS_CONNECTOR] Using UsernamePassword for: {}", username);
                request = request.basic_auth(username, Some(password));
            }
            CredentialType::AppPassword { username, app_password } => {
                tracing::info!("[VCS_CONNECTOR] Using AppPassword for: {}", username);
                request = request.basic_auth(username, Some(app_password));
            }
            CredentialType::None => {
                tracing::info!("[VCS_CONNECTOR] No credentials provided");
            }
            _ => {
                tracing::warn!("[VCS_CONNECTOR] Unsupported credential type");
            }
        }
        
        tracing::info!("[VCS_CONNECTOR] Sending HTTP request");
        let response = request.send().await
            .map_err(|e| {
                tracing::error!("[VCS_CONNECTOR] Network error: {}", e);
                VcsError::NetworkError(e.to_string())
            })?;
        
        let status = response.status();
        tracing::info!("[VCS_CONNECTOR] HTTP response status: {}", status);
        
        if status.is_success() {
            tracing::info!("[VCS_CONNECTOR] Parsing JSON response");
            let json = response.json::<Value>().await
                .map_err(|e| {
                    tracing::error!("[VCS_CONNECTOR] JSON parsing error: {}", e);
                    VcsError::OperationFailed(e.to_string())
                })?;
            tracing::info!("[VCS_CONNECTOR] Successfully parsed JSON response");
            Ok(json)
        } else {
            let error_body = response.text().await.unwrap_or_default();
            tracing::error!("[VCS_CONNECTOR] HTTP error {}: {}", status, error_body);
            
            match status.as_u16() {
                401 => Err(VcsError::AuthenticationFailed(format!("Invalid credentials: {}", error_body))),
                403 => Err(VcsError::PermissionDenied(format!("Access denied: {}", error_body))),
                404 => Err(VcsError::RepositoryNotFound(format!("Repository not found: {}", error_body))),
                _ => Err(VcsError::OperationFailed(format!("HTTP {}: {}", status, error_body))),
            }
        }
    }
}

#[async_trait]
impl VcsConnector for GitConnector {
    async fn test_connection(&self, credentials: &RepositoryCredentials) -> VcsResult<bool> {
        
        match &credentials.credential_type {
            CredentialType::PersonalAccessToken { token } => {
                println!("Testing connection with token starting with: {}", &token[..8]);
                
                let auth_header = if token.starts_with("github_pat_") {
                    println!("Using Bearer auth for fine-grained token");
                    format!("Bearer {}", token)
                } else {
                    println!("Using token auth for classic token");
                    format!("token {}", token)
                };
                
                println!("Making request to GitHub user API...");
                let response = self.client
                    .get("https://api.github.com/user")
                    .header("Authorization", auth_header)
                    .header("User-Agent", "ConHub")
                    .send()
                    .await
                    .map_err(|e| VcsError::NetworkError(e.to_string()))?;
                
                println!("Response status: {}", response.status());
                Ok(response.status().is_success())
            }
            _ => Ok(false), 
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
        
        println!("Successfully got repository metadata for: {}", api_url);
        
        
        let metadata = match provider {
            VcsProvider::GitHub => {
                RepositoryMetadata {
                    name: repo_data["name"].as_str().unwrap_or(&repo).to_string(),
                    description: repo_data["description"].as_str().map(|s| s.to_string()),
                    is_private: repo_data["private"].as_bool().unwrap_or(false),
                    default_branch: repo_data["default_branch"].as_str().unwrap_or("main").to_string(),
                    branches: vec![],
                    tags: vec![],
                    language: repo_data["language"].as_str().map(|s| s.to_string()),
                    size_kb: repo_data["size"].as_u64(),
                    stars: repo_data["stargazers_count"].as_u64().map(|n| n as u32),
                    forks: repo_data["forks_count"].as_u64().map(|n| n as u32),
                    clone_urls: VcsDetector::generate_clone_urls(url, &provider)
                        .map_err(|e| VcsError::InvalidUrl(e))?,
                }
            },
            VcsProvider::GitLab => {
                RepositoryMetadata {
                    name: repo_data["name"].as_str().unwrap_or(&repo).to_string(),
                    description: repo_data["description"].as_str().map(|s| s.to_string()),
                    is_private: !repo_data["visibility"].as_str().unwrap_or("private").eq("public"),
                    default_branch: repo_data["default_branch"].as_str().unwrap_or("main").to_string(),
                    branches: vec![],
                    tags: vec![],
                    language: None, 
                    size_kb: None,  
                    stars: repo_data["star_count"].as_u64().map(|n| n as u32),
                    forks: repo_data["forks_count"].as_u64().map(|n| n as u32),
                    clone_urls: VcsDetector::generate_clone_urls(url, &provider)
                        .map_err(|e| VcsError::InvalidUrl(e))?,
                }
            },
            VcsProvider::Bitbucket => {
                RepositoryMetadata {
                    name: repo_data["name"].as_str().unwrap_or(&repo).to_string(),
                    description: repo_data["description"].as_str().map(|s| s.to_string()),
                    is_private: repo_data["is_private"].as_bool().unwrap_or(false),
                    default_branch: "main".to_string(), 
                    branches: vec![],
                    tags: vec![],
                    language: repo_data["language"].as_str().map(|s| s.to_string()),
                    size_kb: repo_data["size"].as_u64().map(|s| s / 1024), 
                    stars: None, 
                    forks: None, 
                    clone_urls: VcsDetector::generate_clone_urls(url, &provider)
                        .map_err(|e| VcsError::InvalidUrl(e))?,
                }
            },
            _ => return Err(VcsError::UnsupportedVcs(vcs_type)),
        };
        
        Ok(metadata)
    }
    
    async fn list_branches(
        &self,
        url: &str,
        credentials: &RepositoryCredentials,
    ) -> VcsResult<Vec<BranchInfo>> {
        tracing::info!("[VCS_CONNECTOR] list_branches called for: {}", url);
        
        let (vcs_type, provider) = VcsDetector::detect_from_url(url)
            .map_err(|e| {
                tracing::error!("[VCS_CONNECTOR] URL detection failed: {}", e);
                VcsError::InvalidUrl(e)
            })?;
        
        tracing::info!("[VCS_CONNECTOR] Detected - VCS: {:?}, Provider: {:?}", vcs_type, provider);
        
        let (owner, repo) = VcsDetector::extract_repo_info(url)
            .map_err(|e| {
                tracing::error!("[VCS_CONNECTOR] Repo info extraction failed: {}", e);
                VcsError::InvalidUrl(e)
            })?;
        
        tracing::info!("[VCS_CONNECTOR] Extracted - Owner: {}, Repo: {}", owner, repo);
        
        let api_base = self.get_api_base_url(&provider, url)?;
        tracing::info!("[VCS_CONNECTOR] API base URL: {}", api_base);
        
        let api_url = match provider {
            VcsProvider::GitHub => format!("{}/repos/{}/{}/branches", api_base, owner, repo),
            VcsProvider::GitLab => format!("{}/projects/{}%2F{}/repository/branches", api_base, owner, repo),
            VcsProvider::Bitbucket => format!("{}/repositories/{}/{}/refs/branches", api_base, owner, repo),
            _ => {
                tracing::error!("[VCS_CONNECTOR] Unsupported VCS type: {:?}", vcs_type);
                return Err(VcsError::UnsupportedVcs(vcs_type));
            },
        };
        
        tracing::info!("[VCS_CONNECTOR] Making API request to: {}", api_url);
        
        let branches_data = self.make_api_request(&api_url, credentials).await
            .map_err(|e| {
                tracing::error!("[VCS_CONNECTOR] API request failed: {}", e);
                e
            })?;
        
        tracing::info!("[VCS_CONNECTOR] API request successful, parsing response");
        
        let mut branches = Vec::new();
        
        match provider {
            VcsProvider::GitHub => {
                if let Some(branch_array) = branches_data.as_array() {
                    tracing::info!("[VCS_CONNECTOR] GitHub: Processing {} branches", branch_array.len());
                    for branch in branch_array {
                        if let Some(name) = branch["name"].as_str() {
                            tracing::debug!("[VCS_CONNECTOR] GitHub: Found branch {}", name);
                            branches.push(BranchInfo {
                                name: name.to_string(),
                                sha: branch["commit"]["sha"].as_str().unwrap_or("").to_string(),
                                protected: branch["protected"].as_bool().unwrap_or(false),
                                is_default: false,
                            });
                        }
                    }
                } else {
                    tracing::error!("[VCS_CONNECTOR] GitHub: Response is not an array");
                }
            },
            VcsProvider::GitLab => {
                if let Some(branch_array) = branches_data.as_array() {
                    for branch in branch_array {
                        if let Some(name) = branch["name"].as_str() {
                            branches.push(BranchInfo {
                                name: name.to_string(),
                                sha: branch["commit"]["id"].as_str().unwrap_or("").to_string(),
                                protected: branch["protected"].as_bool().unwrap_or(false),
                                is_default: branch["default"].as_bool().unwrap_or(false),
                            });
                        }
                    }
                }
            },
            VcsProvider::Bitbucket => {
                if let Some(values) = branches_data["values"].as_array() {
                    for branch in values {
                        if let Some(name) = branch["name"].as_str() {
                            branches.push(BranchInfo {
                                name: name.to_string(),
                                sha: branch["target"]["hash"].as_str().unwrap_or("").to_string(),
                                protected: false, 
                                is_default: false, 
                            });
                        }
                    }
                }
            },
            _ => return Err(VcsError::UnsupportedVcs(vcs_type)),
        }
        
        tracing::info!("[VCS_CONNECTOR] Returning {} branches", branches.len());
        Ok(branches)
    }
    
    async fn get_file_content(
        &self,
        url: &str,
        path: &str,
        branch: &str,
        credentials: &RepositoryCredentials,
    ) -> VcsResult<FileContent> {
        let (_vcs_type, provider) = VcsDetector::detect_from_url(url)
            .map_err(|e| VcsError::InvalidUrl(e))?;
        let (owner, repo) = VcsDetector::extract_repo_info(url)
            .map_err(|e| VcsError::InvalidUrl(e))?;
        let access_token = match &credentials.credential_type {
            CredentialType::PersonalAccessToken { token } => token.clone(),
            _ => return Err(VcsError::InvalidCredentials("Unsupported credential type".to_string())),
        };
        let api_url = match provider {
            VcsProvider::GitHub => format!(
                "https://api.github.com/repos/{}/{}/contents/{}{}",
                owner, repo, path,
                if branch.is_empty() { String::new() } else { format!("?ref={}", branch) }
            ),
            _ => return Err(VcsError::UnsupportedVcs(VcsType::Git)),
        };
        let response = self.client
            .get(&api_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "ConHub")
            .send()
            .await
            .map_err(|e| VcsError::NetworkError(e.to_string()))?;
        if !response.status().is_success() {
            return Err(VcsError::OperationFailed(format!("HTTP {}", response.status())));
        }
        let file: serde_json::Value = response.json().await
            .map_err(|e| VcsError::OperationFailed(e.to_string()))?;
        let content_b64 = file["content"].as_str().unwrap_or("").replace('\n', "");
        let decoded = general_purpose::STANDARD.decode(&content_b64).map_err(|e| VcsError::OperationFailed(e.to_string()))?;
        let name = file["name"].as_str().unwrap_or(path).to_string();
        let sha = file["sha"].as_str().unwrap_or("").to_string();
        let size = file["size"].as_i64().unwrap_or(decoded.len() as i64) as u64;
        let url_html = file["html_url"].as_str().unwrap_or("").to_string();
        Ok(FileContent {
            path: path.to_string(),
            content: String::from_utf8_lossy(&decoded).to_string(),
            sha,
            size,
            url: url_html,
        })
    }
    
    async fn list_files(
        &self,
        url: &str,
        path: &str,
        branch: &str,
        credentials: &RepositoryCredentials,
        recursive: bool,
    ) -> VcsResult<Vec<String>> {
        let (_vcs_type, provider) = VcsDetector::detect_from_url(url)
            .map_err(|e| VcsError::InvalidUrl(e))?;
        let (owner, repo) = VcsDetector::extract_repo_info(url)
            .map_err(|e| VcsError::InvalidUrl(e))?;

        let mut results: Vec<String> = Vec::new();
        let mut stack: Vec<String> = vec![path.to_string()];
        let access_token = match &credentials.credential_type {
            CredentialType::PersonalAccessToken { token } => token.clone(),
            CredentialType::AppPassword { username: _, app_password: _ } => {
                return Err(VcsError::InvalidCredentials("Use PersonalAccessToken for GitHub".to_string()))
            }
            _ => return Err(VcsError::InvalidCredentials("Unsupported credential type".to_string())),
        };

        while let Some(curr_path) = stack.pop() {
            let url_api = match provider {
                VcsProvider::GitHub => format!(
                    "https://api.github.com/repos/{}/{}/contents/{}{}",
                    owner,
                    repo,
                    curr_path,
                    if branch.is_empty() { String::new() } else { format!("?ref={}", branch) }
                ),
                _ => return Err(VcsError::UnsupportedVcs(VcsType::Git)),
            };

            let mut request = self.client.get(&url_api)
                .header("Authorization", format!("Bearer {}", access_token))
                .header("User-Agent", "ConHub");

            let response = request.send().await
                .map_err(|e| VcsError::NetworkError(e.to_string()))?;
            if !response.status().is_success() {
                return Err(VcsError::OperationFailed(format!("HTTP {}", response.status())));
            }
            let entries = response.json::<serde_json::Value>().await
                .map_err(|e| VcsError::OperationFailed(e.to_string()))?;

            if let Some(arr) = entries.as_array() {
                for entry in arr {
                    let file_type = entry["type"].as_str().unwrap_or("");
                    let file_path = entry["path"].as_str().unwrap_or("");
                    if file_type == "dir" {
                        if recursive { stack.push(file_path.to_string()); }
                    } else if file_type == "file" {
                        results.push(format!("{}/{}/{}", owner, repo, file_path));
                    }
                }
            } else if entries["type"].as_str() == Some("file") {
                if let Some(file_path) = entries["path"].as_str() {
                    results.push(format!("{}/{}/{}", owner, repo, file_path));
                }
            }
        }

        Ok(results)
    }
    
    async fn sync_repository(
        &self,
        _repo_info: &RepositoryInfo,
        _local_path: &str,
    ) -> VcsResult<()> {
        
        Err(VcsError::OperationFailed("Not implemented yet".to_string()))
    }
    
    async fn setup_webhook(
        &self,
        _url: &str,
        _webhook_url: &str,
        _secret: &str,
        _credentials: &RepositoryCredentials,
    ) -> VcsResult<String> {
        
        Err(VcsError::OperationFailed("Not implemented yet".to_string()))
    }
    
    async fn get_file_extensions(
        &self,
        url: &str,
        branch: &str,
        credentials: &RepositoryCredentials,
    ) -> VcsResult<Vec<String>> {
        let files = self.list_files(url, "", branch, credentials, true).await?;
        let mut extensions = HashSet::new();
        
        for file_path in files {
            // Extract file path from owner/repo/path format
            let path = file_path.splitn(3, '/').nth(2).unwrap_or(&file_path);
            if let Some(ext_start) = path.rfind('.') {
                let ext = &path[ext_start..];
                if ext.len() > 1 && ext.len() <= 10 { // Reasonable extension length
                    extensions.insert(ext.to_lowercase());
                }
            }
        }
        
        let mut result: Vec<String> = extensions.into_iter().collect();
        result.sort();
        Ok(result)
    }
}


pub struct SvnConnector {
    #[allow(dead_code)]
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
    async fn test_connection(&self, _credentials: &RepositoryCredentials) -> VcsResult<bool> {
        
        Err(VcsError::UnsupportedVcs(VcsType::Subversion))
    }
    
    async fn get_repository_metadata(
        &self,
        _url: &str,
        _credentials: &RepositoryCredentials,
    ) -> VcsResult<RepositoryMetadata> {
        Err(VcsError::UnsupportedVcs(VcsType::Subversion))
    }
    
    async fn list_branches(
        &self,
        _url: &str,
        _credentials: &RepositoryCredentials,
    ) -> VcsResult<Vec<BranchInfo>> {
        Err(VcsError::UnsupportedVcs(VcsType::Subversion))
    }
    
    async fn get_file_content(
        &self,
        _url: &str,
        _path: &str,
        _branch: &str,
        _credentials: &RepositoryCredentials,
    ) -> VcsResult<FileContent> {
        Err(VcsError::UnsupportedVcs(VcsType::Subversion))
    }
    
    async fn list_files(
        &self,
        _url: &str,
        _path: &str,
        _branch: &str,
        _credentials: &RepositoryCredentials,
        _recursive: bool,
    ) -> VcsResult<Vec<String>> {
        Err(VcsError::UnsupportedVcs(VcsType::Subversion))
    }
    
    async fn sync_repository(
        &self,
        _repo_info: &RepositoryInfo,
        _local_path: &str,
    ) -> VcsResult<()> {
        Err(VcsError::UnsupportedVcs(VcsType::Subversion))
    }
    
    async fn setup_webhook(
        &self,
        _url: &str,
        _webhook_url: &str,
        _secret: &str,
        _credentials: &RepositoryCredentials,
    ) -> VcsResult<String> {
        Err(VcsError::UnsupportedVcs(VcsType::Subversion))
    }
    
    async fn get_file_extensions(
        &self,
        _url: &str,
        _branch: &str,
        _credentials: &RepositoryCredentials,
    ) -> VcsResult<Vec<String>> {
        Err(VcsError::UnsupportedVcs(VcsType::Subversion))
    }
}


pub struct VcsConnectorFactory;

impl VcsConnectorFactory {
    pub fn create_connector(vcs_type: &VcsType) -> Box<dyn VcsConnector> {
        match vcs_type {
            VcsType::Git => Box::new(GitConnector::new()),
            VcsType::Subversion => Box::new(SvnConnector::new()),
            VcsType::Mercurial => Box::new(SvnConnector::new()), 
            VcsType::Bazaar => Box::new(SvnConnector::new()),    
            VcsType::Perforce => Box::new(SvnConnector::new()),  
            VcsType::Unknown => Box::new(GitConnector::new()),   
        }
    }
}
