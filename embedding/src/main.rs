use actix_web::{web, App, HttpServer};
use std::env;
use std::sync::Arc;

mod config;
mod handlers;
mod llm;
mod models;
mod services;

use handlers::{embed_handler, health_handler, disabled_handler, batch_embed_handler};
use services::LlmEmbeddingService;
use conhub_config::feature_toggles::FeatureToggles;

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

    // Load feature toggles
    let toggles = FeatureToggles::from_env_path();
    let heavy_enabled = toggles.is_enabled("Heavy");

    // Initialize services
    let embedding_service = if heavy_enabled {
        log::info!("Initializing embedding and reranking models...");
        let default_model = env::var("QWEN_EMBEDDING_MODEL").unwrap_or_else(|_| "text-embedding-v3".to_string());
        let model = env::var("EMBEDDING_MODEL").unwrap_or_else(|_| default_model.to_string());

        log::info!("Embedding provider: qwen | model: {}", model);

        let embedding_service = Arc::new(
            LlmEmbeddingService::new(&model)
                .expect("Failed to initialize embedding service")
        );
        log::info!("Embedding model initialized successfully");
        Some(embedding_service)
    } else {
        log::warn!("Heavy mode disabled; skipping embedding model initialization.");
        None
    };

    // Service ready for production use

    // Start HTTP server
    log::info!("Starting HTTP server...");
    HttpServer::new(move || {
        let mut app = App::new()
            .app_data(web::Data::new(toggles.clone()))
            .route("/health", web::get().to(health_handler));

        if heavy_enabled {
            app = app
                .app_data(web::Data::new(embedding_service.clone().unwrap()))
                .route("/embed", web::post().to(embed_handler))
                .route("/batch/embed", web::post().to(batch_embed_handler))
                ;
        } else {
            app = app
                .route("/embed", web::post().to(disabled_handler))
                .route("/batch/embed", web::post().to(disabled_handler))
                ;
        }

        app
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
