use reqwest::Client;
use tracing::{info, error};
use anyhow::Result;
use uuid::Uuid;

use conhub_models::chunking::{
    Chunk, ChunkRef, SourceKind, ObserveChunksRequest, ObserveChunksResponse,
};

/// Client for the graph_rag observe endpoint (IDs-only, Option A architecture).
/// This sends chunk metadata (no content) to graph_rag for entity/relationship extraction.
/// Graph_rag fetches the actual chunk text from Postgres.
#[derive(Clone)]
pub struct GraphObserveClient {
    base_url: String,
    client: Client,
}

impl GraphObserveClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: Client::new(),
        }
    }

    /// Observe chunks for graph extraction (IDs-only).
    /// Graph_rag will fetch chunk text from Postgres by chunk_id.
    pub async fn observe_chunks(
        &self,
        tenant_id: Uuid,
        source_id: Uuid,
        source_kind: SourceKind,
        chunks: Vec<Chunk>,
    ) -> Result<ObserveChunksResponse> {
        if chunks.is_empty() {
            return Ok(ObserveChunksResponse {
                total_chunks: 0,
                chunks_processed: 0,
                entities_created: 0,
                relationships_created: 0,
                evidence_created: 0,
            });
        }

        info!("📤 Sending {} chunk refs to graph_rag for observation", chunks.len());

        // Convert full chunks to ChunkRefs (IDs-only, no content)
        let chunk_refs: Vec<ChunkRef> = chunks.into_iter().map(ChunkRef::from).collect();

        let request = ObserveChunksRequest {
            tenant_id,
            source_id,
            source_kind,
            chunks: chunk_refs,
        };

        let url = format!("{}/graph/observe", self.base_url);

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("❌ Graph observe error {}: {}", status, error_text);
            anyhow::bail!("Graph observe error: {} - {}", status, error_text);
        }

        let result: ObserveChunksResponse = response.json().await?;

        info!(
            "✅ Graph observation complete: {} entities, {} relationships, {} evidence",
            result.entities_created,
            result.relationships_created,
            result.evidence_created
        );

        Ok(result)
    }

    /// Check if graph_rag service is healthy
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}
