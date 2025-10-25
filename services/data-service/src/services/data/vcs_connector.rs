use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

use conhub_models::{
    VcsType, VcsProvider, RepositoryInfo, RepositoryCredentials, 
    CredentialType
};
use crate::services::vcs_detector::{VcsDetector, CloneUrls};


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


#[derive(Debug, Clone)]
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
        let mut request = self.client.get(url);
        
        
        match &credentials.credential_type {
            CredentialType::PersonalAccessToken { token } => {
                println!("Making API request to: {}", url);
                
                if url.contains("gitlab") {
                    
                    println!("Using GitLab Bearer auth");
                    request = request.header("Authorization", format!("Bearer {}", token));
                } else if token.starts_with("github_pat_") {
                    
                    println!("Using Bearer auth for fine-grained GitHub token");
                    request = request.header("Authorization", format!("Bearer {}", token));
                } else if token.starts_with("ghp_") {
                    
                    println!("Using token auth for classic GitHub token");
                    request = request.header("Authorization", format!("token {}", token));
                } else {
                    
                    println!("Using Bearer auth for unknown token type");
                    request = request.header("Authorization", format!("Bearer {}", token));
                }
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
                                is_default: false,
                            });
                        }
                    }
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
        
        Ok(branches)
    }
    
    async fn get_file_content(
        &self,
        _url: &str,
        _path: &str,
        _branch: &str,
        _credentials: &RepositoryCredentials,
    ) -> VcsResult<FileContent> {
        
        
        Err(VcsError::OperationFailed("Not implemented yet".to_string()))
    }
    
    async fn list_files(
        &self,
        _url: &str,
        _path: &str,
        _branch: &str,
        _credentials: &RepositoryCredentials,
        _recursive: bool,
    ) -> VcsResult<Vec<String>> {
        
        Err(VcsError::OperationFailed("Not implemented yet".to_string()))
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