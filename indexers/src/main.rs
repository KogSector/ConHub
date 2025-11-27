//! ConHub Unified Indexer
//! 
//! This is the main entry point for the indexer service that runs
//! background jobs for indexing robot memory and other data sources.

use conhub_indexers::{RobotMemoryIndexer, RobotMemoryIndexerConfig};
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("🚀 ConHub Unified Indexer starting...");
    info!("📦 Version: {}", conhub_indexers::version());
    
    // Check health
    if conhub_indexers::health_check() {
        info!("✅ Health check passed");
    }
    
    // Initialize robot memory indexer
    let robot_indexer = RobotMemoryIndexer::from_env();
    
    // Start the indexer
    match robot_indexer.start().await {
        Ok(_) => {
            info!("✅ Robot memory indexer started successfully");
        }
        Err(e) => {
            error!("❌ Failed to start robot memory indexer: {}", e);
            return Err(e.into());
        }
    }
    
    // Keep running
    info!("📡 Indexer running. Press Ctrl+C to stop.");
    
    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    
    info!("🛑 Shutdown signal received");
    robot_indexer.stop().await;
    
    info!("👋 ConHub Unified Indexer stopped");
    
    Ok(())
}
