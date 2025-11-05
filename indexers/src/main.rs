use conhub_indexers::{IndexerConfig, create_indexers};
use tokio;
use conhub_config::feature_toggles::FeatureToggles;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Load feature toggles
    let toggles = FeatureToggles::from_env_path();
    let heavy_enabled = toggles.is_enabled("Heavy");

    // Load configuration
    let config = IndexerConfig::from_env();

    if !heavy_enabled {
        println!("Heavy mode disabled; skipping indexer initialization.");
        // Keep the service alive but idle to report healthy status if needed
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }

    // Create indexers
    let (code_indexer, doc_indexer, web_indexer) = create_indexers(config.clone());

    println!("Indexers service started successfully");
    println!("Configuration: {}", config);

    // Keep the service running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}