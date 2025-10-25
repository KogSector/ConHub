use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;

mod services;
mod handlers;
mod sources;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Service port
    let port = env::var("DATA_SERVICE_PORT")
        .unwrap_or_else(|_| "3013".to_string())
        .parse::<u16>()
        .unwrap_or(3013);

    // Database connection
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://conhub:conhub_password@postgres:5432/conhub".to_string());

    println!("📊 [Data Service] Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    println!("✅ [Data Service] Database connection established");

    // Qdrant connection (vector database)
    let qdrant_url = env::var("QDRANT_URL")
        .unwrap_or_else(|_| "http://qdrant:6333".to_string());

    println!("📊 [Data Service] Connecting to Qdrant at {}...", qdrant_url);
    // TODO: Initialize Qdrant client
    println!("✅ [Data Service] Qdrant connection configured");

    println!("🚀 [Data Service] Starting on port {}", port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .configure(configure_routes)
            .route("/health", web::get().to(health_check))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}

fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/data")
            .route("/sources", web::get().to(handlers::data::list_sources))
            .route("/sources", web::post().to(handlers::data::create_source))
            .route("/sources/{id}", web::get().to(handlers::data::get_source))
            .route("/sources/{id}", web::delete().to(handlers::data::delete_source))
            .route("/sources/{id}/sync", web::post().to(handlers::data::sync_source))
            .route("/documents", web::get().to(handlers::data::list_documents))
            .route("/documents/{id}", web::get().to(handlers::data::get_document))
            .route("/search", web::post().to(handlers::data::search_documents))
    );
}

async fn health_check(pool: web::Data<PgPool>) -> actix_web::Result<web::Json<serde_json::Value>> {
    let db_status = match sqlx::query("SELECT 1 as test").fetch_one(pool.get_ref()).await {
        Ok(_) => "connected",
        Err(e) => {
            log::error!("[Data Service] Database health check failed: {}", e);
            "disconnected"
        }
    };

    Ok(web::Json(serde_json::json!({
        "status": "healthy",
        "service": "data-service",
        "database": db_status,
        "timestamp": chrono::Utc::now()
    })))
}
