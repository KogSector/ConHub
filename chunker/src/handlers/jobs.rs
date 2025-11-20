use actix_web::{web, HttpResponse};
use std::sync::Arc;
use tracing::{info, error};
use uuid::Uuid;

use conhub_models::chunking::{
    StartChunkJobRequest, StartChunkJobResponse,
    ChunkJobStatusResponse, ChunkJobStatus,
};

use crate::models::{AppState, JobHandle};
use crate::services::chunker::ChunkerService;

/// Start a new chunking job
pub async fn start_chunk_job(
    payload: web::Json<StartChunkJobRequest>,
    state: web::Data<Arc<AppState>>,
) -> HttpResponse {
    let job_id = Uuid::new_v4();
    let items_count = payload.items.len();
    
    info!(
        "📦 Starting chunking job {} for source {} with {} items",
        job_id, payload.source_id, items_count
    );

    // Create job handle
    let job_handle = JobHandle::new(job_id, payload.source_id, items_count);
    
    // Store job in state
    {
        let mut jobs = state.jobs.write().await;
        jobs.insert(job_id, job_handle);
    }

    // Spawn background task to process the job
    let state_clone = state.get_ref().clone();
    let request = payload.into_inner();
    
    tokio::spawn(async move {
        let chunker = ChunkerService::new(
            state_clone.embedding_client.clone(),
            state_clone.graph_client.clone(),
        );
        
        if let Err(e) = chunker.process_job(
            job_id,
            &state_clone,
            request,
        ).await {
            error!("❌ Job {} failed: {}", job_id, e);
            
            // Mark job as failed
            let mut jobs = state_clone.jobs.write().await;
            if let Some(job) = jobs.get_mut(&job_id) {
                job.status = ChunkJobStatus::Failed;
                job.error_message = Some(e.to_string());
            }
        }
    });

    HttpResponse::Ok().json(StartChunkJobResponse {
        job_id,
        accepted: items_count,
    })
}

/// Get status of a chunking job
pub async fn get_chunk_job_status(
    path: web::Path<Uuid>,
    state: web::Data<Arc<AppState>>,
) -> HttpResponse {
    let job_id = path.into_inner();
    
    let jobs = state.jobs.read().await;
    
    if let Some(job) = jobs.get(&job_id) {
        HttpResponse::Ok().json(ChunkJobStatusResponse {
            job_id: job.job_id,
            status: job.status.clone(),
            items_total: job.items_total,
            items_processed: job.items_processed,
            chunks_emitted: job.chunks_emitted,
            error_message: job.error_message.clone(),
        })
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "error": "Job not found",
            "job_id": job_id
        }))
    }
}
