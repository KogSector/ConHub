mod config;
mod indexer;
mod search;
mod parser;
mod analyzer;
mod xref;
mod history;
mod web;
mod types;
mod utils;
mod enhanced_search;
mod ai_integration;
mod performance;

use actix_web::{web, App, HttpResponse, HttpServer, Result, middleware::Logger};
use actix_cors::Cors;
use serde_json::json;
use config::LexorConfig;
use web::*;
use std::sync::Arc;

async fn index() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "message": "Lexor - Code Indexing and Search Service",
        "version": "1.0.0",
        "features": [
            "Source code indexing",
            "Full-text search",
            "Symbol cross-referencing",
            "Git history analysis",
            "Multi-language support"
        ]
    })))
}

async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "ok",
        "service": "Lexor"
    })))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    println!("Starting Lexor service on http://localhost:3002");
    
    // Initialize Lexor
    let config = LexorConfig::default();
    let lexor_service = Arc::new(
        LexorService::new(config)
            .expect("Failed to initialize Lexor service")
    );
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec!["Content-Type", "Authorization"]);
            
        App::new()
            .app_data(web::Data::new(lexor_service.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .route("/", web::get().to(index))
            .route("/health", web::get().to(health))
            .service(
                web::scope("/api")
                    .route("/search", web::post().to(search_handler))
                    .route("/projects", web::get().to(get_projects_handler))
                    .route("/projects", web::post().to(add_project_handler))
                    .route("/projects/{id}/index", web::post().to(index_project_handler))
                    .route("/stats", web::get().to(get_stats_handler))
            )
    })
    .bind("127.0.0.1:3002")?
    .run()
    .await
}