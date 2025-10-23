use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;

mod models;
mod handlers;
mod services;
mod errors;
use handlers::auth::configure_auth_routes;

use handlers::security::rulesets;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let port = env::var("BACKEND_PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()
        .unwrap_or(3001);

    
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://conhub:conhub_password@localhost:5432/conhub".to_string());

    println!("📊 Connecting to database: {}", database_url);
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;
    
    println!("✅ Database connection established");

    
    println!("🔄 Checking database migrations...");
    match sqlx::migrate!("backend/migrations").run(&pool).await {
        Ok(_) => println!("✅ Migrations completed successfully"),
        Err(e) => {
            
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
            
            .configure(rulesets::configure)
            .route("/health", web::get().to(health_check))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}

async fn health_check(pool: web::Data<PgPool>) -> actix_web::Result<web::Json<serde_json::Value>> {
    
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
