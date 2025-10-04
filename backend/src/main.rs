mod config;
mod errors;
mod handlers;
mod middleware;
mod models;
mod services;
mod utils;
mod sources;
mod agents;

use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use sqlx::PgPool;
use std::sync::{Arc, Mutex};
use config::AppConfig;
use services::auth_service::AuthService;
use services::session_service::{SessionService, session_cleanup_task};
use services::feature_toggle_service::{FeatureToggleService, watch_feature_toggles};
use services::social_integration_service::SocialIntegrationService;
use services::rule_bank::AIRuleBankService;
use services::ai_service::AIAgentManager;
use middleware::auth::AuthMiddlewareFactory;
use utils::logging::{init_logging, LoggingConfig, PerformanceMonitor, log_startup_info, log_config_info};
use tracing::{info, error};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    // Initialize advanced logging
    let logging_config = LoggingConfig::from_env();
    if let Err(e) = init_logging(logging_config) {
        eprintln!("Failed to initialize logging: {}", e);
        std::process::exit(1);
    }
    
    log_startup_info();
    log_config_info();
    
    let config = AppConfig::from_env();
    
    // Initialize performance monitoring
    let performance_monitor = PerformanceMonitor::new();
    performance_monitor.start_monitoring().await;
    
    // Initialize database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/conhub".to_string());
    
    info!(database_url = %database_url, "Connecting to database");
    
    let db_pool = PgPool::connect(&database_url)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to connect to database");
            std::io::Error::new(std::io::ErrorKind::ConnectionRefused, e)
        })?;
    
    info!("Database connection established successfully");
    
    // Note: Database schema is already set up via scripts/database/schema.sql
    // Run migrations/setup only if needed for development
    
    // Initialize auth service
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| {
            info!("Using default JWT secret - ensure you set JWT_SECRET in production");
            "your-super-secret-jwt-key-change-this-in-production".to_string()
        });
    
    let auth_service = AuthService::new(db_pool.clone(), jwt_secret);
    info!("Auth service initialized");
    
    // Initialize session service
    let session_service = Arc::new(SessionService::new());
    info!("Session service initialized");
    
    // Initialize feature toggle service
    let feature_toggle_service = Arc::new(FeatureToggleService::new("./feature-toggles.json"));
    
    // Initialize AI Rule Bank service
    let rule_bank_service = Arc::new(AIRuleBankService::new(db_pool.clone()));
    info!("AI Rule Bank service initialized");

    // Initialize AI Agent Manager
    let ai_agent_manager = Arc::new(Mutex::new(AIAgentManager::new()));
    info!("AI Agent Manager initialized");
    
    // Initialize feature toggles
    if let Err(e) = feature_toggle_service.init().await {
        error!(error = %e, "Failed to load initial feature toggles, using defaults");
    } else {
        info!("Feature toggles loaded successfully");
    }
    
    // Start background tasks
    let session_service_clone = session_service.clone();
    tokio::spawn(async move {
        info!("Starting session cleanup task");
        session_cleanup_task(session_service_clone).await;
    });
    
    let feature_toggle_service_clone = feature_toggle_service.clone();
    tokio::spawn(async move {
        info!("Starting feature toggle watcher");
        watch_feature_toggles(feature_toggle_service_clone).await;
    });
    
    let app_state = web::Data::new(config.clone());
    let auth_service_data = web::Data::new(auth_service.clone());
    let session_service_data = web::Data::new(session_service.as_ref().clone());
    let feature_toggle_service_data = web::Data::new(feature_toggle_service.as_ref().clone());
    let rule_bank_service_data = web::Data::new((*rule_bank_service).clone());
    let social_service = web::Data::new(SocialIntegrationService::new(db_pool.clone()));
    let db_data = web::Data::new(db_pool);
    let ai_agent_manager_data = web::Data::from(ai_agent_manager.clone());
    
    info!(port = 3001, "Starting ConHub Backend server");
    info!("Configuring HTTP server with CORS and middleware");
    
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
            .app_data(ai_agent_manager_data.clone())
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(AuthMiddlewareFactory::new(Arc::new(auth_service.clone())))
            .configure(handlers::configure_routes)
    })
    .bind("127.0.0.1:3001")?
    .run()
    .await
}
