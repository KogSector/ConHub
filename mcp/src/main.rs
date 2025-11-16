// MCP Service Main Entry Point
use anyhow::Result;
use mcp_service::{McpConfig, connectors::ConnectorManager, protocol::McpServer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    // Start MCP server
    let server = McpServer::new(connector_manager, config);
    server.run().await?;

    Ok(())
}
