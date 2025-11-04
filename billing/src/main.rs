use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;
use tracing::{info, error};
use tracing_subscriber;
use conhub_middleware::auth::AuthMiddlewareFactory;

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

    // Database connection
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://conhub:conhub_password@postgres:5432/conhub".to_string());

    tracing::info!("📊 [Billing Service] Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    tracing::info!("✅ [Billing Service] Database connection established");

    // Stripe API key
    let stripe_key = env::var("STRIPE_SECRET_KEY")
        .expect("STRIPE_SECRET_KEY must be set");

    // Initialize authentication middleware
    let auth_middleware = AuthMiddlewareFactory::new()
        .map_err(|e| {
            tracing::error!("Failed to initialize auth middleware: {}", e);
            e
        })?;

    tracing::info!("🔐 [Billing Service] Authentication middleware initialized");
    tracing::info!("🚀 [Billing Service] Starting on port {}", port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(stripe_key.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(auth_middleware.clone())
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
        web::scope("/api/billing")
            .route("/subscription", web::post().to(handlers::billing::create_subscription))
            .route("/subscription/{id}", web::get().to(handlers::billing::get_subscription))
            .route("/subscription/{id}", web::delete().to(handlers::billing::cancel_subscription))
            .route("/payment-method", web::post().to(handlers::billing::add_payment_method))
            .route("/invoices", web::get().to(handlers::billing::get_invoices))
    );
}

async fn health_check(pool: web::Data<PgPool>) -> actix_web::Result<web::Json<serde_json::Value>> {
    let db_status = match sqlx::query("SELECT 1 as test").fetch_one(pool.get_ref()).await {
        Ok(_) => "connected",
        Err(e) => {
            tracing::error!("[Billing Service] Database health check failed: {}", e);
            "disconnected"
        }
    };

    Ok(web::Json(serde_json::json!({
        "status": "healthy",
        "service": "billing-service",
        "database": db_status,
        "timestamp": chrono::Utc::now()
    })))
}
