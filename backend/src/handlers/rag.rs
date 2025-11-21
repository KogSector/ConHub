use actix_web::{web, HttpResponse};
use crate::services::rag_service::{RagService, RagQueryRequest};
use std::sync::Arc;

pub async fn rag_query(
    req: web::Json<RagQueryRequest>,
    rag_service: web::Data<Arc<RagService>>,
) -> HttpResponse {
    log::info!("RAG query request: {} (mode: {:?})", req.query, req.mode);
    
    match rag_service.query(req.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            log::error!("RAG query failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Query failed",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn rag_vector(
    req: web::Json<RagQueryRequest>,
    rag_service: web::Data<Arc<RagService>>,
) -> HttpResponse {
    log::info!("Vector RAG query: {}", req.query);
    
    let mut request = req.into_inner();
    request.mode = Some(crate::services::rag_service::RagMode::Vector);
    
    match rag_service.query(request).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            log::error!("Vector RAG query failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Query failed",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn rag_hybrid(
    req: web::Json<RagQueryRequest>,
    rag_service: web::Data<Arc<RagService>>,
) -> HttpResponse {
    log::info!("Hybrid RAG query: {}", req.query);
    
    let mut request = req.into_inner();
    request.mode = Some(crate::services::rag_service::RagMode::Hybrid);
    
    match rag_service.query(request).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            log::error!("Hybrid RAG query failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Query failed",
                "details": e.to_string()
            }))
        }
    }
}

pub async fn rag_agentic(
    req: web::Json<RagQueryRequest>,
    rag_service: web::Data<Arc<RagService>>,
) -> HttpResponse {
    log::info!("Agentic RAG query: {}", req.query);
    
    let mut request = req.into_inner();
    request.mode = Some(crate::services::rag_service::RagMode::Agentic);
    
    match rag_service.query(request).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            log::error!("Agentic RAG query failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Query failed",
                "details": e.to_string()
            }))
        }
    }
}
