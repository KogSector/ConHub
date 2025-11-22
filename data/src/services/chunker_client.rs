use reqwest::Client;
use tracing::{info, error};
use anyhow::Result;
use uuid::Uuid;

use conhub_models::chunking::{
    StartChunkJobRequest, StartChunkJobResponse,
    ChunkJobStatusResponse, SourceItem,
};

#[derive(Clone)]
pub struct ChunkerClient {
    base_url: String,
    client: Client,
}

impl ChunkerClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: Client::new(),
        }
    }

    /// Start a chunking job with a batch of source items
    pub async fn start_job(
        &self,
        request: StartChunkJobRequest,
    ) -> Result<StartChunkJobResponse> {
        info!(
            "📤 Sending {} items to chunker service",
            request.items.len()
        );

        let url = format!("{}/chunk/jobs", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("❌ Chunker service error {}: {}", status, error_text);
            anyhow::bail!("Chunker service error: {} - {}", status, error_text);
        }

        let result: StartChunkJobResponse = response.json().await?;
        
        info!("✅ Chunking job started: {}", result.job_id);

        Ok(result)
    }

    /// Get status of a chunking job
    pub async fn get_status(
        &self,
        job_id: Uuid,
    ) -> Result<ChunkJobStatusResponse> {
        let url = format!("{}/chunk/jobs/{}", self.base_url, job_id);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Failed to get job status: {} - {}", status, error_text);
        }

        let result: ChunkJobStatusResponse = response.json().await?;
        Ok(result)
    }

    /// Check if chunker service is healthy
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}
