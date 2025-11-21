use actix_web::{web, App, HttpServer, HttpResponse, middleware::Logger};
use actix_cors::Cors;
use std::env;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod services;
mod handlers;
mod models;

use services::AgenticOrchestrator;

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "agentic"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment
    dotenv::dotenv().ok();

    tracing::info!("🤖 Starting ConHub Agentic Service");

    // Read configuration
    let port = env::var("AGENTIC_PORT")
        .unwrap_or_else(|_| "3005".to_string())
        .parse::<u16>()
        .expect("Invalid AGENTIC_PORT");

    let host = env::var("AGENTIC_HOST")
        .unwrap_or_else(|_| "0.0.0.0".to_string());

    // Initialize service URLs
    let embedding_url = env::var("EMBEDDING_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:8082".to_string());
    let graph_url = env::var("GRAPH_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:8006".to_string());
    let data_url = env::var("DATA_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:3013".to_string());

    // Initialize orchestrator
    let orchestrator = Arc::new(AgenticOrchestrator::new(
        embedding_url.clone(),
        graph_url.clone(),
        data_url.clone(),
    ));

    tracing::info!("✅ Agentic orchestrator initialized");
    tracing::info!("   Embedding: {}", embedding_url);
    tracing::info!("   Graph: {}", graph_url);
    tracing::info!("   Data: {}", data_url);

    let orchestrator_data = web::Data::new(orchestrator);

    // Start HTTP server
    tracing::info!("🚀 Starting HTTP server on {}:{}", host, port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .app_data(orchestrator_data.clone())
            .wrap(cors)
            .wrap(Logger::default())
            .service(
                web::scope("/api/agentic")
                    .route("/query", web::post().to(handlers::agentic_query))
            )
            .route("/health", web::get().to(health))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
