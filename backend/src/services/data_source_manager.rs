use crate::services::data_source_proxy::{
    connect_data_source, sync_data_source, DataSourceRequest, DataSourceResponse
};
use crate::services::indexing_service;
use reqwest::Client;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use log::{info, error};

/// Manages data source connections and triggers indexing automatically
pub struct DataSourceManager {
    client: Client,
    langchain_url: String,
}

impl DataSourceManager {
    pub fn new() -> Self {
        let client = Client::new();
        let langchain_url = std::env::var("LANGCHAIN_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:3002".to_string());
        
        DataSourceManager {
            client,
            langchain_url,
        }
    }

    /// Connect to a data source and automatically trigger indexing
    pub async fn connect_and_index(&self, request: &DataSourceRequest) -> Result<DataSourceResponse, Box<dyn Error>> {
        info!("Connecting to data source: {}", request.name);
        
        // Connect to the data source
        let response = connect_data_source(&self.client, &self.langchain_url, request).await?;
        
        if response.success {
            info!("Successfully connected to data source: {}", request.name);
            
            // Trigger indexing in a separate task to not block the response
            let source_id = response.id.clone();
            let client = self.client.clone();
            let langchain_url = self.langchain_url.clone();
            
            tokio::spawn(async move {
                info!("Starting indexing for data source: {}", source_id);
                match sync_data_source(&client, &langchain_url, &source_id).await {
                    Ok(_) => {
                        info!("Indexing completed for data source: {}", source_id);
                        // Trigger vector DB indexing after document indexing is complete
                        match indexing_service::index_documents().await {
                            Ok(_) => info!("Vector indexing completed for data source: {}", source_id),
                            Err(e) => error!("Vector indexing failed: {}", e),
                        }
                    },
                    Err(e) => error!("Indexing failed for data source {}: {}", source_id, e),
                }
            });
        }
        
        Ok(response)
    }
}

// Singleton instance for the data source manager
lazy_static::lazy_static! {
    static ref DATA_SOURCE_MANAGER: Arc<Mutex<DataSourceManager>> = Arc::new(Mutex::new(
        DataSourceManager::new()
    ));
}

/// Get the global data source manager instance
pub async fn get_manager() -> Arc<Mutex<DataSourceManager>> {
    DATA_SOURCE_MANAGER.clone()
}