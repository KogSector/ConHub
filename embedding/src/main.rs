use actix_web::{web, App, HttpServer};
use std::env;
use std::sync::Arc;

mod config;
mod handlers;
mod llm;
mod models;
mod services;

use handlers::{embed_handler, health_handler, disabled_handler, batch_embed_handler, batch_embed_chunks_handler};
use services::{LlmEmbeddingService, FusionEmbeddingService};
use conhub_config::feature_toggles::FeatureToggles;
use crate::models::EmbeddingStatus;

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
        log::info!("Initializing multi-model fusion embedding service...");
        
        let config_path = env::var("EMBEDDING_FUSION_CONFIG_PATH")
            .unwrap_or_else(|_| "config/fusion_config.json".to_string());
        
        log::info!("Loading fusion config from: {}", config_path);

        match FusionEmbeddingService::new(&config_path) {
            Ok(svc) => {
                log::info!("Fusion embedding service initialized successfully");
                Some(Arc::new(svc))
            }
            Err(e) => {
                log::error!("Failed to initialize fusion embedding service: {}", e);
                None
            }
        }
    } else {
        log::warn!("Heavy mode disabled; skipping embedding model initialization.");
        None
    };

    // Service ready for production use

    // Start HTTP server
    log::info!("Starting HTTP server...");
    let heavy_ready = embedding_service.is_some();
    let status = EmbeddingStatus { available: heavy_ready, reason: if heavy_enabled && !heavy_ready { Some("missing_api_keys".to_string()) } else { None } };

    HttpServer::new(move || {
        let mut app = App::new()
            .app_data(web::Data::new(status.clone()))
            .route("/health", web::get().to(health_handler));

        if heavy_ready {
            app = app
                .app_data(web::Data::new(embedding_service.clone().unwrap()))
                .route("/embed", web::post().to(embed_handler))
                .route("/batch/embed", web::post().to(batch_embed_handler))
                .route("/batch/embed/chunks", web::post().to(batch_embed_chunks_handler))
                ;
        } else {
            app = app
                .app_data(web::Data::new(status.clone()))
                .route("/embed", web::post().to(disabled_handler))
                .route("/batch/embed", web::post().to(disabled_handler))
                .route("/batch/embed/chunks", web::post().to(disabled_handler))
                ;
        }

        app
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
