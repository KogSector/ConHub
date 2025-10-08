use std::error::Error;
use log::{info, error, warn};
use crate::services::vector_db;
use crate::services::feature_toggle_service::FeatureToggleService;

/// Indexes documents from the data sources into the vector database
pub async fn index_documents() -> Result<(), Box<dyn Error>> {
    info!("Starting document indexing process");
    
    // Check if heavy operations are enabled
    let feature_service = FeatureToggleService::new("feature-toggles.json");
    if !feature_service.is_heavy_enabled().await {
        warn!("Heavy operations disabled - skipping document indexing");
        return Ok(());
    }
    
    // Here we would typically:
    // 1. Fetch documents from the data source
    // 2. Process and chunk the documents
    // 3. Generate embeddings
    // 4. Store in vector database
    
    // For now, we'll just call the vector_db service
    match vector_db::index_documents().await {
        Ok(_) => {
            info!("Document indexing completed successfully");
            Ok(())
        },
        Err(e) => {
            error!("Document indexing failed: {}", e);
            Err(e)
        }
    }
}