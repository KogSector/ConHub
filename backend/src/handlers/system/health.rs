use actix_web::{web, HttpResponse, Result};
use serde_json::json;
use crate::config::AppConfig;
use crate::services::health;

async fn index() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "message": "ConHub Backend API",
        "version": "1.0.0",
        "services": {
            "langchain": "http://localhost:3002",
            "haystack": "http://localhost:8001",
            "unified_indexer": "http://localhost:8080"
        },
        "features": [
            "Repository connectivity",
            "AI agent integration",
            "Multi-source indexing",
            "Semantic search",
            "Service orchestration"
        ]
    })))
}

async fn health_check(config: web::Data<AppConfig>) -> Result<HttpResponse> {
    let services = health::get_all_services_status(
        &config.http_client,
        &config.langchain_url,
        &config.haystack_url,
        &config.unified_indexer_url,
    ).await;
    
    Ok(HttpResponse::Ok().json(json!({
        "status": "ok",
        "service": "ConHub Backend",
        "services": services
    })))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(index))
       .route("/health", web::get().to(health_check));
}