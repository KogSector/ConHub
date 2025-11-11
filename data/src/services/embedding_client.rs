use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{info, error};

use crate::connectors::DocumentForEmbedding;

#[derive(Debug, Serialize)]
struct BatchEmbedRequest {
    documents: Vec<DocumentForEmbedding>,
    normalize: bool,
    store_in_vector_db: bool,
}

#[derive(Debug, Deserialize)]
struct BatchEmbedResponse {
    total_documents: usize,
    successful: usize,
    failed: usize,
    results: Vec<DocumentEmbedResult>,
    duration_ms: u64,
}

#[derive(Debug, Deserialize)]
struct DocumentEmbedResult {
    id: Uuid,
    name: String,
    status: String,
    chunks_processed: usize,
    error: Option<String>,
}

/// Client for communicating with the embedding service
pub struct EmbeddingClient {
    client: Client,
    base_url: String,
    enabled: bool,
}

impl EmbeddingClient {
    pub fn new(base_url: String, enabled: bool) -> Self {
        Self {
            client: Client::new(),
            base_url,
            enabled,
        }
    }
    
    /// Send documents to the embedding service for processing
    pub async fn embed_documents(
        &self,
        documents: Vec<DocumentForEmbedding>,
    ) -> Result<BatchEmbedResponse, Box<dyn std::error::Error + Send + Sync>> {
        if !self.enabled {
            info!("📊 Embedding service disabled, skipping {} documents", documents.len());
            return Ok(BatchEmbedResponse {
                total_documents: documents.len(),
                successful: 0,
                failed: 0,
                results: vec![],
                duration_ms: 0,
            });
        }
        
        if documents.is_empty() {
            return Ok(BatchEmbedResponse {
                total_documents: 0,
                successful: 0,
                failed: 0,
                results: vec![],
                duration_ms: 0,
            });
        }
        
        info!("📤 Sending {} documents to embedding service", documents.len());
        
        let request = BatchEmbedRequest {
            documents,
            normalize: true,
            store_in_vector_db: true,
        };
        
        let url = format!("{}/batch/embed", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Embedding service error {}: {}", status, error_text);
            return Err(format!("Embedding service error: {} - {}", status, error_text).into());
        }
        
        let result: BatchEmbedResponse = response.json().await?;
        
        info!(
            "✅ Embedding complete: {} successful, {} failed",
            result.successful,
            result.failed
        );
        
        Ok(result)
    }
    
    /// Check if the embedding service is healthy
    pub async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        if !self.enabled {
            return Ok(false);
        }
        
        let url = format!("{}/health", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        Ok(response.status().is_success())
    }
}

impl Clone for EmbeddingClient {
    fn clone(&self) -> Self {
        Self {
            client: Client::new(),
            base_url: self.base_url.clone(),
            enabled: self.enabled,
        }
    }
}
