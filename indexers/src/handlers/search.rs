use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Instant;

use crate::models::*;
use crate::services::fusion::IntermediateResult;
use crate::IndexerState;

pub async fn search(
    state: web::Data<IndexerState>,
    request: web::Json<SearchRequest>,
) -> Result<HttpResponse> {
    let start_time = Instant::now();
    
    log::info!("Search request: {}", request.query);

    let limit = request.limit.unwrap_or(10);
    let offset = request.offset.unwrap_or(0);

    
    let results = match request.source_type.as_deref() {
        Some("code") | Some("repository") => {
            state.code_indexer.search(&request.query, limit, offset).await
        }
        Some("documentation") | Some("url") | Some("web") => {
            state.web_indexer.search(&request.query, limit, offset).await
        }
        Some("file") => {
            state.doc_indexer.search(&request.query, limit, offset).await
        }
        None | Some(_) => {
            
            let mut all_results = Vec::new();
            
            if let Ok(code_results) = state.code_indexer.search(&request.query, limit, offset).await {
                all_results.extend(code_results);
            }
            
            if let Ok(web_results) = state.web_indexer.search(&request.query, limit, offset).await {
                all_results.extend(web_results);
            }
            
            if let Ok(doc_results) = state.doc_indexer.search(&request.query, limit, offset).await {
                all_results.extend(doc_results);
            }
            
            
            all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
            all_results.truncate(limit);
            
            Ok(all_results)
        }
    };

    match results {
        Ok(search_results) => {
            let processing_time_ms = start_time.elapsed().as_millis() as u64;
            
            Ok(HttpResponse::Ok().json(SearchResponse {
                results: search_results.clone(),
                total_count: search_results.len(),
                query: request.query.clone(),
                processing_time_ms,
            }))
        }
        Err(e) => {
            log::error!("Search failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Search failed",
                "message": e.to_string()
            })))
        }
    }
}

pub async fn search_code(
    state: web::Data<IndexerState>,
    request: web::Json<SearchRequest>,
) -> Result<HttpResponse> {
    let start_time = Instant::now();
    
    log::info!("Code search request: {}", request.query);

    let limit = request.limit.unwrap_or(10);
    let offset = request.offset.unwrap_or(0);

    match state.code_indexer.search(&request.query, limit, offset).await {
        Ok(search_results) => {
            let processing_time_ms = start_time.elapsed().as_millis() as u64;
            
            Ok(HttpResponse::Ok().json(SearchResponse {
                results: search_results.clone(),
                total_count: search_results.len(),
                query: request.query.clone(),
                processing_time_ms,
            }))
        }
        Err(e) => {
            log::error!("Code search failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Code search failed",
                "message": e.to_string()
            })))
        }
    }
}
