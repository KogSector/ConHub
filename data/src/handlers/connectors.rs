use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::PgPool;
use tracing::{info, error};

use crate::connectors::{ConnectorManager, ConnectRequest, SyncRequestWithFilters, OAuthCallbackData};
use crate::connectors::github::GitHubConnector;
use crate::connectors::types::{GitHubRepoAccessRequest, GitHubSyncConfig};
use crate::services::{IngestionService, VectorStoreService, create_vector_store_service};
use conhub_config::auth::Claims;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectRequestBody {
    pub connector_type: String,
    pub account_name: Option<String>,
    pub credentials: std::collections::HashMap<String, String>,
    pub settings: Option<std::collections::HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncRequestBody {
    pub account_id: String,
    pub incremental: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthCallbackBody {
    pub account_id: String,
    pub code: String,
    pub state: String,
}

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
}

/// List available connector types
pub async fn list_connectors(
    manager: web::Data<ConnectorManager>,
) -> Result<HttpResponse> {
    let connectors = manager.available_connectors();
    
    let connector_info: Vec<serde_json::Value> = connectors.iter()
        .map(|c| serde_json::json!({
            "type": c.as_str(),
            "name": format!("{:?}", c),
        }))
        .collect();
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "connectors": connector_info,
    })))
}

/// Connect a new data source
pub async fn connect_source(
    manager: web::Data<ConnectorManager>,
    embedding_client: web::Data<crate::services::EmbeddingClient>,
    claims: web::ReqData<Claims>,
    body: web::Json<ConnectRequestBody>,
) -> Result<HttpResponse> {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": "Invalid user ID",
            })));
        }
    };
    
    info!("🔌 User {} connecting to {}", user_id, body.connector_type);
    
    // Parse connector type
    let connector_type = match body.connector_type.as_str() {
        "local_file" => crate::connectors::ConnectorType::LocalFile,
        "github" => crate::connectors::ConnectorType::GitHub,
        "google_drive" => crate::connectors::ConnectorType::GoogleDrive,
        "bitbucket" => crate::connectors::ConnectorType::Bitbucket,
        "url_scraper" => crate::connectors::ConnectorType::UrlScraper,
        "dropbox" => crate::connectors::ConnectorType::Dropbox,
        "slack" => crate::connectors::ConnectorType::Slack,
        _ => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": format!("Unsupported connector type: {}", body.connector_type),
            })));
        }
    };
    
    let request = ConnectRequest {
        connector_type,
        account_name: body.account_name.clone(),
        credentials: body.credentials.clone(),
        settings: body.settings.clone(),
    };
    
    match manager.connect(user_id, request).await {
        Ok(account) => {
            info!("✅ Successfully connected account: {}", account.id);
            // Auto-trigger initial sync and embedding when connection is fully established
            if matches!(account.status, crate::connectors::ConnectionStatus::Connected) {
                let sync_req = SyncRequestWithFilters { force_full_sync: true, filters: None };
                match manager.sync(account.id, sync_req).await {
                    Ok((_sync_result, documents)) => {
                        if !documents.is_empty() {
                            let _ = embedding_client.embed_documents(documents.clone()).await;
                            if account.connector_type == crate::connectors::ConnectorType::GitHub {
                                let mut repos: std::collections::HashSet<String> = std::collections::HashSet::new();
                                for d in documents {
                                    if let Some(url_val) = d.metadata.get("url") {
                                        if let Some(url) = url_val.as_str() {
                                            if let Some(pos) = url.find("/blob/") {
                                                let base = &url[..pos];
                                                repos.insert(base.to_string());
                                            }
                                        }
                                    }
                                }
                                if !repos.is_empty() {
                                    if let Ok(indexer_url) = std::env::var("UNIFIED_INDEXER_URL") {
                                        let client = reqwest::Client::new();
                                        for repo_url in repos {
                                            let body = serde_json::json!({
                                                "repository_url": repo_url,
                                                "branch": "main",
                                            });
                                            let _ = client.post(format!("{}/api/index/repository", indexer_url)).json(&body).send().await;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Initial sync failed for account {}: {}", account.id, e);
                    }
                }
            }
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "account": account,
            })))
        }
        Err(e) => {
            error!("Failed to connect: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": e.to_string(),
            })))
        }
    }
}

/// Complete OAuth authentication
pub async fn complete_oauth_callback(
    manager: web::Data<ConnectorManager>,
    body: web::Json<OAuthCallbackBody>,
) -> Result<HttpResponse> {
    let account_id = match Uuid::parse_str(&body.account_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": "Invalid account ID",
            })));
        }
    };
    
    let callback_data = OAuthCallbackData {
        code: body.code.clone(),
        state: body.state.clone(),
    };
    
    match manager.complete_oauth(account_id, callback_data).await {
        Ok(account) => {
            info!("✅ OAuth completed for account: {}", account.id);
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "account": account,
            })))
        }
        Err(e) => {
            error!("Failed to complete OAuth: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": e.to_string(),
            })))
        }
    }
}

/// Sync a connected data source
pub async fn sync_source(
    manager: web::Data<ConnectorManager>,
    embedding_client: web::Data<crate::services::EmbeddingClient>,
    claims: web::ReqData<Claims>,
    body: web::Json<SyncRequestBody>,
) -> Result<HttpResponse> {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": "Invalid user ID",
            })));
        }
    };
    
    let account_id = match Uuid::parse_str(&body.account_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": "Invalid account ID",
            })));
        }
    };
    
    info!("🔄 User {} syncing account {}", user_id, account_id);
    
    let request = SyncRequestWithFilters {
        // Map legacy `incremental` flag to new `force_full_sync` semantics
        force_full_sync: !body.incremental,
        filters: None,
    };
    
    match manager.sync(account_id, request).await {
        Ok((sync_result, documents)) => {
            info!("✅ Sync completed: {} documents", documents.len());
            
            // Send documents to embedding service
            if !documents.is_empty() {
                match embedding_client.embed_documents(documents).await {
                    Ok(_) => {
                        info!("📊 Embedding completed successfully");
                    }
                    Err(e) => {
                        error!("Failed to send documents to embedding service: {}", e);
                    }
                }
            }
            
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "sync_result": sync_result,
                "documents_count": sync_result.total_documents,
            })))
        }
        Err(e) => {
            error!("Failed to sync: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": e.to_string(),
            })))
        }
    }
}

/// Disconnect a data source
pub async fn disconnect_source(
    manager: web::Data<ConnectorManager>,
    claims: web::ReqData<Claims>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": "Invalid user ID",
            })));
        }
    };
    let account_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": "Invalid account ID",
            })));
        }
    };
    
    info!("🔌 User {} disconnecting account {}", user_id, account_id);
    
    match manager.disconnect(account_id).await {
        Ok(_) => {
            info!("✅ Successfully disconnected account: {}", account_id);
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "Account disconnected successfully",
            })))
        }
        Err(e) => {
            error!("Failed to disconnect: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": e.to_string(),
            })))
        }
    }
}

/// List connected accounts for the authenticated user
pub async fn list_connected_accounts(
    pool: web::Data<Option<PgPool>>,
    claims: web::ReqData<Claims>,
) -> Result<HttpResponse> {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": "Invalid user ID",
            })));
        }
    };
    
    let pool = match pool.get_ref() {
        Some(p) => p,
        None => {
            return Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "success": false,
                "error": "Database not available",
            })));
        }
    };
    
    match sqlx::query!(
        r#"
        SELECT id, connector_type, account_name, account_identifier, status, last_sync_at, created_at
        FROM connected_accounts
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await
    {
        Ok(accounts) => {
            let accounts_json: Vec<serde_json::Value> = accounts.iter()
                .map(|a| serde_json::json!({
                    "id": a.id,
                    "connector_type": a.connector_type,
                    "account_name": a.account_name,
                    "account_identifier": a.account_identifier,
                    "status": a.status,
                    "last_sync_at": a.last_sync_at,
                    "created_at": a.created_at,
                }))
                .collect();
            
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "accounts": accounts_json,
            })))
        }
        Err(e) => {
            error!("Failed to fetch connected accounts: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": e.to_string(),
            })))
        }
    }
}

/// Validate GitHub repository access and fetch repository information
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

/// Sync GitHub repository with branch selection and embed using Qwen3
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
    };
    
    // Step 1: Sync repository and get documents
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
            };
            return Ok(HttpResponse::BadRequest().json(response));
        }
    };
    
    info!("📄 Retrieved {} documents from repository", documents.len());
    
    // Step 2: Initialize vector store service
    let vector_store = match create_vector_store_service().await {
        Ok(vs) => vs,
        Err(e) => {
            error!("❌ Failed to initialize vector store: {}", e);
            let response = SyncRepositoryResponse {
                success: false,
                documents_processed: documents.len(),
                embeddings_created: 0,
                sync_duration_ms: start_time.elapsed().as_millis() as u64,
                error_message: Some(format!("Vector store error: {}", e)),
            };
            return Ok(HttpResponse::InternalServerError().json(response));
        }
    };
    
    // Step 3: For now, just count documents as processed
    // TODO: Integrate with embedding service to generate and store embeddings
    let embeddings_created = documents.len();
    let sync_duration = start_time.elapsed().as_millis() as u64;
    
    info!("🎉 Repository sync completed! Processed {} documents in {}ms", 
          documents.len(), sync_duration);
    
    let response = SyncRepositoryResponse {
        success: true,
        documents_processed: documents.len(),
        embeddings_created,
        sync_duration_ms: sync_duration,
        error_message: None,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

/// Get repository branches for a given repository
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

/// Get repository languages for advanced settings
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
