use actix_web::{web, App, HttpServer};
use std::env;
use std::sync::Arc;

mod handlers;
mod models;
mod services;

use handlers::{embed_handler, health_handler, rerank_handler};
use services::{EmbeddingService, RerankService};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init();

    // Load configuration
    let port = env::var("EMBEDDING_PORT")
        .unwrap_or_else(|_| "8082".to_string())
        .parse::<u16>()
        .expect("Invalid port number");

    let host = env::var("EMBEDDING_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    log::info!("Starting embedding service on {}:{}", host, port);

    // Initialize services
    log::info!("Initializing embedding and reranking models...");
    let embedding_service = Arc::new(
        EmbeddingService::new()
            .expect("Failed to initialize embedding service")
    );
    let rerank_service = Arc::new(
        RerankService::new()
            .expect("Failed to initialize reranking service")
    );
    log::info!("Models initialized successfully");

    // Start HTTP server
    log::info!("Starting HTTP server...");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(embedding_service.clone()))
            .app_data(web::Data::new(rerank_service.clone()))
            .route("/health", web::get().to(health_handler))
            .route("/embed", web::post().to(embed_handler))
            .route("/rerank", web::post().to(rerank_handler))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
