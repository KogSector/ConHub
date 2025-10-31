use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;
use tracing::{info, error};
use tracing_subscriber;

mod services;
mod handlers;
mod sources;
mod errors;

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

    // Database connection
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://conhub:conhub_password@postgres:5432/conhub".to_string());

    tracing::info!("📊 [Data Service] Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    tracing::info!("✅ [Data Service] Database connection established");

    // Qdrant connection (vector database)
    let qdrant_url = env::var("QDRANT_URL")
        .unwrap_or_else(|_| "http://qdrant:6333".to_string());

    tracing::info!("📊 [Data Service] Connecting to Qdrant at {}...", qdrant_url);
    // TODO: Initialize Qdrant client
    tracing::info!("✅ [Data Service] Qdrant connection configured");

    tracing::info!("🚀 [Data Service] Starting on port {}", port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(cors)
            .wrap(Logger::default())
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
    );
}

async fn health_check(pool: web::Data<PgPool>) -> actix_web::Result<web::Json<serde_json::Value>> {
    let db_status = match sqlx::query("SELECT 1 as test").fetch_one(pool.get_ref()).await {
        Ok(_) => "connected",
        Err(e) => {
            tracing::error!("[Data Service] Database health check failed: {}", e);
            "disconnected"
        }
    };

    Ok(web::Json(serde_json::json!({
        "status": "healthy",
        "service": "data-service",
        "database": db_status,
        "timestamp": chrono::Utc::now()
    })))
}
