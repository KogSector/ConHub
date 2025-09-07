use actix_web::{web, App, HttpResponse, HttpServer, Result};
use serde_json::json;

async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "ok",
        "service": "ConHub Backend"
    })))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting ConHub Backend on http://localhost:8080");
    
    HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(health))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}