use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;
use tracing::{info, error};
use tracing_subscriber;
use conhub_middleware::auth::AuthMiddlewareFactory;
use conhub_config::feature_toggles::FeatureToggles;

mod server;
mod protocol;
mod handlers;
mod services;
mod types;
mod context;
mod error;

use server::MCPServer;
use context::ContextManager;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Service port
    let port = env::var("MCP_SERVICE_PORT")
        .unwrap_or_else(|_| "3030".to_string())
        .parse::<u16>()
        .unwrap_or(3030);

    // Initialize authentication middleware with feature toggle
    let toggles = FeatureToggles::from_env_path();
    let auth_enabled = toggles.is_enabled_or("Auth", true);
    let auth_middleware = if auth_enabled {
        AuthMiddlewareFactory::new()
            .map_err(|e| {
                tracing::error!("Failed to initialize auth middleware: {}", e);
                e
            })?
    } else {
        tracing::warn!("Auth feature disabled via feature toggles; injecting default claims.");
        AuthMiddlewareFactory::disabled()
    };

    tracing::info!("🔐 [MCP Service] Authentication middleware initialized");

    // Database connection (gated by Auth toggle)
    let db_pool_opt: Option<PgPool> = if auth_enabled {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://conhub:conhub_password@postgres:5432/conhub".to_string());

        tracing::info!("📊 [MCP Service] Connecting to database...");
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
            .await?;
        tracing::info!("✅ [MCP Service] Database connection established");
        Some(pool)
    } else {
        tracing::warn!("[MCP Service] Auth disabled; skipping database connection.");
        None
    };

    // Data Service URL for GraphQL queries
    let data_service_url = env::var("DATA_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:3013".to_string());
    
    // Initialize Context Manager
    let context_manager = std::sync::Arc::new(ContextManager::new(
        db_pool_opt.clone(),
        data_service_url.clone()
    ));
    tracing::info!("📊 [MCP Service] Context Manager initialized");
    
    // Initialize MCP Server
    let mcp_server = MCPServer::new(
        db_pool_opt.clone(),
        context_manager.clone()
    );
    tracing::info!("🔌 [MCP Service] MCP Server initialized");

    tracing::info!("🚀 [MCP Service] Starting on port {}", port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .app_data(web::Data::new(db_pool_opt.clone()))
            .app_data(web::Data::new(toggles.clone()))
            .app_data(web::Data::new(mcp_server.clone()))
            .app_data(web::Data::new(context_manager.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(auth_middleware.clone())
            .configure(configure_routes)
            .route("/health", web::get().to(health_check))
            // MCP Protocol WebSocket endpoint
            .route("/mcp", web::get().to(handlers::mcp_websocket))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}

fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/mcp")
            // MCP Server routes
            .route("/resources", web::get().to(handlers::list_resources))
            .route("/resources/{uri}", web::get().to(handlers::read_resource))
            .route("/tools", web::get().to(handlers::list_tools))
            .route("/tools/execute", web::post().to(handlers::execute_tool))
            .route("/prompts", web::get().to(handlers::list_prompts))
            .route("/prompts/{name}", web::get().to(handlers::get_prompt))
            
            // Agent management routes
            .route("/agents", web::get().to(handlers::list_agents))
            .route("/agents", web::post().to(handlers::register_agent))
            .route("/agents/{id}", web::delete().to(handlers::unregister_agent))
            .route("/agents/{id}/context", web::get().to(handlers::get_agent_context))
            
            // Context synchronization routes
            .route("/sync", web::post().to(handlers::sync_context))
            .route("/broadcast", web::post().to(handlers::broadcast_to_agents))
    );
}

async fn health_check(pool_opt: web::Data<Option<PgPool>>) -> actix_web::Result<web::Json<serde_json::Value>> {
    let db_status = match pool_opt.get_ref() {
        Some(pool) => match sqlx::query("SELECT 1 as test").fetch_one(pool).await {
            Ok(_) => "connected",
            Err(e) => {
                tracing::error!("[MCP Service] Database health check failed: {}", e);
                "disconnected"
            }
        },
        None => "disabled",
    };

    Ok(web::Json(serde_json::json!({
        "status": "healthy",
        "service": "mcp-service",
        "database": db_status,
        "timestamp": chrono::Utc::now()
    })))
}
