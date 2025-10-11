use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;

mod models;
mod handlers;
mod services;
mod errors;

use handlers::auth::configure_auth_routes;
// use handlers::billing::configure_billing_routes;
mod rulesets {
    pub use crate::handlers::rulesets::configure;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv::dotenv().ok();

    let port = env::var("BACKEND_PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()
        .unwrap_or(3001);

    // Database connection
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://conhub:conhub_password@localhost:5432/conhub".to_string());

    println!("📊 Connecting to database: {}", database_url);
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");
    
    println!("✅ Database connection established");

    // Check if migrations are needed
    println!("🔄 Checking database migrations...");
    match sqlx::migrate!("backend/migrations").run(&pool).await {
        Ok(_) => println!("✅ Migrations completed successfully"),
        Err(e) => {
            // If migrations fail due to existing schema, that's okay
            if e.to_string().contains("already exists") {
                println!("⏭️ Database schema already exists, skipping migrations");
            } else {
                println!("⚠️ Migration warning: {}", e);
                println!("⏭️ Continuing with existing schema...");
            }
        }
    }

    println!("🚀 Starting ConHub Backend on port {}", port);

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
            .configure(configure_auth_routes)
            // .configure(configure_billing_routes)
            .configure(rulesets::configure)
            .route("/health", web::get().to(health_check))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

async fn health_check(pool: web::Data<PgPool>) -> actix_web::Result<web::Json<serde_json::Value>> {
    // Test database connectivity
    let db_status = match sqlx::query("SELECT 1 as test").fetch_one(pool.get_ref()).await {
        Ok(_) => "connected",
        Err(e) => {
            log::error!("Database health check failed: {}", e);
            "disconnected"
        }
    };

    Ok(web::Json(serde_json::json!({
        "status": "healthy",
        "service": "conhub-backend",
        "database": db_status,
        "timestamp": chrono::Utc::now()
    })))
}