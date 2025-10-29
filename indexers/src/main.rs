use conhub_indexers::{IndexerConfig, create_indexers};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Load configuration
    let config = IndexerConfig::from_env();

    // Create indexers
    let (code_indexer, doc_indexer, web_indexer) = create_indexers(config.clone());

    println!("Indexers service started successfully");
    println!("Configuration: {}", config);

    // Keep the service running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}