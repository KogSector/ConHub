use actix_web::{web, HttpResponse, Result};
use serde_json::json;

use crate::models::*;
use crate::IndexerState;

pub async fn get_status(state: web::Data<IndexerState>) -> Result<HttpResponse> {
    let code_stats = state.code_indexer.get_stats().await;
    let doc_stats = state.doc_indexer.get_stats().await;
    let web_stats = state.web_indexer.get_stats().await;

    let total_active = code_stats.active_jobs + doc_stats.active_jobs + web_stats.active_jobs;
    let total_completed = code_stats.completed_jobs + doc_stats.completed_jobs + web_stats.completed_jobs;
    let total_failed = code_stats.failed_jobs + doc_stats.failed_jobs + web_stats.failed_jobs;
    let total_queue = code_stats.queue_size + doc_stats.queue_size + web_stats.queue_size;

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "status": "operational",
        "total": {
            "active_jobs": total_active,
            "completed_jobs": total_completed,
            "failed_jobs": total_failed,
            "queue_size": total_queue
        },
        "services": {
            "code_indexer": {
                "active_jobs": code_stats.active_jobs,
                "completed_jobs": code_stats.completed_jobs,
                "failed_jobs": code_stats.failed_jobs,
                "queue_size": code_stats.queue_size
            },
            "document_indexer": {
                "active_jobs": doc_stats.active_jobs,
                "completed_jobs": doc_stats.completed_jobs,
                "failed_jobs": doc_stats.failed_jobs,
                "queue_size": doc_stats.queue_size
            },
            "web_indexer": {
                "active_jobs": web_stats.active_jobs,
                "completed_jobs": web_stats.completed_jobs,
                "failed_jobs": web_stats.failed_jobs,
                "queue_size": web_stats.queue_size
            }
        }
    })))
}

pub async fn get_job_status(
    state: web::Data<IndexerState>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let job_id = path.into_inner();

    
    if let Some(job) = state.code_indexer.get_job(&job_id).await {
        return Ok(HttpResponse::Ok().json(IndexingResultResponse {
            job_id: job.id.clone(),
            status: job.status,
            documents_processed: job.documents_processed,
            chunks_created: job.chunks_created,
            embeddings_generated: job.embeddings_generated,
            processing_time_ms: job.processing_time_ms(),
            started_at: job.started_at,
            completed_at: job.completed_at,
            error_message: job.error_message,
        }));
    }

    if let Some(job) = state.doc_indexer.get_job(&job_id).await {
        return Ok(HttpResponse::Ok().json(IndexingResultResponse {
            job_id: job.id.clone(),
            status: job.status,
            documents_processed: job.documents_processed,
            chunks_created: job.chunks_created,
            embeddings_generated: job.embeddings_generated,
            processing_time_ms: job.processing_time_ms(),
            started_at: job.started_at,
            completed_at: job.completed_at,
            error_message: job.error_message,
        }));
    }

    if let Some(job) = state.web_indexer.get_job(&job_id).await {
        return Ok(HttpResponse::Ok().json(IndexingResultResponse {
            job_id: job.id.clone(),
            status: job.status,
            documents_processed: job.documents_processed,
            chunks_created: job.chunks_created,
            embeddings_generated: job.embeddings_generated,
            processing_time_ms: job.processing_time_ms(),
            started_at: job.started_at,
            completed_at: job.completed_at,
            error_message: job.error_message,
        }));
    }

    Ok(HttpResponse::NotFound().json(json!({
        "error": "Job not found",
        "job_id": job_id
    })))
}
