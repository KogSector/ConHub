use actix_web::{web, App, HttpServer};
use conhub_middleware::auth::AuthMiddlewareFactory;
use actix_cors::Cors;
use sqlx::{PgPool, postgres::{PgPoolOptions, PgConnectOptions}};
use std::env;
use std::str::FromStr;
use conhub_observability::{init_tracing, TracingConfig, observability, info, warn, error};

mod services;
mod handlers;
mod errors;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize observability with structured logging
    init_tracing(TracingConfig::for_service("security-service"));

    // Load environment variables
    dotenv::dotenv().ok();

    // Service port
    let port = env::var("SECURITY_SERVICE_PORT")
        .unwrap_or_else(|_| "3014".to_string())
        .parse::<u16>()
        .unwrap_or(3014);

    // Feature toggles: gate database connection when Auth is disabled
    let toggles = conhub_config::feature_toggles::FeatureToggles::from_env_path();
    let auth_enabled = toggles.auth_enabled();

    let db_pool_opt: Option<PgPool> = if auth_enabled {
        // Prefer Neon URL if present, fall back to DATABASE_URL, then local default
        let database_url = env::var("DATABASE_URL_NEON")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .or_else(|| env::var("DATABASE_URL").ok())
            .unwrap_or_else(|| "postgresql://conhub:conhub_password@localhost:5432/conhub".to_string());

        if env::var("DATABASE_URL_NEON").ok().filter(|v| !v.trim().is_empty()).is_some() {
            info!("📊 [Security Service] Connecting to Neon DB...");
        } else {
            info!("📊 [Security Service] Connecting to database...");
        }
        
        // Disable server-side prepared statements for pgbouncer/Neon transaction pooling
        let connect_options = PgConnectOptions::from_str(&database_url)?
            .statement_cache_capacity(0);
        
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect_with(connect_options)
            .await?;
        info!("✅ [Security Service] Database connection established");
        Some(pool)
    } else {
        info!("[Security Service] Auth disabled; skipping database connection.");
        None
    };

    info!("🚀 [Security Service] Starting on port {}", port);

    let auth_middleware = match AuthMiddlewareFactory::new() {
        Ok(m) => m,
        Err(e) => {
            warn!("[Security Service] Auth middleware init failed: {}. Using dev claims.", e);
            AuthMiddlewareFactory::disabled()
        }
    };

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .app_data(web::Data::new(db_pool_opt.clone()))
            .wrap(cors)
            .wrap(observability("security-service"))
            .wrap(auth_middleware.clone())
            .configure(configure_routes)
            .configure(handlers::connections::configure_routes)
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

async fn health_check(pool_opt: web::Data<Option<PgPool>>) -> actix_web::Result<web::Json<serde_json::Value>> {
    let db_status = match pool_opt.get_ref() {
        Some(pool) => match sqlx::query("SELECT 1 as test").fetch_one(pool).await {
            Ok(_) => "connected",
            Err(e) => {
                log::error!("[Security Service] Database health check failed: {}", e);
                "disconnected"
            }
        },
        None => "disabled",
    };

    Ok(web::Json(serde_json::json!({
        "status": "healthy",
        "service": "security-service",
        "database": db_status,
        "timestamp": chrono::Utc::now()
    })))
}
