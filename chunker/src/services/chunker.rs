use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, warn, error};
use anyhow::Result;

use conhub_models::chunking::{
    StartChunkJobRequest, Chunk, ChunkJobStatus, SourceKind,
};

use crate::models::AppState;
use crate::services::embedding_client::EmbeddingClient;
use crate::services::graph_client::GraphClient;
use crate::services::strategies::{CodeChunker, TextChunker, ChatChunker};

pub struct ChunkerService {
    embedding_client: EmbeddingClient,
    graph_client: GraphClient,
}

impl ChunkerService {
    pub fn new(
        embedding_client: EmbeddingClient,
        graph_client: GraphClient,
    ) -> Self {
        Self {
            embedding_client,
            graph_client,
        }
    }

    /// Process a chunking job
    pub async fn process_job(
        &self,
        job_id: Uuid,
        state: &Arc<AppState>,
        request: StartChunkJobRequest,
    ) -> Result<()> {
        info!("🔄 Processing chunking job {}", job_id);

        // Mark job as running
        {
            let mut jobs = state.jobs.write().await;
            if let Some(job) = jobs.get_mut(&job_id) {
                job.status = ChunkJobStatus::Running;
            }
        }

        let mut total_chunks = 0;
        let batch_size = 128; // Process chunks in batches

        for (idx, source_item) in request.items.iter().enumerate() {
            info!(
                "📄 Processing item {}/{}: {} (type: {:?})",
                idx + 1,
                request.items.len(),
                source_item.metadata.get("path").and_then(|v| v.as_str()).unwrap_or("unknown"),
                source_item.source_kind
            );

            // Chunk the item based on its type
            let chunks = match source_item.source_kind {
                SourceKind::CodeRepo => {
                    CodeChunker::chunk(source_item)?
                }
                SourceKind::Document => {
                    TextChunker::chunk(source_item)?
                }
                SourceKind::Chat => {
                    ChatChunker::chunk(source_item)?
                }
                _ => {
                    // Default to text chunker for other types
                    TextChunker::chunk(source_item)?
                }
            };

            if chunks.is_empty() {
                warn!("⚠️  No chunks generated for item {}", source_item.id);
                continue;
            }

            info!("✂️  Generated {} chunks", chunks.len());
            total_chunks += chunks.len();

            // Process chunks in batches
            for batch in chunks.chunks(batch_size) {
                let batch_vec = batch.to_vec();

                // Send to embedding service (fire and forget or await both)
                let embedding_future = self.embedding_client.embed_chunks(batch_vec.clone());
                
                // Send to graph service
                let graph_future = self.graph_client.ingest_chunks(
                    request.source_id,
                    request.source_kind.clone(),
                    batch_vec,
                );

                // Process both in parallel
                let (embed_result, graph_result) = tokio::join!(embedding_future, graph_future);

                if let Err(e) = embed_result {
                    warn!("⚠️  Embedding failed for batch: {}", e);
                }

                if let Err(e) = graph_result {
                    warn!("⚠️  Graph ingestion failed for batch: {}", e);
                }
            }

            // Update progress
            {
                let mut jobs = state.jobs.write().await;
                if let Some(job) = jobs.get_mut(&job_id) {
                    job.items_processed = idx + 1;
                    job.chunks_emitted = total_chunks;
                }
            }
        }

        // Mark job as completed
        {
            let mut jobs = state.jobs.write().await;
            if let Some(job) = jobs.get_mut(&job_id) {
                job.status = ChunkJobStatus::Completed;
                job.chunks_emitted = total_chunks;
            }
        }

        info!("✅ Job {} completed: {} chunks from {} items", job_id, total_chunks, request.items.len());
        Ok(())
    }
}
