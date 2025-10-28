use actix_web::{web, HttpResponse};
use std::sync::Arc;

use crate::models::{ErrorResponse, RerankRequest, RerankResponse, RerankResult};
use crate::services::RerankService;

const MAX_RERANK_DOCS: usize = 100;

pub async fn rerank_handler(
    req: web::Json<RerankRequest>,
    service: web::Data<Arc<RerankService>>,
) -> HttpResponse {
    // Validate query
    if req.query.is_empty() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "Query cannot be empty".to_string(),
        });
    }

    // Validate documents
    if req.documents.is_empty() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "Documents list cannot be empty".to_string(),
        });
    }

    if req.documents.len() > MAX_RERANK_DOCS {
        return HttpResponse::PayloadTooLarge().json(ErrorResponse {
            error: format!("Too many documents (limit: {})", MAX_RERANK_DOCS),
        });
    }

    // Prepare documents for reranking
    let docs: Vec<(String, String)> = req
        .documents
        .iter()
        .map(|doc| (doc.id.clone(), doc.text.clone()))
        .collect();

    // Perform reranking
    match service.rerank(&req.query, docs) {
        Ok(ranked) => {
            // Apply top_k if specified
            let results: Vec<RerankResult> = if let Some(top_k) = req.top_k {
                ranked
                    .into_iter()
                    .take(top_k)
                    .map(|(id, score)| RerankResult { id, score })
                    .collect()
            } else {
                ranked
                    .into_iter()
                    .map(|(id, score)| RerankResult { id, score })
                    .collect()
            };

            HttpResponse::Ok().json(RerankResponse { results })
        }
        Err(e) => {
            log::error!("Reranking failed: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Reranking failed: {}", e),
            })
        }
    }
}
