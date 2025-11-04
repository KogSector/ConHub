mod config;
mod state;
mod routes;
mod services;
mod middleware;
mod models;

use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use sqlx::postgres::PgPoolOptions;
use std::io;
use conhub_middleware::auth::AuthMiddlewareFactory;

use config::AppConfig;
use state::AppState;

#[actix_web::main]
async fn main() -> io::Result<()> {
    // Initialize logger
    env_logger::init();

    log::info!("Starting ConHub Backend Service...");

    // Load configuration from environment
    let config = AppConfig::from_env();
    let port = config.backend_port;

    log::info!("Environment mode: {}", config.env_mode);
    log::info!("Binding to port: {}", port);

    // Database setup
    log::info!("Connecting to PostgreSQL database...");
    let db_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to Postgres");

    log::info!("Connected to PostgreSQL");

    // Redis setup
    log::info!("Connecting to Redis...");
    let redis_client = redis::Client::open(config.redis_url.clone())
        .expect("Failed to create Redis client");

    // Test Redis connection
    let mut redis_conn = redis_client
        .get_connection()
        .expect("Failed to connect to Redis");

    log::info!("Connected to Redis");

    // Initialize authentication middleware
    let auth_middleware = AuthMiddlewareFactory::new()
        .map_err(|e| {
            log::error!("Failed to initialize auth middleware: {}", e);
            std::io::Error::new(std::io::ErrorKind::Other, e)
        })?;

    log::info!("🔐 [Backend Service] Authentication middleware initialized");

    // Initialize application state
    log::info!("Initializing application state...");
    let app_state = AppState::new(db_pool, redis_client, config.clone())
        .await
        .expect("Failed to initialize application state");

    let state_data = web::Data::new(app_state);

    log::info!("Application state initialized");

    // Start HTTP server
    log::info!("Starting HTTP server on 0.0.0.0:{}", port);

    HttpServer::new(move || {
        App::new()
            .app_data(state_data.clone())
            // CORS middleware
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:3000")
                    .allowed_origin("http://localhost:80")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                    .allowed_headers(vec!["Content-Type", "Authorization"])
                    .supports_credentials()
                    .max_age(3600)
            )
            // Logging middleware
            .wrap(actix_web::middleware::Logger::default())
            // Authentication middleware
            .wrap(auth_middleware.clone())
            // Configure all routes
            .configure(routes::configure_routes)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
