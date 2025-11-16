// MCP Service Main Entry Point
use anyhow::Result;
use mcp_service::{McpConfig, connectors::ConnectorManager, protocol::McpServer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use actix_web::{web, App, HttpResponse, HttpServer};

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "mcp-service"
    }))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting ConHub MCP Service");

    // Load configuration
    dotenv::dotenv().ok();
    let config = McpConfig::from_env()?;

    // Initialize database and Redis
    let db_config = conhub_database::DatabaseConfig::from_env();
    let database = conhub_database::Database::new(&db_config).await?;

    tracing::info!("Database and Redis connected");

    // Initialize connector manager
    let connector_manager = ConnectorManager::new(database, &config).await?;

    tracing::info!(
        "Initialized {} connectors",
        connector_manager.connector_count()
    );

    // Start HTTP server for health checks in background
    let port = std::env::var("MCP_PORT").unwrap_or_else(|_| "3004".to_string());
    let port_num: u16 = port.parse().unwrap_or(3004);
    
    tokio::spawn(async move {
        tracing::info!("🚀 [MCP Service] Starting HTTP health server on port {}", port_num);
        HttpServer::new(|| {
            App::new()
                .route("/health", web::get().to(health))
        })
        .bind(("0.0.0.0", port_num))
        .expect("Failed to bind MCP HTTP server")
        .run()
        .await
        .expect("MCP HTTP server failed");
    });

    // Start MCP server on stdio (main protocol)
    let server = McpServer::new(connector_manager, config);
    server.run().await?;

    Ok(())
}
