use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, warn, error};
use anyhow::{Result, Context};

use conhub_models::chunking::{SourceItem, SourceKind, StartChunkJobRequest};

use crate::connectors::types::*;
use crate::services::chunker_client::ChunkerClient;

/// Graph RAG ingestion service that uses the new chunker → embedding → graph pipeline
pub struct GraphRagIngestionService {
    chunker_client: Arc<ChunkerClient>,
}

impl GraphRagIngestionService {
    pub fn new(chunker_url: String) -> Self {
        Self {
            chunker_client: Arc::new(ChunkerClient::new(chunker_url)),
        }
    }

    /// Start a Graph RAG ingestion job
    /// Converts connector documents to SourceItems and sends to chunker
    pub async fn ingest_documents(
        &self,
        source_id: Uuid,
        source_kind: SourceKind,
        documents: Vec<DocumentForEmbedding>,
    ) -> Result<Uuid> {
        info!(
            "🚀 [Graph RAG] Starting ingestion for {} documents from source {}",
            documents.len(),
            source_id
        );

        // Convert DocumentForEmbedding to SourceItem
        let items: Vec<SourceItem> = documents
            .into_iter()
            .map(|doc| self.document_to_source_item(doc, source_id, source_kind.clone()))
            .collect();

        if items.is_empty() {
            warn!("No items to ingest");
            return Ok(Uuid::new_v4()); // Return dummy ID
        }

        // Send to chunker service
        let request = StartChunkJobRequest {
            source_id,
            source_kind,
            items,
        };

        let response = self.chunker_client
            .start_job(request)
            .await
            .context("Failed to start chunking job")?;

        info!(
            "✅ Chunking job {} started with {} items accepted",
            response.job_id,
            response.accepted
        );

        Ok(response.job_id)
    }

    /// Convert legacy DocumentForEmbedding to new SourceItem format
    fn document_to_source_item(
        &self,
        doc: DocumentForEmbedding,
        source_id: Uuid,
        source_kind: SourceKind,
    ) -> SourceItem {
        // Build metadata from document
        let mut metadata = doc.metadata.clone();
        
        // Add additional fields
        metadata["connector_type"] = serde_json::json!(doc.connector_type);
        metadata["external_id"] = serde_json::json!(doc.external_id);
        metadata["name"] = serde_json::json!(doc.name);
        
        if let Some(path) = &doc.path {
            metadata["path"] = serde_json::json!(path);
        }
        
        if let Some(block_type) = &doc.block_type {
            metadata["block_type"] = serde_json::json!(block_type);
        }
        
        if let Some(language) = &doc.language {
            metadata["language"] = serde_json::json!(language);
        }

        SourceItem {
            id: doc.id,
            source_id,
            source_kind,
            content_type: doc.content_type,
            content: doc.content,
            metadata,
            created_at: Some(chrono::Utc::now()),
        }
    }

    /// Check chunker service health
    pub async fn health_check(&self) -> Result<bool> {
        self.chunker_client.health_check().await
    }

    /// Get status of a chunking job
    pub async fn get_job_status(&self, job_id: Uuid) -> Result<conhub_models::chunking::ChunkJobStatusResponse> {
        self.chunker_client.get_status(job_id).await
    }
}
