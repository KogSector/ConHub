use actix_web::{web, App, HttpServer, HttpResponse};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use std::collections::HashMap;
use uuid::Uuid;

mod handlers;
mod services;
mod models;

use models::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();

    let port = env::var("CHUNKER_PORT")
        .unwrap_or_else(|_| "3014".to_string())
        .parse::<u16>()
        .expect("Invalid port number");

    let host = env::var("CHUNKER_HOST")
        .unwrap_or_else(|_| "0.0.0.0".to_string());

    info!("🚀 [Chunker Service] Starting on {}:{}", host, port);

    // Initialize downstream service clients
    let embedding_url = env::var("EMBEDDING_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:8082".to_string());
    
    let graph_url = env::var("GRAPH_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:3015".to_string());

    let max_concurrent_jobs = env::var("MAX_CONCURRENT_JOBS")
        .unwrap_or_else(|_| "10".to_string())
        .parse::<usize>()
        .unwrap_or(10);

    info!("📡 Embedding service: {}", embedding_url);
    info!("🔗 Graph service: {}", graph_url);
    info!("⚙️  Max concurrent jobs: {}", max_concurrent_jobs);

    // Create app state
    let state = Arc::new(AppState {
        embedding_client: services::embedding_client::EmbeddingClient::new(embedding_url),
        graph_client: services::graph_client::GraphClient::new(graph_url),
        max_concurrent_jobs,
        jobs: RwLock::new(HashMap::new()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/health", web::get().to(health_check))
            .route("/chunk/jobs", web::post().to(handlers::jobs::start_chunk_job))
            .route("/chunk/jobs/{job_id}", web::get().to(handlers::jobs::get_chunk_job_status))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "chunker",
        "version": "0.1.0"
    }))
}
