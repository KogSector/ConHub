use actix_web::{web, HttpResponse, Result};
use serde_json::json;
use validator::Validate;
use std::collections::HashMap;

use crate::services::indexing_service::{IndexingService, IndexingRequest, IndexingSourceType, IndexingPriority};

#[derive(serde::Deserialize, Validate)]
pub struct IndexRepositoryRequest {
    #[validate(url)]
    pub repository_url: String,
    pub branch: Option<String>,
    pub include_patterns: Option<Vec<String>>,
    pub exclude_patterns: Option<Vec<String>>,
    pub language_filters: Option<Vec<String>>,
    pub max_file_size: Option<usize>,
}

#[derive(serde::Deserialize, Validate)]
pub struct IndexDocumentationRequest {
    #[validate(url)]
    pub documentation_url: String,
    pub doc_type: Option<String>,
    pub crawl_depth: Option<u32>,
    pub follow_links: Option<bool>,
    pub extract_code_blocks: Option<bool>,
}

#[derive(serde::Deserialize, Validate)]
pub struct IndexUrlRequest {
    #[validate(url)]
    pub url: String,
    pub content_type: Option<String>,
    pub extract_links: Option<bool>,
}

#[derive(serde::Deserialize, Validate)]
pub struct IndexFileRequest {
    pub file_path: String,
    pub file_type: Option<String>,
}

pub async fn index_repository(request: web::Json<IndexRepositoryRequest>) -> Result<HttpResponse> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    let indexing_service = IndexingService::new();
    
    // Prepare metadata
    let mut metadata = HashMap::new();
    if let Some(branch) = &request.branch {
        metadata.insert("branch".to_string(), branch.clone());
    }
    if let Some(include_patterns) = &request.include_patterns {
        metadata.insert("include_patterns".to_string(), include_patterns.join(","));
    }
    if let Some(exclude_patterns) = &request.exclude_patterns {
        metadata.insert("exclude_patterns".to_string(), exclude_patterns.join(","));
    }
    if let Some(language_filters) = &request.language_filters {
        metadata.insert("language_filters".to_string(), language_filters.join(","));
    }
    if let Some(max_file_size) = request.max_file_size {
        metadata.insert("max_file_size".to_string(), max_file_size.to_string());
    }

    let indexing_request = IndexingRequest {
        id: uuid::Uuid::new_v4().to_string(),
        source_type: IndexingSourceType::Repository,
        source_url: request.repository_url.clone(),
        metadata,
        priority: IndexingPriority::Normal,
        created_at: chrono::Utc::now(),
    };

    match indexing_service.index_content(indexing_request).await {
        Ok(result) => {
            log::info!("Repository indexing completed: {:?}", result);
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "message": "Repository indexing completed successfully",
                "result": {
                    "request_id": result.request_id,
                    "status": format!("{:?}", result.status),
                    "documents_processed": result.documents_processed,
                    "chunks_created": result.chunks_created,
                    "embeddings_generated": result.embeddings_generated,
                    "processing_time_ms": result.processing_time_ms,
                    "completed_at": result.completed_at
                }
            })))
        }
        Err(e) => {
            log::error!("Repository indexing failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Repository indexing failed",
                "message": e.to_string()
            })))
        }
    }
}

pub async fn index_documentation(request: web::Json<IndexDocumentationRequest>) -> Result<HttpResponse> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    let indexing_service = IndexingService::new();
    
    // Prepare metadata
    let mut metadata = HashMap::new();
    if let Some(doc_type) = &request.doc_type {
        metadata.insert("doc_type".to_string(), doc_type.clone());
    }
    if let Some(crawl_depth) = request.crawl_depth {
        metadata.insert("crawl_depth".to_string(), crawl_depth.to_string());
    }
    if let Some(follow_links) = request.follow_links {
        metadata.insert("follow_links".to_string(), follow_links.to_string());
    }
    if let Some(extract_code_blocks) = request.extract_code_blocks {
        metadata.insert("extract_code_blocks".to_string(), extract_code_blocks.to_string());
    }

    let indexing_request = IndexingRequest {
        id: uuid::Uuid::new_v4().to_string(),
        source_type: IndexingSourceType::Documentation,
        source_url: request.documentation_url.clone(),
        metadata,
        priority: IndexingPriority::Normal,
        created_at: chrono::Utc::now(),
    };

    match indexing_service.index_content(indexing_request).await {
        Ok(result) => {
            log::info!("Documentation indexing completed: {:?}", result);
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "message": "Documentation indexing completed successfully",
                "result": {
                    "request_id": result.request_id,
                    "status": format!("{:?}", result.status),
                    "documents_processed": result.documents_processed,
                    "chunks_created": result.chunks_created,
                    "embeddings_generated": result.embeddings_generated,
                    "processing_time_ms": result.processing_time_ms,
                    "completed_at": result.completed_at
                }
            })))
        }
        Err(e) => {
            log::error!("Documentation indexing failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Documentation indexing failed",
                "message": e.to_string()
            })))
        }
    }
}

pub async fn index_url(request: web::Json<IndexUrlRequest>) -> Result<HttpResponse> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    let indexing_service = IndexingService::new();
    
    // Prepare metadata
    let mut metadata = HashMap::new();
    if let Some(content_type) = &request.content_type {
        metadata.insert("content_type".to_string(), content_type.clone());
    }
    if let Some(extract_links) = request.extract_links {
        metadata.insert("extract_links".to_string(), extract_links.to_string());
    }

    let indexing_request = IndexingRequest {
        id: uuid::Uuid::new_v4().to_string(),
        source_type: IndexingSourceType::Url,
        source_url: request.url.clone(),
        metadata,
        priority: IndexingPriority::Normal,
        created_at: chrono::Utc::now(),
    };

    match indexing_service.index_content(indexing_request).await {
        Ok(result) => {
            log::info!("URL indexing completed: {:?}", result);
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "message": "URL indexing completed successfully",
                "result": {
                    "request_id": result.request_id,
                    "status": format!("{:?}", result.status),
                    "documents_processed": result.documents_processed,
                    "chunks_created": result.chunks_created,
                    "embeddings_generated": result.embeddings_generated,
                    "processing_time_ms": result.processing_time_ms,
                    "completed_at": result.completed_at
                }
            })))
        }
        Err(e) => {
            log::error!("URL indexing failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "URL indexing failed",
                "message": e.to_string()
            })))
        }
    }
}

pub async fn index_file(request: web::Json<IndexFileRequest>) -> Result<HttpResponse> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    let indexing_service = IndexingService::new();
    
    // Prepare metadata
    let mut metadata = HashMap::new();
    if let Some(file_type) = &request.file_type {
        metadata.insert("file_type".to_string(), file_type.clone());
    }

    let indexing_request = IndexingRequest {
        id: uuid::Uuid::new_v4().to_string(),
        source_type: IndexingSourceType::File,
        source_url: request.file_path.clone(),
        metadata,
        priority: IndexingPriority::Normal,
        created_at: chrono::Utc::now(),
    };

    match indexing_service.index_content(indexing_request).await {
        Ok(result) => {
            log::info!("File indexing completed: {:?}", result);
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "message": "File indexing completed successfully",
                "result": {
                    "request_id": result.request_id,
                    "status": format!("{:?}", result.status),
                    "documents_processed": result.documents_processed,
                    "chunks_created": result.chunks_created,
                    "embeddings_generated": result.embeddings_generated,
                    "processing_time_ms": result.processing_time_ms,
                    "completed_at": result.completed_at
                }
            })))
        }
        Err(e) => {
            log::error!("File indexing failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "File indexing failed",
                "message": e.to_string()
            })))
        }
    }
}

pub async fn get_indexing_status() -> Result<HttpResponse> {
    // In a real implementation, this would check the status of ongoing indexing jobs
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "status": "operational",
        "active_jobs": 0,
        "completed_jobs": 0,
        "failed_jobs": 0,
        "queue_size": 0
    })))
}

pub fn configure_indexing_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/indexing")
            .route("/repository", web::post().to(index_repository))
            .route("/documentation", web::post().to(index_documentation))
            .route("/url", web::post().to(index_url))
            .route("/file", web::post().to(index_file))
            .route("/status", web::get().to(get_indexing_status))
    );
}
