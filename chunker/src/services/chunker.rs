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
use crate::services::strategies::{CodeChunker, TextChunker, ChatChunker, AstCodeChunker, MarkdownChunker, TicketingChunker, WebChunker};

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

            // Try cache first
            let cache_key_strategy = match source_item.source_kind {
                SourceKind::CodeRepo => "ast_code",
                SourceKind::Document => "markdown",
                SourceKind::Chat => "chat",
                SourceKind::Ticketing => "ticketing",
                SourceKind::Web => "web",
                SourceKind::Wiki => "markdown",
                _ => "text",
            };

            let chunks = {
                let mut cache = state.cache.write().await;
                if let Ok(Some(cached_chunks)) = cache.get_chunks(&source_item.content, cache_key_strategy).await {
                    info!("🎯 Using cached chunks for item {}", source_item.id);
                    cached_chunks
                } else {
                    // Chunk the item based on its type with enhanced strategies
                    let chunks = match source_item.source_kind {
                        SourceKind::CodeRepo => {
                            // Try AST-based chunking first, fallback to regex if unsupported language
                            AstCodeChunker::chunk(source_item)
                                .or_else(|_| CodeChunker::chunk(source_item))?
                        }
                        SourceKind::Document => {
                            // Check if markdown
                            if source_item.content_type.contains("markdown") || 
                               source_item.metadata.get("path")
                                   .and_then(|v| v.as_str())
                                   .map(|p| p.ends_with(".md"))
                                   .unwrap_or(false) {
                                MarkdownChunker::chunk(source_item)?
                            } else {
                                TextChunker::chunk(source_item)?
                            }
                        }
                        SourceKind::Chat => {
                            ChatChunker::chunk(source_item)?
                        }
                        SourceKind::Ticketing => {
                            // Issues and PRs use the ticketing chunker
                            TicketingChunker::chunk(source_item)?
                        }
                        SourceKind::Web => {
                            // Web/HTML content from URL scraping
                            WebChunker::chunk(source_item)?
                        }
                        SourceKind::Wiki => {
                            // Wiki pages are typically markdown-like
                            MarkdownChunker::chunk(source_item)?
                        }
                        _ => {
                            // Default to text chunker for other types
                            TextChunker::chunk(source_item)?
                        }
                    };

                    // Cache the chunks
                    if let Err(e) = cache.set_chunks(&source_item.content, cache_key_strategy, &chunks).await {
                        warn!("⚠️  Failed to cache chunks: {}", e);
                    }

                    chunks
                }
            };

            if chunks.is_empty() {
                warn!("⚠️  No chunks generated for item {}", source_item.id);
                continue;
            }

            info!("✂️  Generated {} chunks", chunks.len());
            total_chunks += chunks.len();

            // Evaluate cost policy to determine ingestion targets
            let targets = {
                let cost_policies = state.cost_policies.read().await;
                // Estimate token count from content length (roughly 4 chars per token)
                let token_count = chunks.first().map(|c| c.content.len() / 4).unwrap_or(100);
                cost_policies.evaluate(
                    &source_item.source_kind,
                    Some(&source_item.content_type),
                    source_item.metadata.get("language").and_then(|v| v.as_str()),
                    token_count,
                )
            };

            info!(
                "💰 Cost policy: vector={}, graph={} for {:?}",
                targets.enable_vector, targets.enable_graph, source_item.source_kind
            );

            // Process chunks in batches
            for batch in chunks.chunks(batch_size) {
                let batch_vec = batch.to_vec();

                // Conditionally send to embedding service based on cost policy
                let embedding_future = if targets.enable_vector {
                    Some(self.embedding_client.embed_chunks(batch_vec.clone()))
                } else {
                    None
                };
                
                // Conditionally send to graph service based on cost policy
                let graph_future = if targets.enable_graph {
                    Some(self.graph_client.ingest_chunks(
                        request.source_id,
                        request.source_kind.clone(),
                        batch_vec,
                    ))
                } else {
                    None
                };

                // Process enabled targets in parallel
                match (embedding_future, graph_future) {
                    (Some(embed_fut), Some(graph_fut)) => {
                        let (embed_result, graph_result) = tokio::join!(embed_fut, graph_fut);
                        if let Err(e) = embed_result {
                            warn!("⚠️  Embedding failed for batch: {}", e);
                        }
                        if let Err(e) = graph_result {
                            warn!("⚠️  Graph ingestion failed for batch: {}", e);
                        }
                    }
                    (Some(embed_fut), None) => {
                        if let Err(e) = embed_fut.await {
                            warn!("⚠️  Embedding failed for batch: {}", e);
                        }
                    }
                    (None, Some(graph_fut)) => {
                        if let Err(e) = graph_fut.await {
                            warn!("⚠️  Graph ingestion failed for batch: {}", e);
                        }
                    }
                    (None, None) => {
                        info!("⏭️  Skipping batch (cost policy: none)");
                    }
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
