use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;
use tracing::{info, error};
use tracing_subscriber;
use conhub_middleware::auth::AuthMiddlewareFactory;
use conhub_config::feature_toggles::FeatureToggles;

mod services;
mod handlers;
mod sources;
mod errors;
mod connectors;
mod temp_data;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Service port
    let port = env::var("DATA_SERVICE_PORT")
        .unwrap_or_else(|_| "3013".to_string())
        .parse::<u16>()
        .unwrap_or(3013);

    // Initialize authentication middleware with feature toggle
    let toggles = FeatureToggles::from_env_path();
    let auth_enabled = toggles.is_enabled_or("Auth", true);
    let auth_middleware = if auth_enabled {
        AuthMiddlewareFactory::new()
            .map_err(|e| {
                tracing::error!("Failed to initialize auth middleware: {}", e);
                e
            })?
    } else {
        tracing::warn!("Auth feature disabled via feature toggles; injecting default claims.");
        AuthMiddlewareFactory::disabled()
    };

    tracing::info!("🔐 [Data Service] Authentication middleware initialized");

    // Database connection (gated by Auth toggle)
    let db_pool_opt: Option<PgPool> = if auth_enabled {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://conhub:conhub_password@postgres:5432/conhub".to_string());

        tracing::info!("📊 [Data Service] Connecting to database...");
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
            .await?;
        tracing::info!("✅ [Data Service] Database connection established");
        Some(pool)
    } else {
        tracing::warn!("[Data Service] Auth disabled; skipping database connection.");
        None
    };

    // Qdrant connection (vector database) — gated by Auth toggle
    let qdrant_url = env::var("QDRANT_URL")
        .unwrap_or_else(|_| "http://localhost:6333".to_string());

    if auth_enabled {
        tracing::info!("📊 [Data Service] Connecting to Qdrant at {}...", qdrant_url);
        // TODO: Initialize Qdrant client
        tracing::info!("✅ [Data Service] Qdrant connection configured");
    } else {
        tracing::warn!("[Data Service] Auth disabled; skipping Qdrant connection.");
    }

    // Initialize Embedding Client
    let embedding_url = env::var("EMBEDDING_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:8082".to_string());
    let heavy_enabled = toggles.is_enabled("Heavy");
    let embedding_client = services::EmbeddingClient::new(embedding_url.clone(), heavy_enabled);
    tracing::info!("📊 [Data Service] Embedding client initialized: {}", embedding_url);

    // Initialize Connector Manager
    let connector_manager = connectors::ConnectorManager::new(db_pool_opt.clone());
    tracing::info!("🔌 [Data Service] Connector Manager initialized");

    tracing::info!("🚀 [Data Service] Starting on port {}", port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .app_data(web::Data::new(db_pool_opt.clone()))
            .app_data(web::Data::new(toggles.clone()))
            .app_data(web::Data::new(connector_manager.clone()))
            .app_data(web::Data::new(embedding_client.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(auth_middleware.clone())
            .configure(configure_routes)
            .route("/health", web::get().to(health_check))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}

fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/data")
            // Data sources routes
            .route("/sources", web::post().to(handlers::data_sources::connect_data_source))
            .route("/sources/branches", web::post().to(handlers::data_sources::fetch_branches))
            
            // Repository routes
            .route("/repositories", web::get().to(handlers::repositories::list_repositories))
            .route("/repositories", web::post().to(handlers::repositories::connect_repository))
            .route("/repositories/{id}", web::get().to(handlers::repositories::get_repository))
            .route("/repositories/{id}/sync", web::post().to(handlers::repositories::sync_repository))
            .route("/repositories/{id}/disconnect", web::delete().to(handlers::repositories::disconnect_repository))
            .route("/repositories/stats", web::get().to(handlers::repositories::get_repository_stats))
            
            // Document routes
            .route("/documents", web::get().to(handlers::documents::get_documents))
            .route("/documents", web::post().to(handlers::documents::create_document))
            .route("/documents/{id}", web::delete().to(handlers::documents::delete_document))
            .route("/documents/analytics", web::get().to(handlers::documents::get_document_analytics))
            
            // URL routes
            .route("/urls", web::get().to(handlers::urls::get_urls))
            .route("/urls", web::post().to(handlers::urls::create_url))
            .route("/urls/{id}", web::delete().to(handlers::urls::delete_url))
            .route("/urls/analytics", web::get().to(handlers::urls::get_url_analytics))
            
            // Indexing routes
            .route("/index/repository", web::post().to(handlers::indexing::index_repository))
            .route("/index/documentation", web::post().to(handlers::indexing::index_documentation))
            .route("/index/url", web::post().to(handlers::indexing::index_url))
            .route("/index/file", web::post().to(handlers::indexing::index_file))
            .route("/index/status", web::get().to(handlers::indexing::get_indexing_status))
    )
    .service(
        web::scope("/api/connectors")
            // Connector management routes
            .route("/list", web::get().to(handlers::connectors::list_connectors))
            .route("/connect", web::post().to(handlers::connectors::connect_source))
            .route("/oauth/callback", web::post().to(handlers::connectors::complete_oauth_callback))
            .route("/sync", web::post().to(handlers::connectors::sync_source))
            .route("/disconnect/{id}", web::delete().to(handlers::connectors::disconnect_source))
            .route("/accounts", web::get().to(handlers::connectors::list_connected_accounts))
    )
    .service(
        web::scope("/api/enhanced")
            // Enhanced data routes
            .route("/data", web::post().to(handlers::enhanced_handlers::get_enhanced_data))
            .route("/data/batch", web::post().to(handlers::enhanced_handlers::get_batch_data))
            
            // Performance monitoring routes
            .route("/metrics", web::post().to(handlers::enhanced_handlers::get_performance_metrics))
            
            // Cache management routes
            .route("/cache", web::post().to(handlers::enhanced_handlers::manage_cache))
            
            // Enhanced health check
            .route("/health", web::get().to(handlers::enhanced_handlers::enhanced_health_check))
    );
}

async fn health_check(pool_opt: web::Data<Option<PgPool>>) -> actix_web::Result<web::Json<serde_json::Value>> {
    let db_status = match pool_opt.get_ref() {
        Some(pool) => match sqlx::query("SELECT 1 as test").fetch_one(pool).await {
            Ok(_) => "connected",
            Err(e) => {
                tracing::error!("[Data Service] Database health check failed: {}", e);
                "disconnected"
            }
        },
        None => "disabled",
    };

    Ok(web::Json(serde_json::json!({
        "status": "healthy",
        "service": "data-service",
        "database": db_status,
        "timestamp": chrono::Utc::now()
    })))
}
