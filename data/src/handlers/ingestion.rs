use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{info, error};

use crate::services::IngestionService;
use crate::connectors::SyncRequestWithFilters;
use conhub_models::auth::Claims;

#[derive(Debug, Serialize, Deserialize)]
pub struct StartSyncRequest {
    pub account_id: String,
    pub force_full_sync: Option<bool>,
    pub filters: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncJobStatusResponse {
    pub job_id: String,
    pub account_id: String,
    pub status: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub documents_processed: usize,
    pub total_documents: Option<usize>,
}

/// Start a sync job for a connected account
pub async fn start_sync_job(
    ingestion_service: web::Data<IngestionService>,
    claims: web::ReqData<Claims>,
    body: web::Json<StartSyncRequest>,
) -> Result<HttpResponse> {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": "Invalid user ID",
            })));
        }
    };
    
    let account_id = match Uuid::parse_str(&body.account_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": "Invalid account ID",
            })));
        }
    };
    
    info!("🚀 User {} starting sync job for account {}", user_id, account_id);
    
    let sync_request = SyncRequestWithFilters {
        force_full_sync: body.force_full_sync.unwrap_or(false),
        filters: body.filters.as_ref().and_then(|f| serde_json::from_value(f.clone()).ok()),
    };
    
    match ingestion_service.start_sync_job(user_id, account_id, sync_request).await {
        Ok(job_id) => {
            info!("✅ Sync job started: {}", job_id);
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "job_id": job_id,
                "message": "Sync job started successfully",
            })))
        }
        Err(e) => {
            error!("Failed to start sync job: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": e.to_string(),
            })))
        }
    }
}

/// Get status of a specific sync job
pub async fn get_sync_job_status(
    ingestion_service: web::Data<IngestionService>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let job_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": "Invalid job ID",
            })));
        }
    };
    
    match ingestion_service.get_job_status(job_id).await {
        Some(job_handle) => {
            let status_str = match job_handle.status {
                crate::services::SyncJobStatus::Running => "running",
                crate::services::SyncJobStatus::Completed => "completed",
                crate::services::SyncJobStatus::Failed(_) => "failed",
                crate::services::SyncJobStatus::Cancelled => "cancelled",
            };
            
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "job": SyncJobStatusResponse {
                    job_id: job_handle.job_id.to_string(),
                    account_id: job_handle.account_id.to_string(),
                    status: status_str.to_string(),
                    started_at: job_handle.started_at,
                    documents_processed: job_handle.documents_processed,
                    total_documents: job_handle.total_documents,
                }
            })))
        }
        None => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "error": "Job not found or no longer active",
            })))
        }
    }
}

/// Get status of all active sync jobs
pub async fn get_active_sync_jobs(
    ingestion_service: web::Data<IngestionService>,
) -> Result<HttpResponse> {
    let active_jobs = ingestion_service.get_active_jobs().await;
    
    let jobs_response: Vec<SyncJobStatusResponse> = active_jobs.into_iter()
        .map(|job| {
            let status_str = match job.status {
                crate::services::SyncJobStatus::Running => "running",
                crate::services::SyncJobStatus::Completed => "completed",
                crate::services::SyncJobStatus::Failed(_) => "failed",
                crate::services::SyncJobStatus::Cancelled => "cancelled",
            };
            
            SyncJobStatusResponse {
                job_id: job.job_id.to_string(),
                account_id: job.account_id.to_string(),
                status: status_str.to_string(),
                started_at: job.started_at,
                documents_processed: job.documents_processed,
                total_documents: job.total_documents,
            }
        })
        .collect();
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "active_jobs": jobs_response,
        "count": jobs_response.len(),
    })))
}

/// Cancel a running sync job
pub async fn cancel_sync_job(
    ingestion_service: web::Data<IngestionService>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let job_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": "Invalid job ID",
            })));
        }
    };
    
    match ingestion_service.cancel_job(job_id).await {
        Ok(_) => {
            info!("✅ Sync job cancelled: {}", job_id);
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "Sync job cancelled successfully",
            })))
        }
        Err(e) => {
            error!("Failed to cancel sync job: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": e.to_string(),
            })))
        }
    }
}
