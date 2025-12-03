mod config;
mod state;
mod routes;
mod services;
mod middleware;
mod models;
mod graphql;
mod handlers;

use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use sqlx::postgres::{PgPoolOptions, PgConnectOptions};
use std::io;
use std::str::FromStr;
use conhub_middleware::auth::AuthMiddlewareFactory;
use conhub_config::feature_toggles::FeatureToggles;
use conhub_observability::{init_tracing, TracingConfig, observability, info, warn, error};

use config::AppConfig;
use state::AppState;
use crate::graphql::schema::build_schema;

#[actix_web::main]
async fn main() -> io::Result<()> {
    // Initialize observability with structured logging
    init_tracing(TracingConfig::for_service("backend-service"));

    info!("Starting ConHub Backend Service...");

    // Load configuration from environment
    let config = AppConfig::from_env();
    let port = config.backend_port;

    log::info!("Environment mode: {}", config.env_mode);
    log::info!("Binding to port: {}", port);

    // Initialize authentication middleware with feature toggle
    let toggles = FeatureToggles::from_env_path();
    let auth_enabled = toggles.auth_enabled();
    let auth_middleware = if auth_enabled {
        AuthMiddlewareFactory::new()
            .map_err(|e| {
                log::error!("Failed to initialize auth middleware: {}", e);
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
            })?
    } else {
        log::warn!("Auth feature disabled via feature toggles; injecting default claims.");
        AuthMiddlewareFactory::disabled()
    };

    log::info!("🔐 [Backend Service] Authentication middleware initialized");

    // Database setup (gated by Auth toggle)
    let db_pool_opt = if auth_enabled {
        // Prefer Neon URL if present, fall back to DATABASE_URL
        let database_url = std::env::var("DATABASE_URL_NEON")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .or_else(|| config.database_url.clone())
            .expect("DATABASE_URL or DATABASE_URL_NEON must be set when Auth is enabled");

        if std::env::var("DATABASE_URL_NEON").ok().filter(|v| !v.trim().is_empty()).is_some() {
            log::info!("📊 [Backend Service] Connecting to Neon DB...");
        } else {
            log::info!("Connecting to PostgreSQL database...");
        }
        
        // Disable server-side prepared statements for pgbouncer/Neon transaction pooling
        let connect_options = PgConnectOptions::from_str(&database_url)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
            .statement_cache_capacity(0);
        
        let db_pool = PgPoolOptions::new()
            .max_connections(10)
            .connect_with(connect_options)
            .await
            .expect("Failed to connect to Postgres");
        log::info!("✅ [Backend Service] Database connection established");
        Some(db_pool)
    } else {
        log::warn!("Auth disabled; skipping PostgreSQL connection.");
        None
    };

    // Redis setup (gated by Auth and Redis toggles)
    let redis_enabled = toggles.should_connect_redis();
    let redis_client = if redis_enabled {
        if let Some(redis_url) = config.redis_url.clone() {
            log::info!("📊 [Backend Service] Connecting to Redis...");
            match redis::Client::open(redis_url.clone()) {
                Ok(client) => {
                    // Test Redis connection
                    match client.get_connection() {
                        Ok(_) => {
                            log::info!("✅ [Backend Service] Connected to Redis");
                            Some(client)
                        }
                        Err(e) => {
                            log::warn!("⚠️  [Backend Service] Failed to connect to Redis: {}", e);
                            log::warn!("⚠️  [Backend Service] Service will continue without Redis");
                            None
                        }
                    }
                }
                Err(e) => {
                    log::warn!("⚠️  [Backend Service] Failed to create Redis client: {}", e);
                    log::warn!("⚠️  [Backend Service] Service will continue without Redis");
                    None
                }
            }
        } else {
            log::warn!("⚠️  [Backend Service] REDIS_URL not set, skipping Redis connection");
            None
        }
    } else {
        if !auth_enabled {
            log::warn!("[Backend Service] Auth disabled; skipping Redis connection.");
        } else {
            log::warn!("[Backend Service] Redis feature disabled; skipping Redis connection.");
        }
        None
    };

    // Initialize RAG service
    let embedding_url = std::env::var("EMBEDDING_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:8082".to_string());
    let graph_url = std::env::var("GRAPH_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:8006".to_string());
    let agentic_url = std::env::var("AGENTIC_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:3005".to_string());
    
    let rag_service = std::sync::Arc::new(services::rag_service::RagService::new(
        embedding_url.clone(),
        graph_url.clone(),
        agentic_url.clone(),
    ));
    log::info!("🤖 [Backend Service] RAG service initialized");
    log::info!("   Embedding: {}", embedding_url);
    log::info!("   Graph: {}", graph_url);
    log::info!("   Agentic: {}", agentic_url);

    // Initialize application state
    log::info!("Initializing application state...");
    let app_state = AppState::new(db_pool_opt, redis_client, config.clone())
        .await
        .expect("Failed to initialize application state");

    let state_data = web::Data::new(app_state);
    let rag_data = web::Data::new(rag_service);

    log::info!("Application state initialized");

    // Start HTTP server
    log::info!("Starting HTTP server on 0.0.0.0:{}", port);

    HttpServer::new(move || {
        let schema = build_schema(config.clone(), toggles.clone());

        App::new()
            .app_data(state_data.clone())
            .app_data(rag_data.clone())
            .app_data(web::Data::new(toggles.clone()))
            .app_data(web::Data::new(schema))
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
            // Observability middleware (HTTP logging + tracing)
            .wrap(observability("backend-service"))
            // Authentication middleware
            .wrap(auth_middleware.clone())
            // Configure all routes
            .configure(routes::configure_routes)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
