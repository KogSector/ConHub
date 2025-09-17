mod config;
mod handlers;
mod middleware;
mod models;
mod services;

use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use sqlx::SqlitePool;
use std::sync::Arc;
use config::AppConfig;
use services::auth_service::AuthService;
use services::session_service::{SessionService, session_cleanup_task};
use services::feature_toggle_service::{FeatureToggleService, watch_feature_toggles};
use middleware::auth::AuthMiddleware;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let config = AppConfig::from_env();
    
    // Initialize database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./conhub.db".to_string());
    
    let db_pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to connect to database");
    
    // Run migrations/setup
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            name TEXT NOT NULL,
            avatar_url TEXT,
            organization TEXT,
            role TEXT NOT NULL DEFAULT 'user',
            subscription_tier TEXT NOT NULL DEFAULT 'free',
            is_verified BOOLEAN NOT NULL DEFAULT FALSE,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            last_login_at TEXT
        )"
    )
    .execute(&db_pool)
    .await
    .expect("Failed to create users table");

    // Initialize auth service
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-super-secret-jwt-key-change-this-in-production".to_string());
    
    let auth_service = AuthService::new(db_pool.clone(), jwt_secret);
    auth_service.init_database().await.expect("Failed to initialize auth database");
    
    // Initialize session service
    let session_service = Arc::new(SessionService::new());
    
    // Initialize feature toggle service
    let feature_toggle_service = Arc::new(FeatureToggleService::new("./feature-toggles.json"));
    
    // Start background tasks
    let session_service_clone = session_service.clone();
    tokio::spawn(async move {
        session_cleanup_task(session_service_clone).await;
    });
    
    let feature_toggle_service_clone = feature_toggle_service.clone();
    tokio::spawn(async move {
        watch_feature_toggles(feature_toggle_service_clone).await;
    });
    
    let app_state = web::Data::new(config.clone());
    let auth_service_data = web::Data::new(auth_service);
    let session_service_data = web::Data::new(session_service.as_ref().clone());
    let feature_toggle_service_data = web::Data::new(feature_toggle_service.as_ref().clone());
    let db_data = web::Data::new(db_pool);
    
    println!("Starting ConHub Backend on http://localhost:3001");
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://127.0.0.1:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec!["Content-Type", "Authorization", "Accept"])
            .supports_credentials()
            .max_age(3600);
            
        App::new()
            .app_data(app_state.clone())
            .app_data(auth_service_data.clone())
            .app_data(session_service_data.clone())
            .app_data(feature_toggle_service_data.clone())
            .app_data(db_data.clone())
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(AuthMiddleware)
            .configure(handlers::configure_routes)
    })
    .bind("127.0.0.1:3001")?
    .run()
    .await
}