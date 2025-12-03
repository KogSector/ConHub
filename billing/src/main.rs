use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use sqlx::{PgPool, postgres::{PgPoolOptions, PgConnectOptions}};
use redis::Client as RedisClient;
use std::str::FromStr;
use std::env;
use conhub_middleware::auth::AuthMiddlewareFactory;
use conhub_config::feature_toggles::FeatureToggles;
use conhub_observability::{init_tracing, TracingConfig, observability, info, warn, error};

mod handlers;
mod services;
mod errors;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize observability with structured logging
    init_tracing(TracingConfig::for_service("billing-service"));

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

    // Database connection - always required
    let database_url = env::var("DATABASE_URL_NEON")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| env::var("DATABASE_URL").ok())
        .ok_or_else(|| "DATABASE_URL or DATABASE_URL_NEON must be set")?;

    if env::var("DATABASE_URL_NEON").ok().filter(|v| !v.trim().is_empty()).is_some() {
        tracing::info!("📊 [Billing Service] Connecting to Neon DB...");
    } else {
        tracing::info!("📊 [Billing Service] Connecting to database...");
    }

    // Disable server-side prepared statements for pgbouncer/Neon
    let connect_options = PgConnectOptions::from_str(&database_url)?
        .statement_cache_capacity(0);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect_with(connect_options)
        .await?;
    tracing::info!("✅ [Billing Service] Database connection established");
    let db_pool_opt = Some(pool);

    // Stripe API key (optional in development)
    let stripe_key_opt: Option<String> = match env::var("STRIPE_SECRET_KEY") {
        Ok(key) => {
            tracing::info!("💳 [Billing Service] Stripe configuration loaded");
            Some(key)
        },
        Err(_) => {
            tracing::warn!("[Billing Service] STRIPE_SECRET_KEY not set - Stripe features disabled");
            None
        }
    };

    // Redis connection
    let redis_client_opt: Option<RedisClient> = match env::var("REDIS_URL") {
        Ok(redis_url) => {
            tracing::info!("🔴 [Billing Service] Connecting to Redis...");
            match RedisClient::open(redis_url) {
                Ok(client) => match client.get_connection() {
                    Ok(_) => Some(client),
                    Err(_) => None,
                },
                Err(_) => None,
            }
        }
        Err(_) => {
            tracing::warn!("[Billing Service] REDIS_URL not set - Redis features disabled");
            None
        }
    };

    tracing::info!("🚀 [Billing Service] Starting on port {}", port);
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        let mut app = App::new()
            .app_data(web::Data::new(db_pool_opt.clone()))
            .app_data(web::Data::new(stripe_key_opt.clone()))
            .app_data(web::Data::new(redis_client_opt.clone()))
            .wrap(cors)
            .wrap(observability("billing-service"))
            .wrap(auth_middleware.clone())
            .route("/health", web::get().to(health_check));

        app = app.configure(handlers::billing::configure_billing_routes);

        app
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
