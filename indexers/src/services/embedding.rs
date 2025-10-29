use crate::config::IndexerConfig;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

/// Retry configuration constants
const MAX_RETRIES: u32 = 3;
const INITIAL_RETRY_DELAY: Duration = Duration::from_millis(500);
const MAX_RETRY_DELAY: Duration = Duration::from_secs(10);

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
    pub fn new(config: IndexerConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        let embedding_service_url = config.embedding_service_url.clone();

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

    /// Generate embeddings for multiple texts with retry logic
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

        for attempt in 0..=MAX_RETRIES {
            match self.try_generate_embeddings(&url, &request).await {
                Ok(embeddings) => {
                    if attempt > 0 {
                        log::info!("Embedding request succeeded after {} retries", attempt);
                    }
                    return Ok(embeddings);
                }
                Err(e) => {
                    if attempt == MAX_RETRIES {
                        log::error!("Embedding request failed after {} attempts: {}", MAX_RETRIES + 1, e);
                        return Err(e);
                    }

                    let delay = std::cmp::min(
                        INITIAL_RETRY_DELAY * 2_u32.pow(attempt),
                        MAX_RETRY_DELAY,
                    );

                    log::warn!(
                        "Embedding request failed (attempt {}/{}), retrying in {:?}: {}",
                        attempt + 1,
                        MAX_RETRIES + 1,
                        delay,
                        e
                    );

                    sleep(delay).await;
                }
            }
        }

        unreachable!("Loop should have returned or errored")
    }

    /// Single attempt to generate embeddings
    async fn try_generate_embeddings(
        &self,
        url: &str,
        request: &EmbedRequest,
    ) -> Result<Vec<Vec<f32>>> {
        let response = self
            .http_client
            .post(url)
            .json(request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to connect to embedding service: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Embedding service returned error {}: {}", status, error_text));
        }

        let embed_response = response
            .json::<EmbedResponse>()
            .await
            .map_err(|e| anyhow!("Failed to parse embed response: {}", e))?;

        log::debug!(
            "Received {} embeddings (dim: {})",
            embed_response.count,
            embed_response.dimension
        );

        Ok(embed_response.embeddings)
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
