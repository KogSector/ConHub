use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use sqlx::{PgPool, postgres::{PgPoolOptions, PgConnectOptions}};
use std::str::FromStr;
use std::env;
use tracing::{info, error};
use tracing_subscriber;
use conhub_config::feature_toggles::FeatureToggles;

mod services;
mod handlers;
mod agents;
mod llm;
mod utils;
mod prelude;
mod base;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Service port
    let port = env::var("AI_SERVICE_PORT")
        .unwrap_or_else(|_| "3012".to_string())
        .parse::<u16>()
        .unwrap_or(3012);

    // Feature toggles: gate database connection when Auth is disabled
    let toggles = FeatureToggles::from_env_path();
    let auth_enabled = toggles.auth_enabled();

    let db_pool_opt: Option<PgPool> = if auth_enabled {
        // Prefer Neon if provided; fall back to DATABASE_URL or local default
        let database_url = env::var("DATABASE_URL_NEON")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .or_else(|| env::var("DATABASE_URL").ok())
            .unwrap_or_else(|| "postgresql://conhub:conhub_password@postgres:5432/conhub".to_string());

        if env::var("DATABASE_URL_NEON").ok().filter(|v| !v.trim().is_empty()).is_some() {
            tracing::info!("📊 [AI Service] Connecting to Neon DB...");
        } else {
            tracing::info!("📊 [AI Service] Connecting to database...");
        }

        // Disable server-side prepared statements for pgbouncer/Neon transaction pooling
        let connect_options = PgConnectOptions::from_str(&database_url)?
            .statement_cache_capacity(0);

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect_with(connect_options)
            .await?;
        tracing::info!("✅ [AI Service] Database connection established");
        Some(pool)
    } else {
        tracing::warn!("[AI Service] Auth disabled; skipping database connection.");
        None
    };

    tracing::info!("🚀 [AI Service] Starting on port {}", port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .app_data(web::Data::new(db_pool_opt.clone()))
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
        web::scope("/api/ai")
            .route("/agents", web::get().to(handlers::get_agents))
            .route("/query", web::post().to(handlers::query_agents))
    );
}

async fn health_check(pool_opt: web::Data<Option<PgPool>>) -> actix_web::Result<web::Json<serde_json::Value>> {
    let db_status = match pool_opt.get_ref() {
        Some(pool) => match sqlx::query("SELECT 1 as test").fetch_one(pool).await {
            Ok(_) => "connected",
            Err(e) => {
                tracing::error!("[AI Service] Database health check failed: {}", e);
                "disconnected"
            }
        },
        None => "disabled",
    };

    Ok(web::Json(serde_json::json!({
        "status": "healthy",
        "service": "ai-service",
        "database": db_status,
        "timestamp": chrono::Utc::now()
    })))
}
