use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Instant;

use crate::models::*;
use crate::services::fusion::IntermediateResult;
use crate::services::state::IndexerState;

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

/// Fusion search request
#[derive(Debug, Deserialize)]
pub struct FusionSearchRequest {
    pub query: String,
    pub filters: Option<FusionFilters>,
}

#[derive(Debug, Deserialize)]
pub struct FusionFilters {
    pub source_types: Option<Vec<String>>,
    pub languages: Option<Vec<String>>,
    pub date_range: Option<DateRange>,
}

#[derive(Debug, Deserialize)]
pub struct DateRange {
    pub start: Option<String>,
    pub end: Option<String>,
}

/// Fusion search response
#[derive(Debug, Serialize)]
pub struct FusionSearchResponse {
    pub results: Vec<FusionSearchResult>,
    pub count: usize,
    pub strategy: String,
    pub processing_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct FusionSearchResult {
    pub id: String,
    pub score: f32,
    pub source: String,
    pub content: String,
    pub metadata: serde_json::Value,
}

/// Fusion search handler combining keyword + vector search with RRF and reranking
pub async fn fusion_search(
    state: web::Data<IndexerState>,
    request: web::Json<FusionSearchRequest>,
) -> Result<HttpResponse> {
    let start_time = Instant::now();

    // Validate query
    if request.query.is_empty() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Query cannot be empty"
        })));
    }

    if request.query.len() > 500 {
        return Ok(HttpResponse::PayloadTooLarge().json(json!({
            "error": "Query exceeds maximum length of 500 characters"
        })));
    }

    log::info!("Fusion search request: {}", request.query);

    // Check if fusion components are available
    let fusion_service = match &state.fusion_service {
        Some(service) => service,
        None => {
            log::warn!("Fusion service not available, falling back to basic search");
            return fallback_basic_search(state, request.query.clone(), start_time).await;
        }
    };

    let qdrant_service = match &state.qdrant_service {
        Some(service) => service,
        None => {
            log::warn!("Qdrant service not available, falling back to basic search");
            return fallback_basic_search(state, request.query.clone(), start_time).await;
        }
    };

    // Perform keyword search across all indexes
    let keyword_results = match perform_keyword_search(&state, &request.query).await {
        Ok(results) => results,
        Err(e) => {
            log::error!("Keyword search failed: {}", e);
            Vec::new()
        }
    };

    // Perform fusion search
    match fusion_service
        .fusion_search(
            request.query.clone(),
            keyword_results,
            qdrant_service,
            "conhub_knowledge",
        )
        .await
    {
        Ok(fusion_results) => {
            let processing_time_ms = start_time.elapsed().as_millis() as u64;

            let results: Vec<FusionSearchResult> = fusion_results
                .into_iter()
                .map(|r| FusionSearchResult {
                    id: r.id,
                    score: r.score,
                    source: r.source,
                    content: r.content,
                    metadata: r.metadata,
                })
                .collect();

            Ok(HttpResponse::Ok().json(FusionSearchResponse {
                count: results.len(),
                results,
                strategy: "fusion_rrf_rerank".to_string(),
                processing_time_ms,
            }))
        }
        Err(e) => {
            log::error!("Fusion search failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Fusion search failed",
                "message": e.to_string()
            })))
        }
    }
}

/// Perform keyword search across all indexes
async fn perform_keyword_search(
    state: &IndexerState,
    query: &str,
) -> anyhow::Result<Vec<IntermediateResult>> {
    let mut keyword_results = Vec::new();
    let limit = 100;
    let offset = 0;

    // Search code index
    if let Ok(code_results) = state.code_indexer.search(query, limit, offset).await {
        for (rank, result) in code_results.into_iter().enumerate() {
            keyword_results.push(IntermediateResult {
                id: result.id.clone().unwrap_or_default(),
                rank,
                score: result.score,
                source: "code".to_string(),
                content: result.content.clone().unwrap_or_default(),
                metadata: json!({
                    "title": result.title,
                    "file_path": result.file_path,
                    "language": result.language,
                }),
            });
        }
    }

    // Search web index
    if let Ok(web_results) = state.web_indexer.search(query, limit, offset).await {
        for (rank, result) in web_results.into_iter().enumerate() {
            keyword_results.push(IntermediateResult {
                id: result.id.clone().unwrap_or_default(),
                rank,
                score: result.score,
                source: "web".to_string(),
                content: result.content.clone().unwrap_or_default(),
                metadata: json!({
                    "title": result.title,
                    "url": result.url,
                }),
            });
        }
    }

    // Search document index
    if let Ok(doc_results) = state.doc_indexer.search(query, limit, offset).await {
        for (rank, result) in doc_results.into_iter().enumerate() {
            keyword_results.push(IntermediateResult {
                id: result.id.clone().unwrap_or_default(),
                rank,
                score: result.score,
                source: "doc".to_string(),
                content: result.content.clone().unwrap_or_default(),
                metadata: json!({
                    "title": result.title,
                    "file_path": result.file_path,
                }),
            });
        }
    }

    // Sort by score and take top 100
    keyword_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    keyword_results.truncate(100);

    Ok(keyword_results)
}

/// Fallback to basic search when fusion components unavailable
async fn fallback_basic_search(
    state: web::Data<IndexerState>,
    query: String,
    start_time: Instant,
) -> Result<HttpResponse> {
    let limit = 20;
    let offset = 0;

    let mut all_results = Vec::new();

    if let Ok(code_results) = state.code_indexer.search(&query, limit, offset).await {
        all_results.extend(code_results);
    }

    if let Ok(web_results) = state.web_indexer.search(&query, limit, offset).await {
        all_results.extend(web_results);
    }

    if let Ok(doc_results) = state.doc_indexer.search(&query, limit, offset).await {
        all_results.extend(doc_results);
    }

    all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    all_results.truncate(20);

    let processing_time_ms = start_time.elapsed().as_millis() as u64;

    let results: Vec<FusionSearchResult> = all_results
        .into_iter()
        .map(|r| FusionSearchResult {
            id: r.id.unwrap_or_default(),
            score: r.score,
            source: "unknown".to_string(),
            content: r.content.unwrap_or_default(),
            metadata: json!({
                "title": r.title,
            }),
        })
        .collect();

    Ok(HttpResponse::Ok().json(FusionSearchResponse {
        count: results.len(),
        results,
        strategy: "keyword_only".to_string(),
        processing_time_ms,
    }))
}
