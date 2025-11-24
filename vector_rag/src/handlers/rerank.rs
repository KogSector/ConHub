use actix_web::{web, HttpResponse};
use std::sync::Arc;

use crate::models::{RerankRequest, RerankResponse, RerankResult, ErrorResponse};
use crate::services::FusionEmbeddingService;
use crate::services::embedding::VectorOps;

pub async fn rerank_handler(
    req: web::Json<RerankRequest>,
    service: web::Data<Arc<FusionEmbeddingService>>,
) -> HttpResponse {
    // Validate input
    if req.query.trim().is_empty() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "Query cannot be empty".to_string(),
        });
    }

    if req.documents.is_empty() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "Documents cannot be empty".to_string(),
        });
    }

    // Prepare inputs
    let query = req.query.clone();
    let doc_ids: Vec<String> = req.documents.iter().map(|d| d.id.clone()).collect();
    let doc_texts: Vec<String> = req.documents.iter().map(|d| d.text.clone()).collect();

    // Generate embeddings
    let query_emb = match service.generate_embeddings(&[query], "general_text").await {
        Ok(mut v) => {
            if v.is_empty() { return HttpResponse::InternalServerError().json(ErrorResponse { error: "Failed to generate query embedding".to_string() }); }
            v.remove(0)
        }
        Err(e) => {
            log::error!("Query embedding failed: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse { error: format!("Query embedding failed: {}", e) });
        }
    };

    let doc_embs = match service.generate_embeddings(&doc_texts, "general_text").await {
        Ok(v) => v,
        Err(e) => {
            log::error!("Document embeddings failed: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse { error: format!("Document embeddings failed: {}", e) });
        }
    };

    // Score documents by cosine similarity
    let mut results: Vec<RerankResult> = doc_ids
        .into_iter()
        .zip(doc_embs.into_iter())
        .map(|(id, emb)| {
            let score = VectorOps::cosine_similarity(&query_emb, &emb);
            RerankResult { id, score }
        })
        .collect();

    // Sort descending by score and truncate to top_k
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    let k = req.top_k.unwrap_or(results.len()).min(results.len());
    results.truncate(k);

    HttpResponse::Ok().json(RerankResponse { results })
}