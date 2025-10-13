use actix_web::{web, HttpResponse, Result};
use serde_json::json;
use validator::Validate;

use crate::models::*;
use crate::IndexerState;

pub async fn index_repository(
    state: web::Data<IndexerState>,
    request: web::Json<IndexRepositoryRequest>,
) -> Result<HttpResponse> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": "Validation failed",
            "details": validation_errors.to_string()
        })));
    }

    log::info!("Indexing repository: {}", request.repository_url);

    let mut metadata = std::collections::HashMap::new();
    if let Some(ref branch) = request.branch {
        metadata.insert("branch".to_string(), branch.clone());
    }
    if let Some(ref patterns) = request.include_patterns {
        metadata.insert("include_patterns".to_string(), patterns.join(","));
    }
    if let Some(ref patterns) = request.exclude_patterns {
        metadata.insert("exclude_patterns".to_string(), patterns.join(","));
    }

    match state.code_indexer.index_repository(
        request.repository_url.clone(),
        request.branch.clone().unwrap_or_else(|| "main".to_string()),
        metadata,
    ).await {
        Ok(job) => {
            log::info!("Repository indexing job started: {}", job.id);
            Ok(HttpResponse::Ok().json(IndexingResponse {
                success: true,
                job_id: job.id,
                message: "Repository indexing started successfully".to_string(),
                status: IndexingStatus::InProgress,
            }))
        }
        Err(e) => {
            log::error!("Failed to start repository indexing: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Failed to start repository indexing",
                "message": e.to_string()
            })))
        }
    }
}

pub async fn index_documentation(
    state: web::Data<IndexerState>,
    request: web::Json<IndexDocumentationRequest>,
) -> Result<HttpResponse> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": "Validation failed",
            "details": validation_errors.to_string()
        })));
    }

    log::info!("Indexing documentation: {}", request.documentation_url);

    let mut metadata = std::collections::HashMap::new();
    if let Some(ref doc_type) = request.doc_type {
        metadata.insert("doc_type".to_string(), doc_type.clone());
    }
    if let Some(crawl_depth) = request.crawl_depth {
        metadata.insert("crawl_depth".to_string(), crawl_depth.to_string());
    }

    match state.doc_indexer.index_documentation(
        request.documentation_url.clone(),
        request.crawl_depth.unwrap_or(2),
        metadata,
    ).await {
        Ok(job) => {
            log::info!("Documentation indexing job started: {}", job.id);
            Ok(HttpResponse::Ok().json(IndexingResponse {
                success: true,
                job_id: job.id,
                message: "Documentation indexing started successfully".to_string(),
                status: IndexingStatus::InProgress,
            }))
        }
        Err(e) => {
            log::error!("Failed to start documentation indexing: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Failed to start documentation indexing",
                "message": e.to_string()
            })))
        }
    }
}

pub async fn index_url(
    state: web::Data<IndexerState>,
    request: web::Json<IndexUrlRequest>,
) -> Result<HttpResponse> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": "Validation failed",
            "details": validation_errors.to_string()
        })));
    }

    log::info!("Indexing URL: {}", request.url);

    let mut metadata = std::collections::HashMap::new();
    if let Some(ref content_type) = request.content_type {
        metadata.insert("content_type".to_string(), content_type.clone());
    }

    match state.web_indexer.index_url(
        request.url.clone(),
        request.max_depth.unwrap_or(1),
        metadata,
    ).await {
        Ok(job) => {
            log::info!("URL indexing job started: {}", job.id);
            Ok(HttpResponse::Ok().json(IndexingResponse {
                success: true,
                job_id: job.id,
                message: "URL indexing started successfully".to_string(),
                status: IndexingStatus::InProgress,
            }))
        }
        Err(e) => {
            log::error!("Failed to start URL indexing: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Failed to start URL indexing",
                "message": e.to_string()
            })))
        }
    }
}

pub async fn index_file(
    state: web::Data<IndexerState>,
    request: web::Json<IndexFileRequest>,
) -> Result<HttpResponse> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": "Validation failed",
            "details": validation_errors.to_string()
        })));
    }

    log::info!("Indexing file: {}", request.file_path);

    let metadata = request.metadata.clone().unwrap_or_default();

    match state.doc_indexer.index_file(
        request.file_path.clone(),
        metadata,
    ).await {
        Ok(job) => {
            log::info!("File indexing job started: {}", job.id);
            Ok(HttpResponse::Ok().json(IndexingResponse {
                success: true,
                job_id: job.id,
                message: "File indexing started successfully".to_string(),
                status: IndexingStatus::InProgress,
            }))
        }
        Err(e) => {
            log::error!("Failed to start file indexing: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Failed to start file indexing",
                "message": e.to_string()
            })))
        }
    }
}

pub async fn index_code(
    state: web::Data<IndexerState>,
    request: web::Json<IndexRepositoryRequest>,
) -> Result<HttpResponse> {
    
    index_repository(state, request).await
}
