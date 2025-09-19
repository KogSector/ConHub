mod config;
mod handlers;
mod middleware;
mod models;
mod services;

use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use sqlx::PgPool;
use std::sync::Arc;
use config::AppConfig;
use services::auth_service::AuthService;
use services::session_service::{SessionService, session_cleanup_task};
use services::feature_toggle_service::{FeatureToggleService, watch_feature_toggles};
use services::social_integration_service::SocialIntegrationService;
use services::rule_bank::AIRuleBankService;
use middleware::auth::AuthMiddlewareFactory;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    env_logger::init();
    
    let config = AppConfig::from_env();
    
    // Initialize database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/conhub".to_string());
    
    let db_pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");
    
    // Note: Database schema is already set up via scripts/database/schema.sql
    // Run migrations/setup only if needed for development
    // sqlx::query(
    //     "CREATE TABLE IF NOT EXISTS users (
    //         id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    //         email VARCHAR(255) UNIQUE NOT NULL,
    //         password_hash VARCHAR(255) NOT NULL,
    //         name VARCHAR(255) NOT NULL,
    //         avatar_url TEXT,
    //         organization VARCHAR(255),
    //         role VARCHAR(50) NOT NULL DEFAULT 'user',
    //         subscription_tier VARCHAR(50) NOT NULL DEFAULT 'free',
    //         is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    //         is_active BOOLEAN NOT NULL DEFAULT TRUE,
    //         created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    //         updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    //         last_login_at TIMESTAMP WITH TIME ZONE
    //     )"
    // )
    // .execute(&db_pool)
    // .await
    // .expect("Failed to create users table");

    // Run social platform migrations
    // Note: Skip migrations since we have comprehensive schema already applied
    // let social_migration = include_str!("../migrations/001_social_tables.sql");
    // sqlx::query(social_migration)
    //     .execute(&db_pool)
    //     .await
    //     .expect("Failed to run social platform migrations");

    // Initialize auth service
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-super-secret-jwt-key-change-this-in-production".to_string());
    
    let auth_service = AuthService::new(db_pool.clone(), jwt_secret);
    // auth_service.init_database().await.expect("Failed to initialize auth database");
    
    // Initialize session service
    let session_service = Arc::new(SessionService::new());
    
    // Initialize feature toggle service
    let feature_toggle_service = Arc::new(FeatureToggleService::new("./feature-toggles.json"));
    
    // Initialize AI Rule Bank service
    let rule_bank_service = Arc::new(AIRuleBankService::new(db_pool.clone()));
    
    // Initialize feature toggles
    if let Err(e) = feature_toggle_service.init().await {
        log::warn!("Failed to load initial feature toggles: {}", e);
    }
    
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
    let auth_service_data = web::Data::new(auth_service.clone());
    let session_service_data = web::Data::new(session_service.as_ref().clone());
    let feature_toggle_service_data = web::Data::new(feature_toggle_service.as_ref().clone());
    let rule_bank_service_data = web::Data::new((*rule_bank_service).clone());
    let social_service = web::Data::new(SocialIntegrationService::new(db_pool.clone()));
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
            .app_data(rule_bank_service_data.clone())
            .app_data(social_service.clone())
            .app_data(db_data.clone())
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(AuthMiddlewareFactory::new(Arc::new(auth_service.clone())))
            .configure(handlers::configure_routes)
    })
    .bind("127.0.0.1:3001")?
    .run()
    .await
}