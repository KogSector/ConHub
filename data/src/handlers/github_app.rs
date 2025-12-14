//! GitHub App API Handlers
//! 
//! Endpoints for:
//! - Getting installation URL
//! - Handling installation callback
//! - Listing repositories for an installation
//! - Saving repository sync configuration
//! - Triggering sync jobs

use actix_web::{web, HttpResponse, Result, HttpRequest};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{info, error, warn};
use std::sync::Arc;
use chrono::{Utc, Duration};

use crate::services::github_app_client::{GitHubAppClient, GitHubRepository};

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct InstallUrlResponse {
    pub install_url: String,
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct InstallCallbackQuery {
    pub installation_id: i64,
    pub setup_action: Option<String>, // install, update, or request
    pub state: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InstallCallbackResponse {
    pub success: bool,
    pub installation_id: Option<Uuid>,
    pub github_installation_id: i64,
    pub account_login: String,
    pub account_type: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ListReposQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct RepoWithConfig {
    pub github_repo_id: i64,
    pub full_name: String,
    pub name: String,
    pub owner: String,
    pub default_branch: String,
    pub private: bool,
    pub description: Option<String>,
    pub html_url: String,
    pub language: Option<String>,
    pub topics: Vec<String>,
    // Sync configuration (from DB if exists, defaults otherwise)
    pub sync_code: bool,
    pub sync_issues: bool,
    pub sync_prs: bool,
    pub is_selected: bool,
}

#[derive(Debug, Serialize)]
pub struct ListReposResponse {
    pub success: bool,
    pub installation_id: i64,
    pub total_count: i64,
    pub page: u32,
    pub per_page: u32,
    pub repositories: Vec<RepoWithConfig>,
}

#[derive(Debug, Deserialize)]
pub struct SaveRepoConfigRequest {
    pub repos: Vec<RepoConfigInput>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RepoConfigInput {
    pub github_repo_id: i64,
    pub full_name: String,
    pub name: String,
    pub owner: String,
    pub default_branch: String,
    pub private: bool,
    pub description: Option<String>,
    pub html_url: Option<String>,
    pub sync_code: bool,
    pub sync_issues: bool,
    pub sync_prs: bool,
}

#[derive(Debug, Serialize)]
pub struct SaveRepoConfigResponse {
    pub success: bool,
    pub repos_configured: usize,
    pub sync_jobs_created: Vec<String>,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct TriggerSyncRequest {
    pub job_type: String, // code, issues, prs, full
    pub force_full: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct TriggerSyncResponse {
    pub success: bool,
    pub job_id: Option<Uuid>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct InstallationInfo {
    pub id: Uuid,
    pub github_installation_id: i64,
    pub account_login: String,
    pub account_type: String,
    pub status: String,
    pub repos_count: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct ListInstallationsResponse {
    pub success: bool,
    pub installations: Vec<InstallationInfo>,
}

// ============================================================================
// In-Memory State (for demo/dev - replace with DB in production)
// ============================================================================

use tokio::sync::RwLock;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Installation {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub github_installation_id: i64,
    pub account_login: String,
    pub account_type: String,
    pub status: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct RepoConfig {
    pub id: Uuid,
    pub installation_id: Uuid,
    pub github_repo_id: i64,
    pub full_name: String,
    pub name: String,
    pub owner: String,
    pub default_branch: String,
    pub private: bool,
    pub sync_code: bool,
    pub sync_issues: bool,
    pub sync_prs: bool,
}

#[derive(Debug, Clone)]
pub struct SyncJob {
    pub id: Uuid,
    pub repo_config_id: Uuid,
    pub job_type: String,
    pub status: String,
    pub items_total: i32,
    pub items_processed: i32,
    pub created_at: chrono::DateTime<Utc>,
}

pub struct GitHubAppState {
    pub client: GitHubAppClient,
    pub oauth_states: RwLock<HashMap<String, (Uuid, chrono::DateTime<Utc>)>>, // state -> (tenant_id, expires_at)
    pub installations: RwLock<HashMap<Uuid, Installation>>,
    pub repo_configs: RwLock<HashMap<Uuid, RepoConfig>>,
    pub sync_jobs: RwLock<HashMap<Uuid, SyncJob>>,
}

impl GitHubAppState {
    pub fn new(client: GitHubAppClient) -> Self {
        Self {
            client,
            oauth_states: RwLock::new(HashMap::new()),
            installations: RwLock::new(HashMap::new()),
            repo_configs: RwLock::new(HashMap::new()),
            sync_jobs: RwLock::new(HashMap::new()),
        }
    }
}

// ============================================================================
// Handlers
// ============================================================================

/// Get GitHub App installation URL
/// GET /api/connectors/github/app/install-url
pub async fn get_install_url(
    state: web::Data<Arc<GitHubAppState>>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // Get tenant_id from auth (simplified - use real auth in production)
    let tenant_id = get_tenant_id_from_request(&req);
    
    // Generate state token
    let state_token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::minutes(15);
    
    // Store state for validation
    {
        let mut states = state.oauth_states.write().await;
        states.insert(state_token.clone(), (tenant_id, expires_at));
    }
    
    let install_url = state.client.get_install_url(&state_token);
    
    info!("🔗 Generated GitHub App install URL for tenant {}", tenant_id);
    
    Ok(HttpResponse::Ok().json(InstallUrlResponse {
        install_url,
        state: state_token,
    }))
}

/// Handle GitHub App installation callback
/// GET /api/connectors/github/app/callback
pub async fn handle_install_callback(
    state: web::Data<Arc<GitHubAppState>>,
    query: web::Query<InstallCallbackQuery>,
) -> Result<HttpResponse> {
    info!("📥 GitHub App callback: installation_id={}, action={:?}", 
          query.installation_id, query.setup_action);
    
    // Validate state if provided
    let tenant_id = if let Some(ref state_token) = query.state {
        let states = state.oauth_states.read().await;
        if let Some((tid, expires_at)) = states.get(state_token) {
            if Utc::now() > *expires_at {
                return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "success": false,
                    "error": "State token expired"
                })));
            }
            *tid
        } else {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": "Invalid state token"
            })));
        }
    } else {
        // For testing without state validation
        Uuid::nil()
    };
    
    // Clean up used state
    if let Some(ref state_token) = query.state {
        let mut states = state.oauth_states.write().await;
        states.remove(state_token);
    }
    
    // Get installation details from GitHub
    let installation_details = match state.client.get_installation(query.installation_id).await {
        Ok(details) => details,
        Err(e) => {
            error!("Failed to get installation details: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Failed to verify installation: {}", e)
            })));
        }
    };
    
    // Create installation record
    let installation_id = Uuid::new_v4();
    let installation = Installation {
        id: installation_id,
        tenant_id,
        github_installation_id: query.installation_id,
        account_login: installation_details.account.login.clone(),
        account_type: installation_details.account.account_type.clone(),
        status: "active".to_string(),
        created_at: Utc::now(),
    };
    
    // Store installation
    {
        let mut installations = state.installations.write().await;
        installations.insert(installation_id, installation.clone());
    }
    
    info!("✅ GitHub App installed: {} ({}) -> installation {}", 
          installation_details.account.login,
          installation_details.account.account_type,
          installation_id);
    
    Ok(HttpResponse::Ok().json(InstallCallbackResponse {
        success: true,
        installation_id: Some(installation_id),
        github_installation_id: query.installation_id,
        account_login: installation_details.account.login,
        account_type: installation_details.account.account_type,
        message: "GitHub App installed successfully".to_string(),
    }))
}

/// List installations for current tenant
/// GET /api/connectors/github/app/installations
pub async fn list_installations(
    state: web::Data<Arc<GitHubAppState>>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let tenant_id = get_tenant_id_from_request(&req);
    
    let installations = state.installations.read().await;
    let repo_configs = state.repo_configs.read().await;
    
    let tenant_installations: Vec<InstallationInfo> = installations
        .values()
        .filter(|i| i.tenant_id == tenant_id || tenant_id == Uuid::nil())
        .map(|i| {
            let repos_count = repo_configs.values()
                .filter(|r| r.installation_id == i.id)
                .count() as i64;
            
            InstallationInfo {
                id: i.id,
                github_installation_id: i.github_installation_id,
                account_login: i.account_login.clone(),
                account_type: i.account_type.clone(),
                status: i.status.clone(),
                repos_count,
                created_at: i.created_at.to_rfc3339(),
            }
        })
        .collect();
    
    Ok(HttpResponse::Ok().json(ListInstallationsResponse {
        success: true,
        installations: tenant_installations,
    }))
}

/// List repositories for an installation
/// GET /api/connectors/github/app/{installation_id}/repos
pub async fn list_installation_repos(
    state: web::Data<Arc<GitHubAppState>>,
    path: web::Path<Uuid>,
    query: web::Query<ListReposQuery>,
) -> Result<HttpResponse> {
    let installation_id = path.into_inner();
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(30).min(100);
    
    // Get installation
    let github_installation_id = {
        let installations = state.installations.read().await;
        match installations.get(&installation_id) {
            Some(i) => i.github_installation_id,
            None => {
                return Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "success": false,
                    "error": "Installation not found"
                })));
            }
        }
    };
    
    // Fetch repos from GitHub
    let repos_response = match state.client.list_installation_repositories(
        github_installation_id, page, per_page
    ).await {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to list repositories: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Failed to list repositories: {}", e)
            })));
        }
    };
    
    // Get existing configs
    let repo_configs = state.repo_configs.read().await;
    let existing_configs: HashMap<i64, &RepoConfig> = repo_configs
        .values()
        .filter(|r| r.installation_id == installation_id)
        .map(|r| (r.github_repo_id, r))
        .collect();
    
    // Map repos with config
    let repos_with_config: Vec<RepoWithConfig> = repos_response.repositories
        .into_iter()
        .map(|repo| {
            let config = existing_configs.get(&repo.id);
            RepoWithConfig {
                github_repo_id: repo.id,
                full_name: repo.full_name.clone(),
                name: repo.name.clone(),
                owner: repo.owner.login.clone(),
                default_branch: repo.default_branch.clone(),
                private: repo.private,
                description: repo.description.clone(),
                html_url: repo.html_url.clone(),
                language: repo.language.clone(),
                topics: repo.topics.clone().unwrap_or_default(),
                sync_code: config.map(|c| c.sync_code).unwrap_or(true),
                sync_issues: config.map(|c| c.sync_issues).unwrap_or(false),
                sync_prs: config.map(|c| c.sync_prs).unwrap_or(false),
                is_selected: config.is_some(),
            }
        })
        .collect();
    
    Ok(HttpResponse::Ok().json(ListReposResponse {
        success: true,
        installation_id: github_installation_id,
        total_count: repos_response.total_count,
        page,
        per_page,
        repositories: repos_with_config,
    }))
}

/// Save repository configurations
/// POST /api/connectors/github/app/{installation_id}/repos
pub async fn save_repo_configs(
    state: web::Data<Arc<GitHubAppState>>,
    path: web::Path<Uuid>,
    body: web::Json<SaveRepoConfigRequest>,
) -> Result<HttpResponse> {
    let installation_id = path.into_inner();
    
    // Verify installation exists
    {
        let installations = state.installations.read().await;
        if !installations.contains_key(&installation_id) {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "error": "Installation not found"
            })));
        }
    }
    
    let mut configs_saved = 0;
    let mut sync_jobs_created = Vec::new();
    
    {
        let mut repo_configs = state.repo_configs.write().await;
        let mut sync_jobs = state.sync_jobs.write().await;
        
        for repo_input in &body.repos {
            // Find existing config or create new
            let existing_id = repo_configs.values()
                .find(|r| r.installation_id == installation_id && r.github_repo_id == repo_input.github_repo_id)
                .map(|r| r.id);
            
            let config_id = existing_id.unwrap_or_else(Uuid::new_v4);
            
            let config = RepoConfig {
                id: config_id,
                installation_id,
                github_repo_id: repo_input.github_repo_id,
                full_name: repo_input.full_name.clone(),
                name: repo_input.name.clone(),
                owner: repo_input.owner.clone(),
                default_branch: repo_input.default_branch.clone(),
                private: repo_input.private,
                sync_code: repo_input.sync_code,
                sync_issues: repo_input.sync_issues,
                sync_prs: repo_input.sync_prs,
            };
            
            repo_configs.insert(config_id, config);
            configs_saved += 1;
            
            // Create sync jobs for enabled sync types
            if repo_input.sync_code {
                let job_id = Uuid::new_v4();
                sync_jobs.insert(job_id, SyncJob {
                    id: job_id,
                    repo_config_id: config_id,
                    job_type: "code".to_string(),
                    status: "pending".to_string(),
                    items_total: 0,
                    items_processed: 0,
                    created_at: Utc::now(),
                });
                sync_jobs_created.push(format!("code:{}", job_id));
            }
            
            if repo_input.sync_issues {
                let job_id = Uuid::new_v4();
                sync_jobs.insert(job_id, SyncJob {
                    id: job_id,
                    repo_config_id: config_id,
                    job_type: "issues".to_string(),
                    status: "pending".to_string(),
                    items_total: 0,
                    items_processed: 0,
                    created_at: Utc::now(),
                });
                sync_jobs_created.push(format!("issues:{}", job_id));
            }
            
            if repo_input.sync_prs {
                let job_id = Uuid::new_v4();
                sync_jobs.insert(job_id, SyncJob {
                    id: job_id,
                    repo_config_id: config_id,
                    job_type: "prs".to_string(),
                    status: "pending".to_string(),
                    items_total: 0,
                    items_processed: 0,
                    created_at: Utc::now(),
                });
                sync_jobs_created.push(format!("prs:{}", job_id));
            }
        }
    }
    
    info!("✅ Saved {} repo configs, created {} sync jobs", configs_saved, sync_jobs_created.len());
    
    Ok(HttpResponse::Ok().json(SaveRepoConfigResponse {
        success: true,
        repos_configured: configs_saved,
        sync_jobs_created,
        message: format!("Configured {} repositories", configs_saved),
    }))
}

/// Get selected repos with sync status
/// GET /api/connectors/github/app/{installation_id}/repos/selected
pub async fn get_selected_repos(
    state: web::Data<Arc<GitHubAppState>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let installation_id = path.into_inner();
    
    let repo_configs = state.repo_configs.read().await;
    let sync_jobs = state.sync_jobs.read().await;
    
    let selected: Vec<serde_json::Value> = repo_configs
        .values()
        .filter(|r| r.installation_id == installation_id)
        .map(|r| {
            // Get latest job status for each type
            let code_job = sync_jobs.values()
                .filter(|j| j.repo_config_id == r.id && j.job_type == "code")
                .max_by_key(|j| j.created_at);
            let issues_job = sync_jobs.values()
                .filter(|j| j.repo_config_id == r.id && j.job_type == "issues")
                .max_by_key(|j| j.created_at);
            let prs_job = sync_jobs.values()
                .filter(|j| j.repo_config_id == r.id && j.job_type == "prs")
                .max_by_key(|j| j.created_at);
            
            serde_json::json!({
                "id": r.id,
                "github_repo_id": r.github_repo_id,
                "full_name": r.full_name,
                "name": r.name,
                "owner": r.owner,
                "default_branch": r.default_branch,
                "sync_code": r.sync_code,
                "sync_issues": r.sync_issues,
                "sync_prs": r.sync_prs,
                "code_sync_status": code_job.map(|j| &j.status),
                "issues_sync_status": issues_job.map(|j| &j.status),
                "prs_sync_status": prs_job.map(|j| &j.status),
            })
        })
        .collect();
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "repos": selected
    })))
}

/// Trigger sync for a repo
/// POST /api/connectors/github/app/repos/{repo_config_id}/sync
pub async fn trigger_repo_sync(
    state: web::Data<Arc<GitHubAppState>>,
    path: web::Path<Uuid>,
    body: web::Json<TriggerSyncRequest>,
) -> Result<HttpResponse> {
    let repo_config_id = path.into_inner();
    
    // Verify repo config exists
    {
        let repo_configs = state.repo_configs.read().await;
        if !repo_configs.contains_key(&repo_config_id) {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "error": "Repository configuration not found"
            })));
        }
    }
    
    let job_id = Uuid::new_v4();
    
    {
        let mut sync_jobs = state.sync_jobs.write().await;
        sync_jobs.insert(job_id, SyncJob {
            id: job_id,
            repo_config_id,
            job_type: body.job_type.clone(),
            status: "pending".to_string(),
            items_total: 0,
            items_processed: 0,
            created_at: Utc::now(),
        });
    }
    
    info!("📋 Created sync job {} for repo {} (type: {})", job_id, repo_config_id, body.job_type);
    
    Ok(HttpResponse::Ok().json(TriggerSyncResponse {
        success: true,
        job_id: Some(job_id),
        message: format!("Sync job created: {}", job_id),
    }))
}

/// Get sync job status
/// GET /api/connectors/github/app/jobs/{job_id}
pub async fn get_sync_job_status(
    state: web::Data<Arc<GitHubAppState>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let job_id = path.into_inner();
    
    let sync_jobs = state.sync_jobs.read().await;
    
    match sync_jobs.get(&job_id) {
        Some(job) => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "job": {
                    "id": job.id,
                    "repo_config_id": job.repo_config_id,
                    "job_type": job.job_type,
                    "status": job.status,
                    "items_total": job.items_total,
                    "items_processed": job.items_processed,
                    "created_at": job.created_at.to_rfc3339(),
                }
            })))
        }
        None => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "error": "Job not found"
            })))
        }
    }
}

/// Execute a sync job (actually run the sync)
/// POST /api/connectors/github/app/jobs/{job_id}/execute
pub async fn execute_sync_job(
    state: web::Data<Arc<GitHubAppState>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let job_id = path.into_inner();
    
    // Get job details
    let (repo_config, job_type, github_installation_id) = {
        let sync_jobs = state.sync_jobs.read().await;
        let repo_configs = state.repo_configs.read().await;
        let installations = state.installations.read().await;
        
        let job = match sync_jobs.get(&job_id) {
            Some(j) => j.clone(),
            None => {
                return Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "success": false,
                    "error": "Job not found"
                })));
            }
        };
        
        let config = match repo_configs.get(&job.repo_config_id) {
            Some(c) => c.clone(),
            None => {
                return Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "success": false,
                    "error": "Repository configuration not found"
                })));
            }
        };
        
        let installation = match installations.get(&config.installation_id) {
            Some(i) => i.clone(),
            None => {
                return Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "success": false,
                    "error": "Installation not found"
                })));
            }
        };
        
        (config, job.job_type, installation.github_installation_id)
    };
    
    // Update job status to running
    {
        let mut sync_jobs = state.sync_jobs.write().await;
        if let Some(job) = sync_jobs.get_mut(&job_id) {
            job.status = "running".to_string();
        }
    }
    
    info!("🚀 Executing sync job {} for {}/{} (type: {})", 
          job_id, repo_config.owner, repo_config.name, job_type);
    
    // Get chunker URL from environment
    let chunker_url = std::env::var("CHUNKER_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:3017".to_string());
    
    // Create ingestion service
    let ingestion_service = crate::services::GitHubIngestionService::new(
        Arc::new(state.client.clone()),
        chunker_url,
    );
    
    // Execute based on job type
    let result = match job_type.as_str() {
        "code" => {
            let config = crate::services::CodeSyncConfig {
                branch: repo_config.default_branch.clone(),
                ..Default::default()
            };
            
            ingestion_service.sync_code(
                github_installation_id,
                &repo_config.owner,
                &repo_config.name,
                &config,
                None,
            ).await
        }
        "issues" => {
            let config = crate::services::IssuesSyncConfig::default();
            
            ingestion_service.sync_issues(
                github_installation_id,
                &repo_config.owner,
                &repo_config.name,
                &config,
                None,
            ).await
        }
        "prs" => {
            let config = crate::services::PrsSyncConfig::default();
            
            ingestion_service.sync_prs(
                github_installation_id,
                &repo_config.owner,
                &repo_config.name,
                &config,
                None,
            ).await
        }
        _ => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": format!("Unknown job type: {}", job_type)
            })));
        }
    };
    
    // Update job status based on result
    match result {
        Ok(progress) => {
            {
                let mut sync_jobs = state.sync_jobs.write().await;
                if let Some(job) = sync_jobs.get_mut(&job_id) {
                    job.status = "completed".to_string();
                    job.items_total = progress.items_total;
                    job.items_processed = progress.items_processed;
                }
            }
            
            info!("✅ Sync job {} completed: {} items processed, {} chunks created",
                  job_id, progress.items_processed, progress.chunks_created);
            
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "job_id": job_id,
                "status": "completed",
                "items_total": progress.items_total,
                "items_processed": progress.items_processed,
                "items_failed": progress.items_failed,
                "chunks_created": progress.chunks_created,
                "errors": progress.errors
            })))
        }
        Err(e) => {
            {
                let mut sync_jobs = state.sync_jobs.write().await;
                if let Some(job) = sync_jobs.get_mut(&job_id) {
                    job.status = "failed".to_string();
                }
            }
            
            error!("❌ Sync job {} failed: {}", job_id, e);
            
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "job_id": job_id,
                "status": "failed",
                "error": e.to_string()
            })))
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn get_tenant_id_from_request(req: &HttpRequest) -> Uuid {
    // In production, extract from JWT claims
    // For now, use a default or header-based approach
    req.headers()
        .get("X-Tenant-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_else(Uuid::nil)
}

// ============================================================================
// Route Configuration
// ============================================================================

pub fn configure_github_app_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/connectors/github/app")
            .route("/install-url", web::get().to(get_install_url))
            .route("/callback", web::get().to(handle_install_callback))
            .route("/installations", web::get().to(list_installations))
            .route("/{installation_id}/repos", web::get().to(list_installation_repos))
            .route("/{installation_id}/repos", web::post().to(save_repo_configs))
            .route("/{installation_id}/repos/selected", web::get().to(get_selected_repos))
            .route("/repos/{repo_config_id}/sync", web::post().to(trigger_repo_sync))
            .route("/jobs/{job_id}", web::get().to(get_sync_job_status))
            .route("/jobs/{job_id}/execute", web::post().to(execute_sync_job))
    );
}
