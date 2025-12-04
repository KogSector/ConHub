use actix_web::{web, App, HttpServer};
use std::env;
use std::sync::Arc;
use conhub_observability::{init_tracing, TracingConfig, observability, info, warn, error};

mod config;
mod handlers;
mod llm;
mod models;
mod services;

use handlers::{fusion_embed_handler, embed_handler, health_handler, disabled_handler, batch_embed_handler, batch_embed_chunks_handler, rerank_handler, vector_search, search_by_ids, search_by_entity};
use services::{LlmEmbeddingService, FusionEmbeddingService};
use conhub_config::feature_toggles::FeatureToggles;
use crate::models::EmbeddingStatus;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize observability with structured logging
    init_tracing(TracingConfig::for_service("vector-rag"));

    // Load configuration
    let port = env::var("EMBEDDING_PORT")
        .unwrap_or_else(|_| "8082".to_string())
        .parse::<u16>()
        .expect("Invalid port number");

    let host = env::var("EMBEDDING_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    info!("Starting embedding service on {}:{}", host, port);

    // Load feature toggles
    let toggles = FeatureToggles::from_env_path();
    let heavy_enabled = toggles.is_enabled("Heavy");

    // Initialize services
    let embedding_service = if heavy_enabled {
        info!("Initializing multi-model fusion embedding service...");
        
        let config_path = env::var("EMBEDDING_FUSION_CONFIG_PATH")
            .unwrap_or_else(|_| "config/fusion_config.json".to_string());
        
        info!("Loading fusion config from: {}", config_path);

        match FusionEmbeddingService::new(&config_path) {
            Ok(svc) => {
                info!("Fusion embedding service initialized successfully");
                Some(Arc::new(svc))
            }
            Err(e) => {
                error!("Failed to initialize fusion embedding service: {}", e);
                None
            }
        }
    } else {
        warn!("Heavy mode disabled; skipping embedding model initialization.");
        None
    };

    // Service ready for production use

    // Start HTTP server
    info!("Starting HTTP server...");
    let heavy_ready = embedding_service.is_some();
    let status = EmbeddingStatus { available: heavy_ready, reason: if heavy_enabled && !heavy_ready { Some("missing_api_keys".to_string()) } else { None } };

    HttpServer::new(move || {
        let mut app = App::new()
            .wrap(observability("vector-rag"))
            .app_data(web::Data::new(status.clone()))
            .route("/health", web::get().to(health_handler));

        if heavy_ready {
            app = app
                .app_data(web::Data::new(embedding_service.clone().unwrap()))
                .route("/embed", web::post().to(fusion_embed_handler))
                .route("/batch/embed", web::post().to(batch_embed_handler))
                .route("/batch/embed/chunks", web::post().to(batch_embed_chunks_handler))
                .route("/rerank", web::post().to(rerank_handler))
                // Vector search endpoints
                .route("/vector/search", web::post().to(vector_search))
                .route("/vector/search_by_ids", web::post().to(search_by_ids))
                .route("/vector/search_by_entity/{entity_id}", web::get().to(search_by_entity))
                ;
        } else {
            app = app
                .app_data(web::Data::new(status.clone()))
                .route("/embed", web::post().to(disabled_handler))
                .route("/batch/embed", web::post().to(disabled_handler))
                .route("/batch/embed/chunks", web::post().to(disabled_handler))
                .route("/rerank", web::post().to(disabled_handler))
                ;
        }

        app
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
