use std::sync::Arc;
use uuid::Uuid;
use sqlx::PgPool;
use tracing::{info, warn, error};
use anyhow::{Result, Context};
use tokio::sync::RwLock;
use std::collections::HashMap;

use crate::connectors::{ConnectorManager, types::*};
use super::{EmbeddingClient, GraphRagIngestionService};
use conhub_models::chunking::SourceKind;

/// Ingestion service that orchestrates document processing pipeline
pub struct IngestionService {
    connector_manager: Arc<ConnectorManager>,
    embedding_client: Arc<EmbeddingClient>,
    db_pool: Option<PgPool>,
    active_jobs: Arc<RwLock<HashMap<Uuid, SyncJobHandle>>>,
    graph_rag_ingestion: Option<Arc<GraphRagIngestionService>>,
}

/// Handle for tracking active sync jobs
#[derive(Debug, Clone)]
pub struct SyncJobHandle {
    pub job_id: Uuid,
    pub account_id: Uuid,
    pub status: SyncJobStatus,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub documents_processed: usize,
    pub total_documents: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum SyncJobStatus {
    Running,
    Completed,
    Failed(String),
    Cancelled,
}

impl IngestionService {
    pub fn new(
        connector_manager: Arc<ConnectorManager>,
        embedding_client: Arc<EmbeddingClient>,
        db_pool: Option<PgPool>,
    ) -> Self {
        Self {
            connector_manager,
            embedding_client,
            db_pool,
            active_jobs: Arc::new(RwLock::new(HashMap::new())),
            graph_rag_ingestion: None,
        }
    }

    /// Add GraphRAG ingestion service (builder pattern)
    pub fn with_graph_rag(mut self, graph_rag: Arc<GraphRagIngestionService>) -> Self {
        self.graph_rag_ingestion = Some(graph_rag);
        self
    }

    /// Start a sync job for a connected account
    pub async fn start_sync_job(
        &self,
        user_id: Uuid,
        account_id: Uuid,
        request: SyncRequestWithFilters,
    ) -> Result<Uuid> {
        let job_id = Uuid::new_v4();
        
        info!("🚀 Starting sync job {} for account {}", job_id, account_id);

        // Create job record in database
        if let Some(ref pool) = self.db_pool {
            sqlx::query!(
                r#"
                INSERT INTO sync_jobs (id, user_id, account_id, connector_type, job_type, status, config)
                SELECT $1, $2, $3, connector_type, $4, 'pending', $5
                FROM connected_accounts WHERE id = $3
                "#,
                job_id,
                user_id,
                account_id,
                if request.force_full_sync { "full_sync" } else { "incremental_sync" },
                serde_json::to_value(&request)?
            )
            .execute(pool)
            .await
            .context("Failed to create sync job")?;
        }

        // Start the sync process in background
        let service = Arc::new(self.clone());
        let job_handle = SyncJobHandle {
            job_id,
            account_id,
            status: SyncJobStatus::Running,
            started_at: chrono::Utc::now(),
            documents_processed: 0,
            total_documents: None,
        };

        // Store job handle
        {
            let mut active_jobs = self.active_jobs.write().await;
            active_jobs.insert(job_id, job_handle);
        }

        // Spawn background task
        tokio::spawn(async move {
            if let Err(e) = service.execute_sync_job(job_id, account_id, request).await {
                error!("Sync job {} failed: {}", job_id, e);
                service.mark_job_failed(job_id, e.to_string()).await;
            }
        });

        Ok(job_id)
    }

    /// Execute a sync job
    async fn execute_sync_job(
        &self,
        job_id: Uuid,
        account_id: Uuid,
        request: SyncRequestWithFilters,
    ) -> Result<()> {
        info!("🔄 Executing sync job {} for account {}", job_id, account_id);

        // Update job status to running
        self.update_job_status(job_id, "running").await?;

        // Create sync run record
        let run_id = self.create_sync_run(job_id).await?;

        // Perform the sync
        let sync_result = self.connector_manager
            .sync(account_id, request)
            .await
            .context("Connector sync failed")?;

        let (sync_summary, documents_for_embedding) = sync_result;

        info!(
            "📊 Sync completed: {} documents discovered, {} ready for embedding",
            sync_summary.total_documents,
            documents_for_embedding.len()
        );

        // Trigger GraphRAG ingestion pipeline (chunker → embedding → graph)
        if let Some(graph_rag) = &self.graph_rag_ingestion {
            if !documents_for_embedding.is_empty() {
                // Determine source kind based on connector type
                let source_kind = documents_for_embedding
                    .first()
                    .map(|d| match d.connector_type {
                        crate::connectors::ConnectorType::GitHub
                        | crate::connectors::ConnectorType::Bitbucket
                        | crate::connectors::ConnectorType::GitLab => SourceKind::CodeRepo,
                        crate::connectors::ConnectorType::Slack => SourceKind::Chat,
                        _ => SourceKind::Document,
                    })
                    .unwrap_or(SourceKind::Document);

                info!("🧩 Starting GraphRAG ingestion for {} documents (source_kind: {:?})", 
                     documents_for_embedding.len(), source_kind);

                let docs_for_graph = documents_for_embedding.clone();
                match graph_rag.ingest_documents(account_id, source_kind, docs_for_graph).await {
                    Ok(chunk_job_id) => {
                        info!("✅ GraphRAG chunking job started: {}", chunk_job_id);
                    }
                    Err(e) => {
                        warn!("⚠️  GraphRAG ingestion failed for job {}: {}", job_id, e);
                    }
                }
            }
        }

        // Update sync run with results
        self.update_sync_run(
            run_id,
            sync_summary.total_documents,
            documents_for_embedding.len(),
            0, // failed count - will be updated as we process
        ).await?;

        // Process documents for embedding
        let mut processed_count = 0;
        let mut failed_count = 0;

        for document in documents_for_embedding {
            match self.process_document_for_embedding(document).await {
                Ok(_) => {
                    processed_count += 1;
                    self.update_job_progress(job_id, processed_count).await;
                }
                Err(e) => {
                    failed_count += 1;
                    warn!("Failed to process document for embedding: {}", e);
                }
            }
        }

        // Update final sync run status
        self.complete_sync_run(run_id, processed_count, failed_count).await?;

        // Mark job as completed
        self.update_job_status(job_id, "completed").await?;
        self.remove_active_job(job_id).await;

        info!("✅ Sync job {} completed successfully", job_id);
        Ok(())
    }

    /// Process a document for embedding
    async fn process_document_for_embedding(
        &self,
        document: DocumentForEmbedding,
    ) -> Result<()> {
        info!("📄 Processing document for embedding: {}", document.name);

        // Store document metadata in database
        if let Some(ref pool) = self.db_pool {
            // Insert or update source document
            sqlx::query!(
                r#"
                INSERT INTO source_documents (
                    id, source_id, connector_type, external_id, name, path,
                    content_type, size, metadata, created_at, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                ON CONFLICT (source_id, external_id) 
                DO UPDATE SET 
                    name = EXCLUDED.name,
                    path = EXCLUDED.path,
                    content_type = EXCLUDED.content_type,
                    size = EXCLUDED.size,
                    metadata = EXCLUDED.metadata,
                    updated_at = CURRENT_TIMESTAMP
                "#,
                document.id,
                document.source_id,
                document.connector_type.as_str(),
                document.external_id,
                document.name,
                document.path,
                &document.content_type.to_string(),
                document.content.len() as i64,
                document.metadata
            )
            .execute(pool)
            .await
            .context("Failed to store document metadata")?;

            // Store document chunks if provided
            if let Some(chunks) = &document.chunks {
                for chunk in chunks {
                    sqlx::query!(
                        r#"
                        INSERT INTO document_chunks (
                            document_id, chunk_number, content, start_offset, end_offset, metadata
                        ) VALUES ($1, $2, $3, $4, $5, $6)
                        ON CONFLICT (document_id, chunk_number)
                        DO UPDATE SET
                            content = EXCLUDED.content,
                            start_offset = EXCLUDED.start_offset,
                            end_offset = EXCLUDED.end_offset,
                            metadata = EXCLUDED.metadata,
                            updated_at = CURRENT_TIMESTAMP
                        "#,
                        document.id,
                        chunk.chunk_number as i32,
                        chunk.content,
                        chunk.start_offset as i32,
                        chunk.end_offset as i32,
                        chunk.metadata.clone().unwrap_or_else(|| serde_json::json!({}))
                    )
                    .execute(pool)
                    .await
                    .context("Failed to store document chunk")?;
                }
            }

            // Add to embedding queue
            sqlx::query!(
                r#"
                INSERT INTO embedding_queue (document_id, status)
                VALUES ($1, 'pending')
                ON CONFLICT (document_id) DO NOTHING
                "#,
                document.id
            )
            .execute(pool)
            .await
            .context("Failed to add to embedding queue")?;
        }

        // Send to embedding service if Heavy features are enabled
        // Note: We'll always try to embed for now, the client handles feature toggle internally
        let chunks = document.chunks.unwrap_or_else(|| {
            // Create default chunk if no chunks provided
            vec![DocumentChunk {
                chunk_number: 0,
                content: document.content.clone(),
                start_offset: 0,
                end_offset: document.content.len(),
                metadata: Some(serde_json::json!({
                    "document_name": document.name,
                    "document_id": document.id
                })),
            }]
        });

        // NOTE: Embeddings are now stored ONLY in vector DB via chunker → vector_rag pipeline.
        // We no longer write embeddings to Postgres document_chunks.embedding_vector.
        // This is part of the dual-truth architecture (Option A):
        // - Postgres chunks table = text truth
        // - Vector DB = semantic/embedding truth
        // - Graph = relationship truth
        for chunk in chunks {
            match self.embedding_client.embed_text(&chunk.content).await {
                Ok(_embedding) => {
                    // Embedding generated successfully - it will be stored in vector DB by the pipeline.
                    // We intentionally do NOT store embeddings in Postgres anymore.
                    info!("✅ Embedding generated for chunk {} (stored in vector DB only)", chunk.chunk_number);
                }
                Err(e) => {
                    warn!("Failed to generate embedding for chunk {}: {}", chunk.chunk_number, e);
                }
            }
        }

        // Mark document as indexed
        if let Some(ref pool) = self.db_pool {
            sqlx::query!(
                "UPDATE source_documents SET indexed_at = CURRENT_TIMESTAMP WHERE id = $1",
                document.id
            )
            .execute(pool)
            .await
            .context("Failed to mark document as indexed")?;

            // Update embedding queue status
            sqlx::query!(
                r#"
                UPDATE embedding_queue 
                SET status = 'completed', processed_at = CURRENT_TIMESTAMP 
                WHERE document_id = $1
                "#,
                document.id
            )
            .execute(pool)
            .await
            .context("Failed to update embedding queue")?;
        }

        Ok(())
    }

    /// Get status of all active sync jobs
    pub async fn get_active_jobs(&self) -> Vec<SyncJobHandle> {
        let active_jobs = self.active_jobs.read().await;
        active_jobs.values().cloned().collect()
    }

    /// Get status of a specific sync job
    pub async fn get_job_status(&self, job_id: Uuid) -> Option<SyncJobHandle> {
        let active_jobs = self.active_jobs.read().await;
        active_jobs.get(&job_id).cloned()
    }

    /// Cancel a running sync job
    pub async fn cancel_job(&self, job_id: Uuid) -> Result<()> {
        info!("🛑 Cancelling sync job {}", job_id);
        
        self.update_job_status(job_id, "cancelled").await?;
        self.remove_active_job(job_id).await;
        
        Ok(())
    }

    // Helper methods for database operations
    async fn create_sync_run(&self, job_id: Uuid) -> Result<Uuid> {
        if let Some(ref pool) = self.db_pool {
            let run_id = Uuid::new_v4();
            let run_number: i32 = sqlx::query_scalar!(
                "SELECT COALESCE(MAX(run_number), 0) + 1 FROM sync_runs WHERE job_id = $1",
                job_id
            )
            .fetch_one(pool)
            .await
            .unwrap_or(1);

            sqlx::query!(
                r#"
                INSERT INTO sync_runs (id, job_id, run_number, status)
                VALUES ($1, $2, $3, 'running')
                "#,
                run_id,
                job_id,
                run_number
            )
            .execute(pool)
            .await
            .context("Failed to create sync run")?;

            Ok(run_id)
        } else {
            Ok(Uuid::new_v4()) // Return dummy ID when no database
        }
    }

    async fn update_sync_run(
        &self,
        run_id: Uuid,
        discovered: usize,
        processed: usize,
        failed: usize,
    ) -> Result<()> {
        if let Some(ref pool) = self.db_pool {
            sqlx::query!(
                r#"
                UPDATE sync_runs 
                SET documents_discovered = $1, documents_processed = $2, documents_failed = $3
                WHERE id = $4
                "#,
                discovered as i32,
                processed as i32,
                failed as i32,
                run_id
            )
            .execute(pool)
            .await
            .context("Failed to update sync run")?;
        }
        Ok(())
    }

    async fn complete_sync_run(&self, run_id: Uuid, processed: usize, failed: usize) -> Result<()> {
        if let Some(ref pool) = self.db_pool {
            sqlx::query!(
                r#"
                UPDATE sync_runs 
                SET status = 'completed', completed_at = CURRENT_TIMESTAMP,
                    documents_processed = $1, documents_failed = $2
                WHERE id = $3
                "#,
                processed as i32,
                failed as i32,
                run_id
            )
            .execute(pool)
            .await
            .context("Failed to complete sync run")?;
        }
        Ok(())
    }

    async fn update_job_status(&self, job_id: Uuid, status: &str) -> Result<()> {
        if let Some(ref pool) = self.db_pool {
            match status {
                "running" => {
                    sqlx::query!(
                        r#"
                        UPDATE sync_jobs 
                        SET status = $1, started_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
                        WHERE id = $2
                        "#,
                        status,
                        job_id
                    )
                    .execute(pool)
                    .await
                    .context("Failed to update job status")?;
                }
                "completed" | "cancelled" | "failed" => {
                    sqlx::query!(
                        r#"
                        UPDATE sync_jobs 
                        SET status = $1, completed_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
                        WHERE id = $2
                        "#,
                        status,
                        job_id
                    )
                    .execute(pool)
                    .await
                    .context("Failed to update job status")?;
                }
                _ => {
                    sqlx::query!(
                        r#"
                        UPDATE sync_jobs 
                        SET status = $1, updated_at = CURRENT_TIMESTAMP
                        WHERE id = $2
                        "#,
                        status,
                        job_id
                    )
                    .execute(pool)
                    .await
                    .context("Failed to update job status")?;
                }
            }
        }
        Ok(())
    }

    async fn update_job_progress(&self, job_id: Uuid, processed_count: usize) {
        let mut active_jobs = self.active_jobs.write().await;
        if let Some(job) = active_jobs.get_mut(&job_id) {
            job.documents_processed = processed_count;
        }
    }

    async fn mark_job_failed(&self, job_id: Uuid, error: String) {
        if let Err(e) = self.update_job_status(job_id, "failed").await {
            error!("Failed to update job status to failed: {}", e);
        }

        if let Some(ref pool) = self.db_pool {
            if let Err(e) = sqlx::query!(
                "UPDATE sync_jobs SET error_message = $1 WHERE id = $2",
                error,
                job_id
            )
            .execute(pool)
            .await
            {
                error!("Failed to store job error message: {}", e);
            }
        }

        self.remove_active_job(job_id).await;
    }

    async fn remove_active_job(&self, job_id: Uuid) {
        let mut active_jobs = self.active_jobs.write().await;
        active_jobs.remove(&job_id);
    }
}

impl Clone for IngestionService {
    fn clone(&self) -> Self {
        Self {
            connector_manager: Arc::clone(&self.connector_manager),
            embedding_client: Arc::clone(&self.embedding_client),
            db_pool: self.db_pool.clone(),
            active_jobs: Arc::clone(&self.active_jobs),
            graph_rag_ingestion: self.graph_rag_ingestion.clone(),
        }
    }
}
