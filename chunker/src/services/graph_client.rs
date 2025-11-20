use reqwest::Client;
use tracing::{info, error};
use anyhow::Result;
use uuid::Uuid;

use conhub_models::chunking::{
    Chunk, SourceKind, IngestChunksRequest, IngestChunksResponse,
};

#[derive(Clone)]
pub struct GraphClient {
    base_url: String,
    client: Client,
}

impl GraphClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: Client::new(),
        }
    }

    /// Send chunks to graph service for entity/relationship extraction
    pub async fn ingest_chunks(
        &self,
        source_id: Uuid,
        source_kind: SourceKind,
        chunks: Vec<Chunk>,
    ) -> Result<IngestChunksResponse> {
        if chunks.is_empty() {
            return Ok(IngestChunksResponse {
                total_chunks: 0,
                chunks_processed: 0,
                entities_created: 0,
                relationships_created: 0,
            });
        }

        info!("📤 Sending {} chunks to graph service for ingestion", chunks.len());

        let request = IngestChunksRequest {
            source_id,
            source_kind,
            chunks,
        };

        let url = format!("{}/graph/chunks", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("❌ Graph service error {}: {}", status, error_text);
            anyhow::bail!("Graph service error: {} - {}", status, error_text);
        }

        let result: IngestChunksResponse = response.json().await?;
        
        info!(
            "✅ Graph ingestion complete: {}/{} chunks processed, {} entities, {} relationships",
            result.chunks_processed,
            result.total_chunks,
            result.entities_created,
            result.relationships_created
        );

        Ok(result)
    }
}
