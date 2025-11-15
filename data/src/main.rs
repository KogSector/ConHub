
use actix_web::{web, App, HttpServer, HttpResponse, Result};
use tracing::{info, warn};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let port = env::var("PORT").unwrap_or_else(|_| "3013".to_string());
    let bind_addr = format!("0.0.0.0:{}", port);
    
    info!("🚀 [Data Service] Starting on port {}", port);
    info!("⚠️  [Data Service] Running in minimal mode - database features disabled");
    
    HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(health_check))
            .route("/status", web::get().to(status_check))
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
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "service": "data-service",
        "version": "0.1.0",
        "status": "running",
        "features": {
            "database": false,
            "connectors": false,
            "embedding": false
        }
    })))
}
