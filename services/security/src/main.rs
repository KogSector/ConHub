use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;

mod services;
mod handlers;
mod errors;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Service port
    let port = env::var("SECURITY_SERVICE_PORT")
        .unwrap_or_else(|_| "3014".to_string())
        .parse::<u16>()
        .unwrap_or(3014);

    // Database connection
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://conhub:conhub_password@postgres:5432/conhub".to_string());

    println!("📊 [Security Service] Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    println!("✅ [Security Service] Database connection established");

    println!("🚀 [Security Service] Starting on port {}", port);

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
        web::scope("/api/security")
            .route("/rulesets", web::get().to(handlers::rulesets::list_rulesets))
            .route("/rulesets", web::post().to(handlers::rulesets::create_ruleset))
            .route("/rulesets/{id}", web::get().to(handlers::rulesets::get_ruleset))
            .route("/rulesets/{id}", web::put().to(handlers::rulesets::update_ruleset))
            .route("/rulesets/{id}", web::delete().to(handlers::rulesets::delete_ruleset))
            .route("/audit-logs", web::get().to(handlers::security::get_audit_logs))
    );
}

async fn health_check(pool: web::Data<PgPool>) -> actix_web::Result<web::Json<serde_json::Value>> {
    let db_status = match sqlx::query("SELECT 1 as test").fetch_one(pool.get_ref()).await {
        Ok(_) => "connected",
        Err(e) => {
            log::error!("[Security Service] Database health check failed: {}", e);
            "disconnected"
        }
    };

    Ok(web::Json(serde_json::json!({
        "status": "healthy",
        "service": "security-service",
        "database": db_status,
        "timestamp": chrono::Utc::now()
    })))
}
