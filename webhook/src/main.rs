use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use sqlx::{PgPool, postgres::{PgPoolOptions, PgConnectOptions}};
use std::str::FromStr;
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

    // Database connection with graceful degradation (prefer Neon)
    let database_url = env::var("DATABASE_URL_NEON")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| env::var("DATABASE_URL").ok())
        .unwrap_or_else(|| "postgresql://conhub:conhub_password@localhost:5432/conhub".to_string());

    if env::var("DATABASE_URL_NEON").ok().filter(|v| !v.trim().is_empty()).is_some() {
        println!("📊 [Webhook Service] Connecting to Neon DB...");
    } else {
        println!("📊 [Webhook Service] Connecting to database...");
    }

    // Disable server-side prepared statements for pgbouncer/Neon
    let connect_options = PgConnectOptions::from_str(&database_url)?
        .statement_cache_capacity(0);

    let pool_result = PgPoolOptions::new()
        .max_connections(10)
        .connect_with(connect_options)
        .await;

    let pool = match pool_result {
        Ok(p) => {
            println!("✅ [Webhook Service] Database connection established");
            Some(p)
        }
        Err(e) => {
            eprintln!("⚠️  [Webhook Service] Failed to connect to database: {}", e);
            eprintln!("⚠️  [Webhook Service] Service will start but database operations will fail");
            eprintln!("⚠️  [Webhook Service] Please ensure PostgreSQL is running and DATABASE_URL is correct");
            None
        }
    };

    println!("🚀 [Webhook Service] Starting on port {}", port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        let mut app = App::new()
            .wrap(cors)
            .wrap(Logger::default());

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
        None => "not_configured"
    };

    Ok(web::Json(serde_json::json!({
        "status": "healthy",
        "service": "webhook-service",
        "database": db_status,
        "timestamp": chrono::Utc::now()
    })))
}
