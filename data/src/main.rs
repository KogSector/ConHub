
use actix_web::{web, App, HttpServer, HttpResponse, Result};
use conhub_middleware::auth::AuthMiddlewareFactory;
use tracing::{info, warn, error};
use std::env;
use std::sync::Arc;

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
    
    pub use zilliz_client::*;
    #[allow(deprecated)]
    pub use qdrant_client::*;
    pub use vector_store::*;
    pub use kafka_client::*;
    pub use github_app_client::*;
    pub use github_ingestion::*;
}

mod handlers {
    pub mod robots;
    pub mod robot_ingestion;
    pub mod github_app;
}

// Simplified handlers inline
use serde::{Deserialize, Serialize};

use connectors::github::GitHubConnector;
use connectors::types::{GitHubRepoAccessRequest, GitHubSyncConfig};
use services::{create_vector_store_service, GitHubAppClient, GitHubAppConfig};
use handlers::github_app::{GitHubAppState, configure_github_app_routes};

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
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
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

    HttpServer::new(move || {
        let mut app = App::new()
            .wrap(auth_middleware.clone())
            .app_data(web::Data::new(kafka_producer.clone()))
            .route("/health", web::get().to(health_check))
            .route("/status", web::get().to(status_check))
            // GitHub repository management routes (legacy PAT-based)
            .route("/api/github/validate-access", web::post().to(validate_github_repo_access))
            .route("/api/github/sync-repository", web::post().to(sync_github_repository))
            .route("/api/github/branches", web::post().to(get_repository_branches))
            .route("/api/github/languages", web::post().to(get_repository_languages))
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
            .route("/api/ingestion/robots/{robot_id}/frames", web::post().to(robot_ingestion::ingest_frames));
        
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
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "service": "data-service",
        "version": "0.1.0",
        "status": "running",
        "features": {
            "database": false,
            "github_connector": true,
            "github_app_connector": github_app_enabled,
            "repository_sync": true,
            "branch_selection": true,
            "language_detection": true,
            "embedding": false,
            "robot_connector": true,
            "robot_memory": true,
            "kafka_integration": kafka_enabled,
            "http_ingestion": true,
            "issues_sync": github_app_enabled,
            "prs_sync": github_app_enabled
        }
    })))
}
