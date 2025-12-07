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

    // Initialize authentication middleware (Auth is always required)
    let toggles = FeatureToggles::from_env_path();
    let auth_middleware = AuthMiddlewareFactory::new()
        .map_err(|e| {
            log::error!("Failed to initialize auth middleware: {}", e);
            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
        })?;

    log::info!("🔐 [Backend Service] Authentication middleware initialized");

    // Database setup (always connect when a DB URL is configured)
    let database_url = std::env::var("DATABASE_URL_NEON")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| config.database_url.clone())
        .unwrap_or_else(|| "postgresql://conhub:conhub_password@localhost:5432/conhub".to_string());

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
    let db_pool_opt = Some(db_pool);

    // Redis setup (controlled by Redis feature toggle)
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
        log::warn!("[Backend Service] Redis feature disabled; skipping Redis connection.");
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
            // Middleware execution order is REVERSE of registration order.
            // We want: CORS (first) -> Observability -> Auth (last before routes)
            // So we register: Auth first, then Observability, then CORS last.
            
            // 1. Authentication middleware (innermost - runs last on request, first on response)
            .wrap(auth_middleware.clone())
            // 2. Observability middleware (HTTP logging + tracing)
            .wrap(observability("backend-service"))
            // 3. CORS middleware (outermost - runs first on request, handles OPTIONS preflight)
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:3000")
                    .allowed_origin("https://localhost:3000")
                    .allowed_origin("http://127.0.0.1:3000")
                    .allowed_origin("http://localhost:80")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                    .allow_any_header()
                    .supports_credentials()
                    .max_age(3600),
            )
            // Configure all routes
            .configure(routes::configure_routes)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
