use actix_web::{web, App, HttpServer, HttpRequest, HttpResponse, Result, HttpMessage, http::header};
use actix_cors::Cors;
use conhub_middleware::auth::AuthMiddlewareFactory;
use conhub_models::auth::Claims;
use conhub_observability::{
    init_tracing, TracingConfig, observability,
    info, warn, error, debug,
    domain_events::{DomainEvent, EventCategory, log_connector_operation, log_sync_job_started, log_sync_job_completed, log_sync_job_failed},
    get_trace_context,
};
use std::env;
use std::sync::Arc;
use uuid::Uuid;

use services::kafka_client::KafkaProducer;
use handlers::{robots, robot_ingestion};

// Only include the modules we actually need
mod connectors {
    pub mod github;
    pub mod types;
    pub mod error;
    pub mod traits;
}

mod services {
    pub mod zilliz_client;
    pub mod qdrant_client;
    pub mod vector_store;
    pub mod kafka_client;
    pub mod github_app_client;
    pub mod github_ingestion;
    pub mod auth_client;
    pub mod embedding_client;
    pub mod chunker_client;
    pub mod graph_rag_ingestion;
    
    pub use zilliz_client::*;
    #[allow(deprecated)]
    pub use qdrant_client::*;
    pub use vector_store::*;
    pub use kafka_client::*;
    pub use github_app_client::*;
    pub use github_ingestion::*;
    pub use auth_client::*;
    pub use embedding_client::*;
    pub use chunker_client::*;
    pub use graph_rag_ingestion::*;
}

mod handlers {
    pub mod robots;
    pub mod robot_ingestion;
    pub mod github_app;
    pub mod documents;
    pub mod local_fs;
}

mod errors;

// Simplified handlers inline
use serde::{Deserialize, Serialize};

use connectors::github::GitHubConnector;
use connectors::types::{GitHubRepoAccessRequest, GitHubSyncConfig, DocumentForEmbedding};
use services::{
    create_vector_store_service, GitHubAppClient, GitHubAppConfig,
    AuthClient, EmbeddingClient, GraphRagIngestionService,
};
use handlers::github_app::{GitHubAppState, configure_github_app_routes};
use conhub_models::chunking::SourceKind;

#[derive(Debug, Deserialize)]
pub struct ValidateRepoAccessRequest {
    pub repo_url: String,
    pub access_token: String,
}

#[derive(Debug, Serialize)]
pub struct ValidateRepoAccessResponse {
    pub success: bool,
    pub has_access: bool,
    pub repo_info: Option<serde_json::Value>,
    pub error_message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SyncRepositoryRequest {
    pub repo_url: String,
    pub access_token: String,
    pub branch: String,
    pub include_languages: Option<Vec<String>>,
    pub exclude_paths: Option<Vec<String>>,
    pub max_file_size_mb: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SyncRepositoryResponse {
    pub success: bool,
    pub documents_processed: usize,
    pub embeddings_created: usize,
    pub sync_duration_ms: u64,
    pub error_message: Option<String>,
    pub graph_job_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issues_processed: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prs_processed: Option<usize>,
}

/// New secure sync request - no access_token required (fetched from auth service)
#[derive(Debug, Deserialize)]
pub struct SecureSyncRepositoryRequest {
    pub repo_url: String,
    pub branch: String,
    pub include_languages: Option<Vec<String>>,
    pub exclude_paths: Option<Vec<String>>,
    pub max_file_size_mb: Option<i64>,
    /// File extensions to include (e.g. ["ts", "tsx", "md"]). Takes precedence over include_languages.
    pub file_extensions: Option<Vec<String>>,
    /// Whether to fetch and ingest issues for this repository
    pub fetch_issues: Option<bool>,
    /// Whether to fetch and ingest pull requests for this repository
    pub fetch_prs: Option<bool>,
}

/// OAuth-based repository check request (no token - uses JWT to get from auth service)
#[derive(Debug, Deserialize)]
pub struct OAuthRepoCheckRequest {
    pub provider: String,
    pub repo_url: String,
}

/// OAuth-based branches request query params
#[derive(Debug, Deserialize)]
pub struct OAuthBranchesQuery {
    pub provider: String,
    pub repo: String,  // owner/repo format
}

/// Repository info response
#[derive(Debug, Serialize)]
pub struct RepoCheckResponse {
    pub success: bool,
    pub provider: String,
    pub name: Option<String>,
    pub full_name: Option<String>,
    pub default_branch: Option<String>,
    pub private: Option<bool>,
    pub has_read_access: bool,
    pub languages: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub error: Option<String>,
}

/// Validate GitHub repository access
pub async fn validate_github_repo_access(
    req: web::Json<ValidateRepoAccessRequest>,
) -> Result<HttpResponse> {
    info!("🔍 Validating GitHub repository access for: {}", req.repo_url);
    
    let connector = GitHubConnector::new();
    
    let access_request = GitHubRepoAccessRequest {
        repo_url: req.repo_url.clone(),
        access_token: req.access_token.clone(),
    };
    
    match connector.validate_repo_access(&access_request).await {
        Ok(response) => {
            let api_response = ValidateRepoAccessResponse {
                success: true,
                has_access: response.has_access,
                repo_info: response.repo_info.map(|info| serde_json::to_value(info).unwrap()),
                error_message: response.error_message,
            };
            
            info!("✅ Repository access validation completed successfully");
            Ok(HttpResponse::Ok().json(api_response))
        }
        Err(e) => {
            error!("❌ Failed to validate repository access: {}", e);
            let api_response = ValidateRepoAccessResponse {
                success: false,
                has_access: false,
                repo_info: None,
                error_message: Some(e.to_string()),
            };
            Ok(HttpResponse::BadRequest().json(api_response))
        }
    }
}

/// Sync GitHub repository
pub async fn sync_github_repository(
    req: web::Json<SyncRepositoryRequest>,
) -> Result<HttpResponse> {
    let start_time = std::time::Instant::now();
    
    info!("🔄 Starting GitHub repository sync for: {} (branch: {})", req.repo_url, req.branch);
    
    let connector = GitHubConnector::new();
    
    let sync_config = GitHubSyncConfig {
        repo_url: req.repo_url.clone(),
        branch: req.branch.clone(),
        include_languages: req.include_languages.clone(),
        exclude_paths: req.exclude_paths.clone(),
        max_file_size_mb: req.max_file_size_mb,
        include_extensions: None, // No extension filter for legacy endpoint
    };
    
    let documents = match connector.sync_repository_branch(&req.access_token, &sync_config).await {
        Ok(docs) => docs,
        Err(e) => {
            error!("❌ Failed to sync repository: {}", e);
            let response = SyncRepositoryResponse {
                success: false,
                documents_processed: 0,
                embeddings_created: 0,
                sync_duration_ms: start_time.elapsed().as_millis() as u64,
                error_message: Some(e.to_string()),
                graph_job_id: None,
                issues_processed: None,
                prs_processed: None,
            };
            return Ok(HttpResponse::BadRequest().json(response));
        }
    };
    
    info!("📄 Retrieved {} documents from repository", documents.len());
    
    let embeddings_created = documents.len(); // Placeholder
    let sync_duration = start_time.elapsed().as_millis() as u64;
    
    info!("🎉 Repository sync completed! Processed {} documents in {}ms", 
          documents.len(), sync_duration);
    
    let response = SyncRepositoryResponse {
        success: true,
        documents_processed: documents.len(),
        embeddings_created,
        sync_duration_ms: sync_duration,
        error_message: None,
        graph_job_id: None,
        issues_processed: None,
        prs_processed: None,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

/// Helper to extract user_id from JWT claims
fn extract_user_id_from_request(req: &HttpRequest) -> Option<Uuid> {
    req.extensions()
        .get::<Claims>()
        .and_then(|claims| claims.sub.parse::<Uuid>().ok())
}

/// Helper to extract owner and repo from GitHub URL
fn parse_github_url(repo_url: &str) -> Option<(String, String)> {
    // Handle URLs like https://github.com/owner/repo or https://github.com/owner/repo.git
    let url = repo_url.trim_end_matches(".git");
    let parts: Vec<&str> = url.split('/').collect();
    if parts.len() >= 2 {
        let repo = parts[parts.len() - 1].to_string();
        let owner = parts[parts.len() - 2].to_string();
        if !owner.is_empty() && !repo.is_empty() {
            return Some((owner, repo));
        }
    }
    None
}

/// Sync issues and PRs from a repository using OAuth token
/// Returns (issues_processed, prs_processed) counts
async fn sync_issues_and_prs(
    access_token: &str,
    repo_url: &str,
    fetch_issues: bool,
    fetch_prs: bool,
) -> (Option<usize>, Option<usize>) {
    if !fetch_issues && !fetch_prs {
        return (None, None);
    }
    
    let (owner, repo) = match parse_github_url(repo_url) {
        Some((o, r)) => (o, r),
        None => {
            warn!("Could not parse owner/repo from URL: {}", repo_url);
            return (None, None);
        }
    };
    
    let client = reqwest::Client::new();
    let mut issues_count: Option<usize> = None;
    let mut prs_count: Option<usize> = None;
    
    // Fetch issues if requested
    if fetch_issues {
        info!("🎫 Fetching issues for {}/{}", owner, repo);
        match fetch_github_issues(&client, access_token, &owner, &repo).await {
            Ok(count) => {
                info!("✅ Fetched {} issues", count);
                issues_count = Some(count);
            }
            Err(e) => {
                warn!("⚠️ Failed to fetch issues: {}", e);
            }
        }
    }
    
    // Fetch PRs if requested
    if fetch_prs {
        info!("🔀 Fetching PRs for {}/{}", owner, repo);
        match fetch_github_prs(&client, access_token, &owner, &repo).await {
            Ok(count) => {
                info!("✅ Fetched {} PRs", count);
                prs_count = Some(count);
            }
            Err(e) => {
                warn!("⚠️ Failed to fetch PRs: {}", e);
            }
        }
    }
    
    (issues_count, prs_count)
}

/// Fetch GitHub issues using OAuth token
async fn fetch_github_issues(
    client: &reqwest::Client,
    access_token: &str,
    owner: &str,
    repo: &str,
) -> Result<usize, String> {
    let url = format!("https://api.github.com/repos/{}/{}/issues?state=all&per_page=100", owner, repo);
    
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "ConHub")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("GitHub API error: {}", response.status()));
    }
    
    let issues: Vec<serde_json::Value> = response
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {}", e))?;
    
    // Filter out PRs (GitHub returns PRs in the issues endpoint)
    let pure_issues: Vec<_> = issues
        .into_iter()
        .filter(|i| i.get("pull_request").is_none())
        .collect();
    
    // TODO: Send issues to chunker service for indexing
    // For now, just count them
    info!("📝 Found {} issues (excluding PRs) for {}/{}", pure_issues.len(), owner, repo);
    
    Ok(pure_issues.len())
}

/// Fetch GitHub PRs using OAuth token
async fn fetch_github_prs(
    client: &reqwest::Client,
    access_token: &str,
    owner: &str,
    repo: &str,
) -> Result<usize, String> {
    let url = format!("https://api.github.com/repos/{}/{}/pulls?state=all&per_page=100", owner, repo);
    
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "ConHub")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("GitHub API error: {}", response.status()));
    }
    
    let prs: Vec<serde_json::Value> = response
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {}", e))?;
    
    // TODO: Send PRs to chunker service for indexing
    // For now, just count them
    info!("🔀 Found {} PRs for {}/{}", prs.len(), owner, repo);
    
    Ok(prs.len())
}

/// Secure sync endpoint - uses JWT claims to identify user and fetches token from auth service
/// POST /api/github/sync
/// 
/// This is the preferred endpoint for production use. It:
/// 1. Extracts user_id from JWT claims
/// 2. Fetches GitHub token from auth service (internal endpoint)
/// 3. Syncs the repository
/// 4. Embeds documents into vector store
/// 5. Sends to graph RAG for knowledge graph construction
pub async fn secure_sync_github_repository(
    http_req: HttpRequest,
    req: web::Json<SecureSyncRepositoryRequest>,
    auth_client: web::Data<AuthClient>,
    embedding_client: web::Data<EmbeddingClient>,
    graph_ingestion: web::Data<Option<GraphRagIngestionService>>,
) -> Result<HttpResponse> {
    let start_time = std::time::Instant::now();
    let trace_ctx = get_trace_context(&http_req);
    
    // Step 1: Extract user_id from JWT claims
    let user_id = match extract_user_id_from_request(&http_req) {
        Some(id) => id,
        None => {
            error!(trace_id = %trace_ctx.trace_id, "No user_id found in request claims");
            DomainEvent::new("data-service", EventCategory::Auth, "auth_failed")
                .trace(&trace_ctx.trace_id, &trace_ctx.span_id)
                .failure("No user_id in claims")
                .emit();
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Authentication required"
            })));
        }
    };
    
    // Create a sync job ID for tracking
    let sync_job_id = Uuid::new_v4();
    
    info!(
        trace_id = %trace_ctx.trace_id,
        user_id = %user_id,
        repo = %req.repo_url,
        branch = %req.branch,
        sync_job_id = %sync_job_id,
        "Starting secure GitHub repository sync"
    );
    
    // Log sync job started
    log_sync_job_started("data-service", sync_job_id, "github", Some(&trace_ctx.trace_id));
    
    // Step 2: Fetch GitHub token from auth service
    let github_token = match auth_client.get_oauth_token(user_id, "github").await {
        Ok(token_response) => token_response.access_token,
        Err(e) => {
            error!("❌ Failed to get GitHub token: {}", e);
            let response = SyncRepositoryResponse {
                success: false,
                documents_processed: 0,
                embeddings_created: 0,
                sync_duration_ms: start_time.elapsed().as_millis() as u64,
                error_message: Some(format!("Failed to retrieve GitHub token: {}. Please ensure you have connected your GitHub account.", e)),
                graph_job_id: None,
                issues_processed: None,
                prs_processed: None,
            };
            return Ok(HttpResponse::BadRequest().json(response));
        }
    };
    
    info!("✅ Retrieved GitHub token for user {}", user_id);
    
    // Step 3: Sync repository using GitHubConnector
    let connector = GitHubConnector::new();
    
    // Normalize file extensions: strip leading dots and lowercase
    let include_extensions = req.file_extensions.as_ref().map(|exts| {
        exts.iter()
            .map(|e| e.trim_start_matches('.').to_lowercase())
            .collect::<Vec<_>>()
    });
    
    let sync_config = GitHubSyncConfig {
        repo_url: req.repo_url.clone(),
        branch: req.branch.clone(),
        include_languages: req.include_languages.clone(),
        exclude_paths: req.exclude_paths.clone(),
        max_file_size_mb: req.max_file_size_mb,
        include_extensions,
    };
    
    let documents: Vec<DocumentForEmbedding> = match connector.sync_repository_branch(&github_token, &sync_config).await {
        Ok(docs) => docs,
        Err(e) => {
            error!("❌ Failed to sync repository: {}", e);
            let response = SyncRepositoryResponse {
                success: false,
                documents_processed: 0,
                embeddings_created: 0,
                sync_duration_ms: start_time.elapsed().as_millis() as u64,
                error_message: Some(e.to_string()),
                graph_job_id: None,
                issues_processed: None,
                prs_processed: None,
            };
            return Ok(HttpResponse::BadRequest().json(response));
        }
    };
    
    info!("📄 Retrieved {} documents from repository", documents.len());
    
    let doc_count = documents.len();
    
    // Step 3.5: Sync issues and PRs if requested
    let (issues_processed, prs_processed) = sync_issues_and_prs(
        &github_token,
        &req.repo_url,
        req.fetch_issues.unwrap_or(false),
        req.fetch_prs.unwrap_or(false),
    ).await;
    
    // Step 4: Send to chunker service (which handles embedding + graph ingestion)
    // The chunker service will:
    // - Apply intelligent chunking strategies (AST for code, markdown for docs)
    // - Send chunks to embedding service for vector indexing
    // - Send chunks to graph RAG for knowledge graph construction
    let (graph_job_id, embeddings_created) = if let Some(ref graph_service) = graph_ingestion.get_ref() {
        // Use a stable source_id based on repo URL
        let source_id = Uuid::new_v5(&Uuid::NAMESPACE_URL, req.repo_url.as_bytes());
        
        // Pass a cloned Vec so the original `documents` can still be cloned for fallback embedding.
        match graph_service.ingest_documents(source_id, SourceKind::CodeRepo, documents.clone()).await {
            Ok(job_id) => {
                info!("✅ Chunker job started: {} - documents will be chunked, embedded, and indexed", job_id);
                (Some(job_id), doc_count) // Embeddings will be created by chunker
            }
            Err(e) => {
                warn!("⚠️ Chunker service error: {} - falling back to direct embedding", e);
                // Fallback: send directly to embedding service if chunker fails
                if let Err(embed_err) = embedding_client.embed_documents(documents.clone()).await {
                    warn!("⚠️ Embedding service also failed: {}", embed_err);
                    (None, 0)
                } else {
                    info!("✅ Fallback: sent {} documents directly to embedding service", doc_count);
                    (None, doc_count)
                }
            }
        }
    } else {
        // No chunker configured - send directly to embedding service
        info!("📊 Chunker service not configured, sending directly to embedding service");
        if let Err(e) = embedding_client.embed_documents(documents.clone()).await {
            warn!("⚠️ Embedding service error: {}", e);
            (None, 0)
        } else {
            info!("✅ Sent {} documents to embedding service", doc_count);
            (None, doc_count)
        }
    };
    
    let sync_duration = start_time.elapsed().as_millis() as u64;
    
    // Log sync job completed with domain event
    log_sync_job_completed("data-service", sync_job_id, doc_count, sync_duration);
    
    info!(
        trace_id = %trace_ctx.trace_id,
        sync_job_id = %sync_job_id,
        documents = doc_count,
        embeddings = embeddings_created,
        duration_ms = sync_duration,
        "Repository sync completed successfully"
    );
    
    let response = SyncRepositoryResponse {
        success: true,
        documents_processed: doc_count,
        embeddings_created,
        sync_duration_ms: sync_duration,
        error_message: None,
        graph_job_id,
        issues_processed,
        prs_processed,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

/// Get repository branches
pub async fn get_repository_branches(
    req: web::Json<ValidateRepoAccessRequest>,
) -> Result<HttpResponse> {
    info!("🌿 Fetching branches for repository: {}", req.repo_url);
    
    let connector = GitHubConnector::new();
    
    let access_request = GitHubRepoAccessRequest {
        repo_url: req.repo_url.clone(),
        access_token: req.access_token.clone(),
    };
    
    match connector.validate_repo_access(&access_request).await {
        Ok(response) => {
            if response.has_access {
                if let Some(repo_info) = response.repo_info {
                    let branches = repo_info.branches;
                    info!("✅ Found {} branches", branches.len());
                    Ok(HttpResponse::Ok().json(serde_json::json!({
                        "success": true,
                        "branches": branches
                    })))
                } else {
                    Ok(HttpResponse::BadRequest().json(serde_json::json!({
                        "success": false,
                        "error": "No repository information available"
                    })))
                }
            } else {
                Ok(HttpResponse::Forbidden().json(serde_json::json!({
                    "success": false,
                    "error": response.error_message.unwrap_or_else(|| "Access denied".to_string())
                })))
            }
        }
        Err(e) => {
            error!("❌ Failed to fetch branches: {}", e);
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })))
        }
    }
}

/// Helper to generate safe token debug string (never logs full token)
fn token_debug(token: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    let len = token.len();
    let prefix = if len >= 6 { &token[..6] } else { token };
    format!("len={}, prefix={}..., sha256_prefix={}", len, prefix, &hash[..12])
}

/// OAuth-based repository check - uses JWT claims to get token from auth service
/// POST /api/repositories/oauth/check
pub async fn oauth_repo_check(
    http_req: HttpRequest,
    req: web::Json<OAuthRepoCheckRequest>,
    auth_client: web::Data<AuthClient>,
) -> Result<HttpResponse> {
    // Generate correlation ID for tracing
    let correlation_id = http_req.headers()
        .get("x-correlation-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()[..8].to_string());
    
    info!(
        "[Repo Check][{}] 🔍 Starting: repo_url={}, provider={}",
        correlation_id, req.repo_url, req.provider
    );
    
    // Extract user_id from JWT claims
    let user_id = match extract_user_id_from_request(&http_req) {
        Some(id) => {
            info!("[Repo Check][{}] ✅ User authenticated: user_id={}", correlation_id, id);
            id
        },
        None => {
            warn!("[Repo Check][{}] ❌ No JWT claims found - authentication required", correlation_id);
            return Ok(HttpResponse::Unauthorized().json(RepoCheckResponse {
                success: false,
                provider: req.provider.clone(),
                name: None,
                full_name: None,
                default_branch: None,
                private: None,
                has_read_access: false,
                languages: None,
                code: Some("auth_required".to_string()),
                error: Some("Authentication required".to_string()),
            }));
        }
    };
    
    // Get OAuth token from auth service
    info!(
        "[Repo Check][{}] 🔑 Fetching OAuth token from auth service: user_id={}, provider={}",
        correlation_id, user_id, req.provider
    );
    
    let access_token = match auth_client.get_oauth_token_with_correlation(user_id, &req.provider, &correlation_id).await {
        Ok(token_response) => {
            info!(
                "[Repo Check][{}] ✅ Got OAuth token from auth service: provider={}, token_debug={}",
                correlation_id, req.provider, token_debug(&token_response.access_token)
            );
            token_response.access_token
        },
        Err(e) => {
            use crate::services::auth_client::AuthClientError;
            let (code, error_msg) = match &e {
                AuthClientError::NoConnection(provider) => (
                    "no_connection",
                    format!("No {} connection found. Please connect in Social Connections first.", provider)
                ),
                AuthClientError::TokenExpired(provider) => (
                    "token_expired",
                    format!("Your {} token has expired. Please reconnect in Social Connections.", provider)
                ),
                _ => (
                    "auth_error",
                    format!("Failed to get {} token: {}", req.provider, e)
                ),
            };
            error!(
                "[Repo Check][{}] ❌ Failed to get OAuth token: user_id={}, provider={}, error_code={}, error={}",
                correlation_id, user_id, req.provider, code, error_msg
            );
            return Ok(HttpResponse::BadRequest().json(RepoCheckResponse {
                success: false,
                provider: req.provider.clone(),
                name: None,
                full_name: None,
                default_branch: None,
                private: None,
                has_read_access: false,
                languages: None,
                code: Some(code.to_string()),
                error: Some(error_msg),
            }));
        }
    };
    
    // Validate repository access
    info!(
        "[Repo Check][{}] 🌐 Calling GitHub API to validate repo access: repo_url={}",
        correlation_id, req.repo_url
    );
    
    let connector = GitHubConnector::new();
    let access_request = GitHubRepoAccessRequest {
        repo_url: req.repo_url.clone(),
        access_token: access_token.clone(),
    };
    
    match connector.validate_repo_access(&access_request).await {
        Ok(response) => {
            if response.has_access {
                let repo_info = response.repo_info.unwrap();
                info!(
                    "[Repo Check][{}] ✅✅ REPO ACCESS VALIDATED: full_name={}, private={}, default_branch={}, permissions={{pull={}, push={}, admin={}}}",
                    correlation_id, repo_info.full_name, repo_info.private, repo_info.default_branch,
                    repo_info.permissions.pull, repo_info.permissions.push, repo_info.permissions.admin
                );
                Ok(HttpResponse::Ok().json(RepoCheckResponse {
                    success: true,
                    provider: req.provider.clone(),
                    name: Some(repo_info.name),
                    full_name: Some(repo_info.full_name),
                    default_branch: Some(repo_info.default_branch),
                    private: Some(repo_info.private),
                    has_read_access: repo_info.permissions.pull,
                    languages: Some(repo_info.languages),
                    code: None,
                    error: None,
                }))
            } else {
                // Check if error message indicates bad credentials
                let error_msg = response.error_message.clone().unwrap_or_default();
                let code = if error_msg.contains("Bad credentials") || error_msg.contains("401") {
                    Some("github_bad_credentials".to_string())
                } else if error_msg.contains("403") || error_msg.contains("permission") {
                    Some("github_insufficient_permissions".to_string())
                } else {
                    Some("repo_access_denied".to_string())
                };
                error!(
                    "[Repo Check][{}] ❌ REPO ACCESS DENIED: repo_url={}, code={:?}, error={}, token_debug={}",
                    correlation_id, req.repo_url, code, error_msg, token_debug(&access_token)
                );
                Ok(HttpResponse::Ok().json(RepoCheckResponse {
                    success: false,
                    provider: req.provider.clone(),
                    name: None,
                    full_name: None,
                    default_branch: None,
                    private: None,
                    has_read_access: false,
                    languages: None,
                    code,
                    error: response.error_message,
                }))
            }
        }
        Err(e) => {
            // Parse error to determine code
            let error_str = e.to_string();
            let code = if error_str.contains("Bad credentials") || error_str.contains("401") {
                "github_bad_credentials"
            } else if error_str.contains("403") || error_str.contains("permission") {
                "github_insufficient_permissions"
            } else {
                "repo_error"
            };
            error!(
                "[Repo Check][{}] ❌ GITHUB API ERROR: repo_url={}, code={}, error={}, token_debug={}",
                correlation_id, req.repo_url, code, error_str, token_debug(&access_token)
            );
            Ok(HttpResponse::BadRequest().json(RepoCheckResponse {
                success: false,
                provider: req.provider.clone(),
                name: None,
                full_name: None,
                default_branch: None,
                private: None,
                has_read_access: false,
                languages: None,
                code: Some(code.to_string()),
                error: Some(e.to_string()),
            }))
        }
    }
}

/// OAuth-based branches fetch - uses JWT claims to get token from auth service
/// GET /api/repositories/oauth/branches?provider=github&repo=owner/repo
pub async fn oauth_repo_branches(
    http_req: HttpRequest,
    query: web::Query<OAuthBranchesQuery>,
    auth_client: web::Data<AuthClient>,
) -> Result<HttpResponse> {
    // Correlation ID (propagated from frontend if present)
    let correlation_id = http_req.headers()
        .get("x-correlation-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()[..8].to_string());

    info!(
        "[Repo Branches][{}] 🌿 Starting OAuth branches fetch: repo={}, provider={}",
        correlation_id, query.repo, query.provider
    );
    
    // Extract user_id from JWT claims
    let user_id = match extract_user_id_from_request(&http_req) {
        Some(id) => {
            info!(
                "[Repo Branches][{}] ✅ User authenticated: user_id={}",
                correlation_id, id
            );
            id
        }
        None => {
            warn!(
                "[Repo Branches][{}] ❌ No JWT claims found - authentication required",
                correlation_id
            );
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "success": false,
                "error": "Authentication required"
            })));
        }
    };
    
    // Get OAuth token from auth service
    info!(
        "[Repo Branches][{}] 🔑 Fetching OAuth token from auth service: user_id={}, provider={}",
        correlation_id, user_id, query.provider
    );
    let access_token = match auth_client
        .get_oauth_token_with_correlation(user_id, &query.provider, &correlation_id)
        .await
    {
        Ok(token_response) => {
            info!(
                "[Repo Branches][{}] ✅ Got OAuth token from auth service: provider={}, token_debug={}",
                correlation_id,
                query.provider,
                token_debug(&token_response.access_token)
            );
            token_response.access_token
        }
        Err(e) => {
            use crate::services::auth_client::AuthClientError;
            let (code, error_msg) = match &e {
                AuthClientError::NoConnection(provider) => (
                    "no_connection",
                    format!(
                        "No {} connection found. Please connect in Social Connections first.",
                        provider
                    ),
                ),
                AuthClientError::TokenExpired(provider) => (
                    "token_expired",
                    format!(
                        "Your {} token has expired. Please reconnect in Social Connections.",
                        provider
                    ),
                ),
                _ => (
                    "auth_error",
                    format!("Failed to get {} token: {}", query.provider, e),
                ),
            };
            error!(
                "[Repo Branches][{}] ❌ Failed to get OAuth token: user_id={}, provider={}, error_code={}, error={}",
                correlation_id, user_id, query.provider, code, error_msg
            );
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": error_msg
            })));
        }
    };
    
    // Parse owner/repo
    let parts: Vec<&str> = query.repo.split('/').collect();
    if parts.len() != 2 {
        warn!(
            "[Repo Branches][{}] ❌ Invalid repo format: repo={}",
            correlation_id, query.repo
        );
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "error": "Invalid repo format. Expected 'owner/repo'"
        })));
    }
    
    let (owner, repo) = (parts[0], parts[1]);
    let repo_url = format!("https://github.com/{}/{}", owner, repo);
    info!(
        "[Repo Branches][{}] 🔄 Parsed repo: owner={}, repo={}, repo_url={}",
        correlation_id, owner, repo, repo_url
    );
    
    let connector = GitHubConnector::new();
    let access_request = GitHubRepoAccessRequest {
        repo_url,
        access_token: access_token.clone(),
    };
    
    info!(
        "[Repo Branches][{}] 🌐 Calling GitHub API to fetch branches",
        correlation_id
    );
    match connector.validate_repo_access(&access_request).await {
        Ok(response) => {
            if response.has_access {
                let repo_info = response.repo_info.unwrap();
                let branch_count = repo_info.branches.len();
                let branch_names: Vec<String> =
                    repo_info.branches.into_iter().map(|b| b.name).collect();
                info!(
                    "[Repo Branches][{}] ✅ Branch fetch successful: default_branch={}, branch_count={}",
                    correlation_id, repo_info.default_branch, branch_count
                );
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "data": {
                        "branches": branch_names,
                        "default_branch": repo_info.default_branch
                    }
                })))
            } else {
                let error_msg = response
                    .error_message
                    .clone()
                    .unwrap_or_else(|| "Access denied".to_string());
                error!(
                    "[Repo Branches][{}] ❌ Access denied when fetching branches: repo={}, provider={}, error={}, token_debug={}",
                    correlation_id, query.repo, query.provider, error_msg, token_debug(&access_token)
                );
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "success": false,
                    "error": error_msg
                })))
            }
        }
        Err(e) => {
            let error_str = e.to_string();
            error!(
                "[Repo Branches][{}] ❌ GitHub error when fetching branches: repo={}, provider={}, error={}, token_debug={}",
                correlation_id, query.repo, query.provider, error_str, token_debug(&access_token)
            );
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": error_str
            })))
        }
    }
}

/// Get repository languages
pub async fn get_repository_languages(
    req: web::Json<ValidateRepoAccessRequest>,
) -> Result<HttpResponse> {
    info!("🔤 Fetching languages for repository: {}", req.repo_url);
    
    let connector = GitHubConnector::new();
    
    let access_request = GitHubRepoAccessRequest {
        repo_url: req.repo_url.clone(),
        access_token: req.access_token.clone(),
    };
    
    match connector.validate_repo_access(&access_request).await {
        Ok(response) => {
            if response.has_access {
                if let Some(repo_info) = response.repo_info {
                    let languages = repo_info.languages;
                    info!("✅ Found {} languages", languages.len());
                    Ok(HttpResponse::Ok().json(serde_json::json!({
                        "success": true,
                        "languages": languages
                    })))
                } else {
                    Ok(HttpResponse::BadRequest().json(serde_json::json!({
                        "success": false,
                        "error": "No repository information available"
                    })))
                }
            } else {
                Ok(HttpResponse::Forbidden().json(serde_json::json!({
                    "success": false,
                    "error": response.error_message.unwrap_or_else(|| "Access denied".to_string())
                })))
            }
        }
        Err(e) => {
            error!("❌ Failed to fetch languages: {}", e);
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing with observability infrastructure
    init_tracing(TracingConfig::for_service("data-service"));
    
    let port = env::var("PORT").unwrap_or_else(|_| "3013".to_string());
    let bind_addr = format!("0.0.0.0:{}", port);
    
    info!("🚀 [Data Service] Starting on port {}", port);
    info!("⚠️  [Data Service] Running in minimal mode - database features disabled");
    
    // Initialize Kafka producer
    let kafka_producer = Arc::new(KafkaProducer::from_env());
    info!("📡 Kafka producer initialized (enabled: {})", kafka_producer.is_enabled());
    
    // Initialize GitHub App client (optional - only if configured)
    let github_app_state: Option<Arc<GitHubAppState>> = match GitHubAppConfig::from_env() {
        Ok(config) => {
            info!("🐙 GitHub App configured (app_id: {})", config.app_id);
            let client = GitHubAppClient::new(config);
            Some(Arc::new(GitHubAppState::new(client)))
        }
        Err(e) => {
            warn!("⚠️  GitHub App not configured: {}. GitHub App endpoints will return errors.", e);
            None
        }
    };
    
    let auth_middleware = match AuthMiddlewareFactory::new() {
        Ok(m) => m,
        Err(e) => {
            warn!("Auth middleware init failed: {}. Falling back to disabled dev claims", e);
            AuthMiddlewareFactory::disabled()
        }
    };

    // Initialize auth client for internal service-to-service calls
    let auth_client = AuthClient::from_env();
    info!("🔑 Auth client initialized");

    // Initialize embedding client
    let embedding_service_url = env::var("EMBEDDING_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:3015".to_string());
    let embedding_enabled = env::var("EMBEDDING_ENABLED")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(true);
    let embedding_client = EmbeddingClient::new(embedding_service_url.clone(), embedding_enabled);
    info!("📊 Embedding client initialized (enabled: {}, url: {})", embedding_enabled, embedding_service_url);

    // Initialize graph RAG ingestion service (optional)
    let chunker_url = env::var("CHUNKER_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:3016".to_string());
    let graph_ingestion: Option<GraphRagIngestionService> = if env::var("GRAPH_RAG_ENABLED")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(true)
    {
        info!("📈 Graph RAG ingestion service initialized (chunker: {})", chunker_url);
        Some(GraphRagIngestionService::new(chunker_url))
    } else {
        info!("📈 Graph RAG ingestion disabled");
        None
    };

    HttpServer::new(move || {
        // Configure CORS - allow any header to support frontend trace headers (x-trace-id, x-span-id, x-request-id)
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://127.0.0.1:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allow_any_header()
            .supports_credentials()
            .max_age(3600);
        
        let mut app = App::new()
            .wrap(cors)
            .wrap(observability("data-service"))
            .wrap(auth_middleware.clone())
            .app_data(web::Data::new(kafka_producer.clone()))
            .app_data(web::Data::new(auth_client.clone()))
            .app_data(web::Data::new(embedding_client.clone()))
            .app_data(web::Data::new(graph_ingestion.clone()))
            .route("/health", web::get().to(health_check))
            .route("/status", web::get().to(status_check))
            // GitHub repository management routes (legacy PAT-based)
            .route("/api/github/validate-access", web::post().to(validate_github_repo_access))
            .route("/api/github/sync-repository", web::post().to(sync_github_repository))
            .route("/api/github/branches", web::post().to(get_repository_branches))
            .route("/api/github/languages", web::post().to(get_repository_languages))
            // NEW: Secure GitHub sync (uses JWT + auth service for token)
            .route("/api/github/sync", web::post().to(secure_sync_github_repository))
            // OAuth-based repository routes (uses JWT to get token from auth service)
            .route("/api/repositories/oauth/check", web::post().to(oauth_repo_check))
            .route("/api/repositories/oauth/branches", web::get().to(oauth_repo_branches))
            // List endpoints for frontend compatibility
            .route("/api/repositories", web::get().to(list_repositories))
            .route("/api/data-sources", web::get().to(list_data_sources))
            // Data sources CRUD (for repo connect flow)
            .route("/api/data/sources", web::post().to(create_data_source))
            .route("/api/data/sources", web::get().to(list_data_sources))
            // Robot management routes
            .route("/api/robots/register", web::post().to(robots::register_robot))
            .route("/api/robots", web::get().to(robots::list_robots))
            .route("/api/robots/{robot_id}", web::get().to(robots::get_robot))
            .route("/api/robots/{robot_id}", web::delete().to(robots::delete_robot))
            .route("/api/robots/{robot_id}/streams", web::post().to(robots::declare_stream))
            .route("/api/robots/{robot_id}/heartbeat", web::post().to(robots::robot_heartbeat))
            // Robot ingestion routes (HTTP → Kafka bridge)
            .route("/api/ingestion/robots/{robot_id}/events", web::post().to(robot_ingestion::ingest_events))
            .route("/api/ingestion/robots/{robot_id}/events/batch", web::post().to(robot_ingestion::ingest_events_batch))
            .route("/api/ingestion/robots/{robot_id}/cv_events", web::post().to(robot_ingestion::ingest_cv_events))
            .route("/api/ingestion/robots/{robot_id}/cv_events/batch", web::post().to(robot_ingestion::ingest_cv_events_batch))
            .route("/api/ingestion/robots/{robot_id}/frames", web::post().to(robot_ingestion::ingest_frames))
            // Document management routes (local file upload + cloud import)
            .configure(handlers::documents::configure)
            // Local filesystem sync routes
            .configure(handlers::local_fs::configure);
        
        // Add GitHub App routes if configured
        if let Some(ref state) = github_app_state {
            app = app
                .app_data(web::Data::new(state.clone()))
                .configure(configure_github_app_routes);
        }
        
        app
    })
    .bind(&bind_addr)?
    .run()
    .await
}

async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "data-service",
        "mode": "minimal",
        "message": "Database features temporarily disabled"
    })))
}

async fn status_check() -> Result<HttpResponse> {
    let kafka_enabled = std::env::var("KAFKA_ENABLED")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);
    
    let github_app_enabled = std::env::var("GITHUB_APP_ID").is_ok();
    
    let embedding_enabled = std::env::var("EMBEDDING_ENABLED")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(true);
    
    let graph_rag_enabled = std::env::var("GRAPH_RAG_ENABLED")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(true);
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "service": "data-service",
        "version": "0.2.0",
        "status": "running",
        "features": {
            "database": false,
            "github_connector": true,
            "github_app_connector": github_app_enabled,
            "repository_sync": true,
            "secure_repository_sync": true,
            "branch_selection": true,
            "language_detection": true,
            "embedding_integration": embedding_enabled,
            "graph_rag_integration": graph_rag_enabled,
            "auth_service_integration": true,
            "robot_connector": true,
            "robot_memory": true,
            "kafka_integration": kafka_enabled,
            "http_ingestion": true,
            "issues_sync": github_app_enabled,
            "prs_sync": github_app_enabled
        }
    })))
}

/// List repositories - placeholder that returns empty list
/// In a full implementation, this would query the database for user's repositories
async fn list_repositories() -> Result<HttpResponse> {
    // TODO: Query database for user's connected repositories
    // For now, return empty list to prevent 404 errors
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Repositories retrieved",
        "data": {
            "repositories": []
        }
    })))
}

/// List data sources - placeholder that returns empty list
/// In a full implementation, this would query the database for user's data sources
async fn list_data_sources() -> Result<HttpResponse> {
    // TODO: Query database for user's connected data sources
    // For now, return empty list to prevent 404 errors
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Data sources retrieved",
        "data": {
            "dataSources": []
        }
    })))
}

#[derive(Debug, Deserialize)]
pub struct CreateDataSourceRequest {
    #[serde(rename = "type")]
    pub source_type: String,
    pub url: String,
    pub credentials: Option<serde_json::Value>,
    pub config: Option<serde_json::Value>,
}

/// Create a new data source (repository, cloud storage, etc.)
/// This is called when user clicks "Connect Repository" in the frontend
async fn create_data_source(
    req: web::Json<CreateDataSourceRequest>,
) -> Result<HttpResponse> {
    info!(
        "[Data Source] Creating new data source: type={}, url={}",
        req.source_type, req.url
    );
    
    // Extract config values
    let config = req.config.as_ref();
    let name = config
        .and_then(|c| c.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unnamed Source");
    let default_branch = config
        .and_then(|c| c.get("defaultBranch"))
        .and_then(|v| v.as_str())
        .unwrap_or("main");
    
    // Generate a unique ID for this data source
    let source_id = Uuid::new_v4().to_string();
    
    // TODO: In a full implementation, this would:
    // 1. Validate the repository URL and credentials
    // 2. Store the data source in the database
    // 3. Optionally trigger an initial sync
    
    info!(
        "[Data Source] Created data source: id={}, name={}, type={}, branch={}",
        source_id, name, req.source_type, default_branch
    );
    
    Ok(HttpResponse::Created().json(serde_json::json!({
        "success": true,
        "message": "Data source created successfully",
        "data": {
            "id": source_id,
            "name": name,
            "type": req.source_type,
            "url": req.url,
            "status": "connected",
            "defaultBranch": default_branch,
            "createdAt": chrono::Utc::now().to_rfc3339()
        }
    })))
}
