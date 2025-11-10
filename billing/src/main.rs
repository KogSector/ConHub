use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;
use tracing::{info, error};
use tracing_subscriber;
use conhub_middleware::auth::AuthMiddlewareFactory;
use conhub_config::feature_toggles::FeatureToggles;

mod handlers;
mod services;
mod errors;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Service port
    let port = env::var("BILLING_SERVICE_PORT")
        .unwrap_or_else(|_| "3011".to_string())
        .parse::<u16>()
        .unwrap_or(3011);

    // Initialize authentication middleware with feature toggle
    let toggles = FeatureToggles::from_env_path();
    let auth_enabled = toggles.auth_enabled();
    let auth_middleware = if auth_enabled {
        AuthMiddlewareFactory::new()
            .map_err(|e| {
                tracing::error!("Failed to initialize auth middleware: {}", e);
                e
            })?
    } else {
        tracing::warn!("Auth feature disabled via feature toggles; injecting default claims.");
        AuthMiddlewareFactory::disabled()
    };

    tracing::info!("🔐 [Billing Service] Authentication middleware initialized");

    // Database connection (gated by Auth toggle)
    let db_pool_opt: Option<PgPool> = if auth_enabled {
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set when Auth is enabled");

        tracing::info!("📊 [Billing Service] Connecting to database...");
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
            .await?;
        tracing::info!("✅ [Billing Service] Database connection established");
        Some(pool)
    } else {
        tracing::warn!("[Billing Service] Auth disabled; skipping database connection.");
        None
    };

    // Stripe API key (optional in local dev when Auth is disabled)
    let stripe_key_opt: Option<String> = if auth_enabled {
        match env::var("STRIPE_SECRET_KEY") {
            Ok(key) => Some(key),
            Err(_) => {
                tracing::error!("[Billing Service] STRIPE_SECRET_KEY must be set when Auth is enabled");
                return Err("Missing STRIPE_SECRET_KEY".into());
            }
        }
    } else {
        tracing::warn!("[Billing Service] Auth disabled; skipping Stripe configuration.");
        None
    };

    tracing::info!("🚀 [Billing Service] Starting on port {}", port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .app_data(web::Data::new(db_pool_opt.clone()))
            .app_data(web::Data::new(stripe_key_opt.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(auth_middleware.clone())
            .configure(handlers::billing::configure_billing_routes)
            .route("/health", web::get().to(health_check))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}

// Routes are configured in handlers::billing::configure_billing_routes

async fn health_check(pool_opt: web::Data<Option<PgPool>>) -> actix_web::Result<web::Json<serde_json::Value>> {
    let db_status = match pool_opt.get_ref() {
        Some(pool) => match sqlx::query("SELECT 1 as test").fetch_one(pool).await {
            Ok(_) => "connected",
            Err(e) => {
                tracing::error!("[Billing Service] Database health check failed: {}", e);
                "disconnected"
            }
        },
        None => "disabled",
    };

    Ok(web::Json(serde_json::json!({
        "status": "healthy",
        "service": "billing-service",
        "database": db_status,
        "timestamp": chrono::Utc::now()
    })))
}
