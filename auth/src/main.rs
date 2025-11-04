use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;
use tracing_subscriber;

mod services;
mod handlers;

use services::{AuthMiddlewareFactory, role_auth_middleware};

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Service port
    let port = env::var("AUTH_SERVICE_PORT")
        .unwrap_or_else(|_| "3010".to_string())
        .parse::<u16>()
        .unwrap_or(3010);

    // Database connection
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://conhub:conhub_password@postgres:5432/conhub".to_string());

    println!("📊 [Auth Service] Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    println!("✅ [Auth Service] Database connection established");

    // Redis connection for sessions
    let redis_url = env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://redis:6379".to_string());

    let redis_client = redis::Client::open(redis_url.clone())?;
    println!("✅ [Auth Service] Redis connection established");

    println!("🚀 [Auth Service] Starting on port {}", port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(redis_client.clone()))
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
    use conhub_models::auth::UserRole;
    
    cfg.service(
        web::scope("/api/auth")
            .route("/login", web::post().to(handlers::auth::login))
            .route("/register", web::post().to(handlers::auth::register))
            .route("/forgot-password", web::post().to(handlers::auth::forgot_password))
            .route("/reset-password", web::post().to(handlers::auth::reset_password))
            .service(
                web::scope("")
                    .wrap(AuthMiddlewareFactory::new())
                    .route("/logout", web::post().to(handlers::auth::logout))
                    .route("/me", web::get().to(handlers::auth::get_current_user))
                    .route("/refresh", web::post().to(handlers::auth::refresh_token))
                    .route("/profile", web::get().to(handlers::auth::get_profile))
                    .service(
                        web::scope("/admin")
                            .wrap(role_auth_middleware(vec![UserRole::Admin]))
                            .route("/users", web::get().to(handlers::auth::list_users))
                    )
            )
            // OAuth routes (public)
            .route("/oauth/{provider}", web::get().to(handlers::oauth::oauth_login))
            .route("/oauth/{provider}/callback", web::get().to(handlers::oauth::oauth_callback))
    );
}

async fn health_check(pool: web::Data<PgPool>) -> actix_web::Result<web::Json<serde_json::Value>> {
    // Check database connection
    let db_status = match sqlx::query("SELECT 1 as test").fetch_one(pool.get_ref()).await {
        Ok(_) => "connected",
        Err(e) => {
            tracing::error!("[Auth Service] Database health check failed: {}", e);
            "disconnected"
        }
    };

    Ok(web::Json(serde_json::json!({
        "status": "healthy",
        "service": "auth-service",
        "database": db_status,
        "timestamp": chrono::Utc::now()
    })))
}
