use actix_web::{web, App, HttpResponse, HttpServer, Result, middleware::Logger};
use actix_cors::Cors;
use serde_json::json;

async fn index() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "message": "ConHub Backend API",
        "version": "1.0.0",
        "features": [
            "Core API services",
            "Authentication",
            "Data management"
        ]
    })))
}

async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "ok",
        "service": "ConHub Backend"
    })))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    println!("Starting ConHub Backend on http://localhost:3001");
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec!["Content-Type", "Authorization"]);
            
        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .route("/", web::get().to(index))
            .route("/health", web::get().to(health))
    })
    .bind("127.0.0.1:3001")?
    .run()
    .await
}