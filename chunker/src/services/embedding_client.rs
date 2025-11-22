use reqwest::Client;
use tracing::{info, error};
use anyhow::Result;

use conhub_models::chunking::{
    Chunk, EmbedChunk, BatchEmbedChunksRequest, BatchEmbedChunksResponse,
};

#[derive(Clone)]
pub struct EmbeddingClient {
    base_url: String,
    client: Client,
}

impl EmbeddingClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: Client::new(),
        }
    }

    /// Send chunks to embedding service for vectorization
    pub async fn embed_chunks(&self, chunks: Vec<Chunk>) -> Result<BatchEmbedChunksResponse> {
        if chunks.is_empty() {
            return Ok(BatchEmbedChunksResponse {
                total_chunks: 0,
                successful: 0,
                failed: 0,
                duration_ms: Some(0),
            });
        }

        info!("📤 Sending {} chunks to embedding service", chunks.len());

        // Convert Chunk to EmbedChunk
        let embed_chunks: Vec<EmbedChunk> = chunks
            .into_iter()
            .map(|c| EmbedChunk {
                chunk_id: c.chunk_id,
                content: c.content,
                metadata: c.metadata,
            })
            .collect();

        let request = BatchEmbedChunksRequest {
            chunks: embed_chunks,
            normalize: true,
            store_in_vector_db: true,
        };

        let url = format!("{}/batch/embed/chunks", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("❌ Embedding service error {}: {}", status, error_text);
            anyhow::bail!("Embedding service error: {} - {}", status, error_text);
        }

        let result: BatchEmbedChunksResponse = response.json().await?;
        
        info!(
            "✅ Embedding complete: {}/{} successful",
            result.successful,
            result.total_chunks
        );

        Ok(result)
    }
}
