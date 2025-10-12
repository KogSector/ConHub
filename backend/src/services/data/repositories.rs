use std::collections::HashMap;
use std::sync::Mutex;
use chrono::Utc;
use uuid::Uuid;

use crate::models::{
    ConnectRepositoryRequest, RepositoryInfo, RepositoryCredentials, 
    RepositoryConfig, RepositorySyncStatus, VcsType, VcsProvider
};
use crate::services::vcs_detector::VcsDetector;
use crate::services::vcs_connector::{VcsConnectorFactory, VcsError, VcsResult};

lazy_static::lazy_static! {
    static ref REPOSITORIES: Mutex<HashMap<String, RepositoryInfo>> = Mutex::new(HashMap::new());
}

/// Enhanced repository service with VCS support
pub struct RepositoryService;

impl RepositoryService {
    pub fn new() -> Self {
        Self
    }
    
    /// Connect a new repository with auto-detection of VCS type
    pub async fn connect_repository(&self, request: ConnectRepositoryRequest) -> VcsResult<RepositoryInfo> {
        // Auto-detect VCS type and provider if not specified
        let (vcs_type, provider) = match (&request.vcs_type, &request.provider) {
            (Some(vcs), Some(prov)) => (vcs.clone(), prov.clone()),
            _ => VcsDetector::detect_from_url(&request.url)
                .map_err(|e| VcsError::InvalidUrl(e))?,
        };
        
        // Extract repository owner and name
        let (owner, repo_name) = VcsDetector::extract_repo_info(&request.url)
            .map_err(|e| VcsError::InvalidUrl(e))?;
        
        // Create VCS connector for the detected type
        let connector = VcsConnectorFactory::create_connector(&vcs_type);
        
        // Test connection with provided credentials
        let connection_valid = connector.test_connection(&request.credentials).await?;
        if !connection_valid {
            return Err(VcsError::AuthenticationFailed("Invalid credentials".to_string()));
        }
        
        // Get repository metadata
        let metadata = connector.get_repository_metadata(&request.url, &request.credentials).await?;
        
        // Create repository configuration with defaults
        let config = request.config.unwrap_or_else(|| RepositoryConfig {
            branch: metadata.default_branch.clone(),
            auto_sync: true,
            webhook_enabled: false,
            webhook_secret: None,
            include_branches: vec![metadata.default_branch.clone()],
            exclude_paths: vec![
                ".git/".to_string(),
                "node_modules/".to_string(),
                "target/".to_string(),
                "build/".to_string(),
                "dist/".to_string(),
            ],
            include_file_extensions: vec![
                ".rs".to_string(),
                ".js".to_string(),
                ".ts".to_string(),
                ".py".to_string(),
                ".java".to_string(),
                ".go".to_string(),
                ".md".to_string(),
                ".txt".to_string(),
                ".json".to_string(),
                ".yaml".to_string(),
                ".yml".to_string(),
            ],
            max_file_size_mb: 10,
            sync_frequency_minutes: Some(60),
        });
        
        // Generate clone URLs
        let clone_urls = VcsDetector::generate_clone_urls(&request.url, &provider)
            .map_err(|e| VcsError::InvalidUrl(e))?;
        
        // Create repository info
        let repo_info = RepositoryInfo {
            id: Uuid::new_v4().to_string(),
            name: repo_name,
            description: metadata.description,
            url: request.url,
            vcs_type,
            provider,
            owner,
            is_private: metadata.is_private,
            default_branch: metadata.default_branch,
            clone_url: clone_urls.https,
            ssh_url: clone_urls.ssh,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_synced: None,
            sync_status: RepositorySyncStatus::Connected,
            credentials: request.credentials,
            config,
        };
        
        // Store repository info
        let mut repositories = REPOSITORIES.lock().unwrap();
        repositories.insert(repo_info.id.clone(), repo_info.clone());
        
        Ok(repo_info)
    }
    
    /// Get repository by ID
    pub fn get_repository(&self, id: &str) -> Option<RepositoryInfo> {
        let repositories = REPOSITORIES.lock().unwrap();
        repositories.get(id).cloned()
    }
    
    /// List all connected repositories
    pub fn list_repositories(&self) -> Vec<RepositoryInfo> {
        let repositories = REPOSITORIES.lock().unwrap();
        repositories.values().cloned().collect()
    }
    
    /// Update repository configuration
    pub fn update_repository_config(&self, id: &str, config: RepositoryConfig) -> VcsResult<()> {
        let mut repositories = REPOSITORIES.lock().unwrap();
        if let Some(repo) = repositories.get_mut(id) {
            repo.config = config;
            repo.updated_at = Utc::now();
            Ok(())
        } else {
            Err(VcsError::RepositoryNotFound(format!("Repository with ID {} not found", id)))
        }
    }
    
    /// Test repository connection
    pub async fn test_repository_connection(&self, id: &str) -> VcsResult<bool> {
        let repo_info = self.get_repository(id)
            .ok_or_else(|| VcsError::RepositoryNotFound(format!("Repository with ID {} not found", id)))?;
        
        let connector = VcsConnectorFactory::create_connector(&repo_info.vcs_type);
        connector.test_connection(&repo_info.credentials).await
    }
    
    /// Sync repository content
    pub async fn sync_repository(&self, id: &str) -> VcsResult<()> {
        let mut repositories = REPOSITORIES.lock().unwrap();
        
        if let Some(repo) = repositories.get_mut(id) {
            // Update sync status to syncing
            repo.sync_status = RepositorySyncStatus::Syncing;
            repo.updated_at = Utc::now();
            
            let _connector = VcsConnectorFactory::create_connector(&repo.vcs_type);
            
            // Here you would implement the actual syncing logic
            // For now, we'll just simulate a successful sync
            
            // Update sync status and timestamp
            repo.sync_status = RepositorySyncStatus::SyncCompleted;
            repo.last_synced = Some(Utc::now());
            
            Ok(())
        } else {
            Err(VcsError::RepositoryNotFound(format!("Repository with ID {} not found", id)))
        }
    }
    
    /// Delete repository connection
    pub fn disconnect_repository(&self, id: &str) -> VcsResult<()> {
        let mut repositories = REPOSITORIES.lock().unwrap();
        
        if repositories.remove(id).is_some() {
            Ok(())
        } else {
            Err(VcsError::RepositoryNotFound(format!("Repository with ID {} not found", id)))
        }
    }
    
    /// List branches for a repository
    pub async fn list_repository_branches(&self, id: &str) -> VcsResult<Vec<String>> {
        let repo_info = self.get_repository(id)
            .ok_or_else(|| VcsError::RepositoryNotFound(format!("Repository with ID {} not found", id)))?;
        
        let connector = VcsConnectorFactory::create_connector(&repo_info.vcs_type);
        let branches = connector.list_branches(&repo_info.url, &repo_info.credentials).await?;
        
        Ok(branches.into_iter().map(|b| b.name).collect())
    }
    
    /// Update repository credentials
    pub fn update_repository_credentials(&self, id: &str, credentials: RepositoryCredentials) -> VcsResult<()> {
        let mut repositories = REPOSITORIES.lock().unwrap();
        
        if let Some(repo) = repositories.get_mut(id) {
            repo.credentials = credentials;
            repo.updated_at = Utc::now();
            Ok(())
        } else {
            Err(VcsError::RepositoryNotFound(format!("Repository with ID {} not found", id)))
        }
    }
    
    /// Setup webhook for repository
    pub async fn setup_repository_webhook(&self, id: &str, webhook_url: &str) -> VcsResult<String> {
        let mut repositories = REPOSITORIES.lock().unwrap();
        
        if let Some(repo) = repositories.get_mut(id) {
            let connector = VcsConnectorFactory::create_connector(&repo.vcs_type);
            
            // Generate webhook secret if not provided
            let secret = repo.config.webhook_secret.clone()
                .unwrap_or_else(|| Uuid::new_v4().to_string());
            
            let webhook_id = connector.setup_webhook(
                &repo.url,
                webhook_url,
                &secret,
                &repo.credentials,
            ).await?;
            
            // Update repository configuration
            repo.config.webhook_enabled = true;
            repo.config.webhook_secret = Some(secret.clone());
            repo.updated_at = Utc::now();
            
            Ok(webhook_id)
        } else {
            Err(VcsError::RepositoryNotFound(format!("Repository with ID {} not found", id)))
        }
    }
}

/// Credential validation service
pub struct CredentialValidator;

impl CredentialValidator {
    /// Validate credentials for a specific VCS type
    pub async fn validate_credentials(
        vcs_type: &VcsType,
        _provider: &VcsProvider,
        credentials: &RepositoryCredentials,
    ) -> VcsResult<bool> {
        let connector = VcsConnectorFactory::create_connector(vcs_type);
        connector.test_connection(credentials).await
    }
    
    /// Check if credentials are expired
    #[allow(dead_code)]
    pub fn are_credentials_expired(credentials: &RepositoryCredentials) -> bool {
        if let Some(expires_at) = credentials.expires_at {
            return Utc::now() > expires_at;
        }
        false
    }
    
    /// Refresh OAuth token if possible
    #[allow(dead_code)]
    pub async fn refresh_oauth_token(_credentials: &RepositoryCredentials) -> VcsResult<RepositoryCredentials> {
        // OAuth functionality removed - return error for now
        Err(VcsError::OperationFailed("OAuth functionality has been removed".to_string()))
    }
}

/// Repository synchronization scheduler
#[allow(dead_code)]
pub struct RepositorySyncScheduler;

impl RepositorySyncScheduler {
    /// Schedule automatic synchronization for repositories
    #[allow(dead_code)]
    pub async fn schedule_sync(&self, repository_service: &RepositoryService) -> VcsResult<()> {
        let repositories = repository_service.list_repositories();
        
        for repo in repositories {
            if repo.config.auto_sync && repo.config.sync_frequency_minutes.is_some() {
                // Here you would implement the scheduling logic
                // This could use a background task scheduler like tokio-cron-scheduler
                log::info!("Scheduling sync for repository: {}", repo.name);
            }
        }
        
        Ok(())
    }
    
    /// Run immediate sync for all auto-sync enabled repositories
    #[allow(dead_code)]
    pub async fn sync_all(&self, repository_service: &RepositoryService) -> Vec<(String, VcsResult<()>)> {
        let repositories = repository_service.list_repositories();
        let mut results = Vec::new();
        
        for repo in repositories {
            if repo.config.auto_sync {
                let result = repository_service.sync_repository(&repo.id).await;
                results.push((repo.id, result));
            }
        }
        
        results
    }
}