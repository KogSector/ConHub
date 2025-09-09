use actix_web::{web, App, HttpResponse, HttpServer, Result};
use actix_cors::Cors;
use serde_json::json;

async fn index() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "message": "Welcome to ConHub Backend API - Unify repositories, docs, and URLs with AI",
        "version": "1.0.0",
        "endpoints": {
            "health": "/health",
            "api_docs": "Coming soon..."
        }
    })))
}

async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "ok",
        "service": "ConHub Backend - Multi-source AI Context Platform"
    })))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting ConHub Backend on http://localhost:3001");
    
    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec!["Content-Type", "Authorization"]);
            
        App::new()
            .wrap(cors)
            .route("/", web::get().to(index))
            .route("/health", web::get().to(health))
    })
    .bind("127.0.0.1:3001")?
    .run()
    .await
}