use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;

mod handlers;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Service port
    let port = env::var("WEBHOOK_SERVICE_PORT")
        .unwrap_or_else(|_| "3015".to_string())
        .parse::<u16>()
        .unwrap_or(3015);

    // Database connection
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://conhub:conhub_password@postgres:5432/conhub".to_string());

    println!("📊 [Webhook Service] Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    println!("✅ [Webhook Service] Database connection established");

    println!("🚀 [Webhook Service] Starting on port {}", port);

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
        web::scope("/api/webhooks")
            .route("/github", web::post().to(handlers::github::handle_github_webhook))
            .route("/gitlab", web::post().to(handlers::handle_gitlab_webhook))
            .route("/stripe", web::post().to(handlers::stripe::handle_stripe_webhook))
    );
}

async fn health_check(pool: web::Data<PgPool>) -> actix_web::Result<web::Json<serde_json::Value>> {
    let db_status = match sqlx::query("SELECT 1 as test").fetch_one(pool.get_ref()).await {
        Ok(_) => "connected",
        Err(e) => {
            log::error!("[Webhook Service] Database health check failed: {}", e);
            "disconnected"
        }
    };

    Ok(web::Json(serde_json::json!({
        "status": "healthy",
        "service": "webhook-service",
        "database": db_status,
        "timestamp": chrono::Utc::now()
    })))
}
