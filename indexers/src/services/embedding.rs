use crate::config::IndexerConfig;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Request to embedding service
#[derive(Debug, Serialize)]
struct EmbedRequest {
    text: Vec<String>,
    normalize: bool,
}

/// Response from embedding service
#[derive(Debug, Deserialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f32>>,
    dimension: usize,
    model: String,
    count: usize,
}

pub struct EmbeddingService {
    config: IndexerConfig,
    http_client: reqwest::Client,
    embedding_service_url: String,
}

impl EmbeddingService {
    pub fn new(config: IndexerConfig, embedding_service_url: String) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        Self {
            config,
            http_client,
            embedding_service_url,
        }
    }

    /// Generate embedding for a single text
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.generate_batch_embeddings(&[text.to_string()]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("No embedding returned"))
    }

    /// Generate embeddings for multiple texts
    pub async fn generate_batch_embeddings(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let url = format!("{}/embed", self.embedding_service_url);

        let request = EmbedRequest {
            text: texts.to_vec(),
            normalize: true,
        };

        log::debug!("Requesting embeddings from: {}", url);

        match self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<EmbedResponse>().await {
                        Ok(embed_response) => {
                            log::debug!(
                                "Received {} embeddings (dim: {})",
                                embed_response.count,
                                embed_response.dimension
                            );
                            Ok(embed_response.embeddings)
                        }
                        Err(e) => {
                            log::error!("Failed to parse embed response: {}", e);
                            Err(anyhow!("Failed to parse embed response: {}", e))
                        }
                    }
                } else {
                    let status = response.status();
                    let error_text = response.text().await.unwrap_or_default();
                    log::error!("Embedding service error: {} - {}", status, error_text);
                    Err(anyhow!("Embedding service returned error: {}", status))
                }
            }
            Err(e) => {
                log::error!("Failed to connect to embedding service: {}", e);
                Err(anyhow!("Failed to connect to embedding service: {}", e))
            }
        }
    }

    /// Store embedding in Qdrant (handled by QdrantService, kept for compatibility)
    pub async fn store_embedding(
        &self,
        id: &str,
        embedding: &[f32],
        metadata: std::collections::HashMap<String, String>,
    ) -> Result<()> {
        if self.config.qdrant_url.is_some() {
            log::debug!("Storing embedding {} (to be handled by QdrantService)", id);
        } else {
            log::debug!("Qdrant not configured, skipping embedding storage for {}", id);
        }
        Ok(())
    }

    /// Search similar embeddings (handled by QdrantService, kept for compatibility)
    pub async fn search_similar(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<(String, f32)>> {
        if self.config.qdrant_url.is_some() {
            log::debug!("Searching similar embeddings (to be handled by QdrantService)");
        }
        Ok(Vec::new())
    }

    /// Check if embedding service is available
    pub async fn is_available(&self) -> bool {
        let url = format!("{}/health", self.embedding_service_url);
        match self.http_client.get(&url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running embedding service
    async fn test_generate_embedding() {
        let config = crate::config::IndexerConfig::from_env();
        let service = EmbeddingService::new(
            config,
            "http://localhost:8082".to_string()
        );

        let text = "This is a test text";
        let embedding = service.generate_embedding(text).await.unwrap();

        assert_eq!(embedding.len(), 768);

        // Check normalization
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01);
    }

    #[tokio::test]
    #[ignore] // Requires running embedding service
    async fn test_batch_embeddings() {
        let config = crate::config::IndexerConfig::from_env();
        let service = EmbeddingService::new(
            config,
            "http://localhost:8082".to_string()
        );

        let texts = vec![
            "First text".to_string(),
            "Second text".to_string(),
            "Third text".to_string(),
        ];

        let embeddings = service.generate_batch_embeddings(&texts).await.unwrap();

        assert_eq!(embeddings.len(), 3);
        assert_eq!(embeddings[0].len(), 768);
    }
}
