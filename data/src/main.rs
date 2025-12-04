use actix_web::{web, App, HttpServer, HttpRequest, HttpResponse, Result, HttpMessage};
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
}

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
}

/// New secure sync request - no access_token required (fetched from auth service)
#[derive(Debug, Deserialize)]
pub struct SecureSyncRepositoryRequest {
    pub repo_url: String,
    pub branch: String,
    pub include_languages: Option<Vec<String>>,
    pub exclude_paths: Option<Vec<String>>,
    pub max_file_size_mb: Option<i64>,
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
                graph_job_id: None,
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
    };
    
    Ok(HttpResponse::Ok().json(response))
}

/// Helper to extract user_id from JWT claims
fn extract_user_id_from_request(req: &HttpRequest) -> Option<Uuid> {
    req.extensions()
        .get::<Claims>()
        .and_then(|claims| claims.sub.parse::<Uuid>().ok())
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
            };
            return Ok(HttpResponse::BadRequest().json(response));
        }
    };
    
    info!("✅ Retrieved GitHub token for user {}", user_id);
    
    // Step 3: Sync repository using GitHubConnector
    let connector = GitHubConnector::new();
    
    let sync_config = GitHubSyncConfig {
        repo_url: req.repo_url.clone(),
        branch: req.branch.clone(),
        include_languages: req.include_languages.clone(),
        exclude_paths: req.exclude_paths.clone(),
        max_file_size_mb: req.max_file_size_mb,
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
            };
            return Ok(HttpResponse::BadRequest().json(response));
        }
    };
    
    info!("📄 Retrieved {} documents from repository", documents.len());
    
    let doc_count = documents.len();
    
    // Step 4: Send to embedding service (vector store)
    let mut embeddings_created = 0;
    if let Err(e) = embedding_client.embed_documents(documents.clone()).await {
        warn!("⚠️ Embedding service error (continuing): {}", e);
    } else {
        embeddings_created = doc_count;
        info!("✅ Sent {} documents to embedding service", doc_count);
    }
    
    // Step 5: Send to graph RAG ingestion
    let graph_job_id = if let Some(ref graph_service) = graph_ingestion.get_ref() {
        // Use a stable source_id based on repo URL
        let source_id = Uuid::new_v5(&Uuid::NAMESPACE_URL, req.repo_url.as_bytes());
        
        match graph_service.ingest_documents(source_id, SourceKind::CodeRepo, documents).await {
            Ok(job_id) => {
                info!("✅ Graph RAG ingestion job started: {}", job_id);
                Some(job_id)
            }
            Err(e) => {
                warn!("⚠️ Graph RAG ingestion error (continuing): {}", e);
                None
            }
        }
    } else {
        info!("📊 Graph RAG ingestion service not configured, skipping");
        None
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
        let mut app = App::new()
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
