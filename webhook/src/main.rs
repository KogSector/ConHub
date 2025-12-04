use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use sqlx::{PgPool, postgres::{PgPoolOptions, PgConnectOptions}};
use std::str::FromStr;
use std::env;
use conhub_observability::{init_tracing, TracingConfig, observability, info, warn, error};

mod handlers;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize observability with structured logging
    init_tracing(TracingConfig::for_service("webhook-service"));

    // Load environment variables
    dotenv::dotenv().ok();

    // Service port
    let port = env::var("WEBHOOK_SERVICE_PORT")
        .unwrap_or_else(|_| "3015".to_string())
        .parse::<u16>()
        .unwrap_or(3015);

    let toggles = conhub_config::feature_toggles::FeatureToggles::from_env_path();
    let auth_enabled = toggles.auth_enabled();

    let pool = if auth_enabled {
        let database_url = env::var("DATABASE_URL_NEON")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .or_else(|| env::var("DATABASE_URL").ok())
            .unwrap_or_else(|| "postgresql://conhub:conhub_password@localhost:5432/conhub".to_string());

        if env::var("DATABASE_URL_NEON").ok().filter(|v| !v.trim().is_empty()).is_some() {
            info!("📊 [Webhook Service] Connecting to Neon DB...");
        } else {
            info!("📊 [Webhook Service] Connecting to database...");
        }

        let connect_options = PgConnectOptions::from_str(&database_url)?
            .statement_cache_capacity(0);

        match PgPoolOptions::new()
            .max_connections(10)
            .connect_with(connect_options)
            .await {
            Ok(p) => {
                info!("✅ [Webhook Service] Database connection established");
                Some(p)
            }
            Err(e) => {
                error!("⚠️  [Webhook Service] Failed to connect to database: {}", e);
                None
            }
        }
    } else {
        info!("[Webhook Service] Auth disabled; skipping database connection.");
        None
    };

    info!("🚀 [Webhook Service] Starting on port {}", port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        let mut app = App::new()
            .wrap(cors)
            .wrap(observability("webhook-service"));

        if let Some(p) = pool.clone() {
            app = app.app_data(web::Data::new(p));
        }

        app.configure(configure_routes)
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

async fn health_check(pool: Option<web::Data<PgPool>>) -> actix_web::Result<web::Json<serde_json::Value>> {
    let db_status = match pool {
        Some(p) => {
            match sqlx::query("SELECT 1 as test").fetch_one(p.get_ref()).await {
                Ok(_) => "connected",
                Err(e) => {
                    log::error!("[Webhook Service] Database health check failed: {}", e);
                    "disconnected"
                }
            }
        }
        None => "disabled"
    };

    Ok(web::Json(serde_json::json!({
        "status": "healthy",
        "service": "webhook-service",
        "database": db_status,
        "timestamp": chrono::Utc::now()
    })))
}
