use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::env;
use crate::services::FusionEmbeddingService;
use crate::services::vector_store::{VectorStoreService, build_zilliz_filter};
use std::time::Instant;

#[derive(Debug, Deserialize)]
pub struct VectorSearchRequest {
    pub query_text: String,
    pub tenant_id: String,
    pub collection_type: Option<String>, // "code", "docs", "chats"
    pub top_k: Option<usize>,
    pub filters: Option<SearchFilters>,
}

#[derive(Debug, Deserialize)]
pub struct SearchFilters {
    pub connector_types: Option<Vec<String>>,
    pub repositories: Option<Vec<String>>,
    pub date_range: Option<DateRange>,
    pub authors: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct DateRange {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Serialize)]
pub struct VectorSearchResponse {
    pub results: Vec<SearchResult>,
    pub total: usize,
    pub query_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub chunk_id: String,
    pub content: String,
    pub score: f32,
    pub metadata: serde_json::Value,
    pub source: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchByIdsRequest {
    pub chunk_ids: Vec<String>,
    pub tenant_id: String,
}

#[derive(Debug, Serialize)]
pub struct SearchByIdsResponse {
    pub results: Vec<ChunkResult>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct ChunkResult {
    pub chunk_id: String,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub metadata: serde_json::Value,
}

/// Vector search endpoint - semantic search with filters
pub async fn vector_search(
    req: web::Json<VectorSearchRequest>,
    embedding_service: web::Data<Arc<FusionEmbeddingService>>,
) -> HttpResponse {
    let start = Instant::now();
    log::info!("Vector search request for tenant: {} (query: {})", 
              req.tenant_id, req.query_text);
    
    // Generate embedding for query text
    let texts = vec![req.query_text.clone()];
    let embeddings = match embedding_service.generate_embeddings(&texts, "code_query").await {
        Ok(embeddings) => embeddings,
        Err(e) => {
            log::error!("Failed to generate embedding: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to generate embedding",
                "details": e.to_string()
            }));
        }
    };
    let query_embedding = embeddings.get(0).cloned().unwrap_or_default();
    
    if query_embedding.is_empty() {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to generate query embedding"
        }));
    }
    
    // Get Zilliz configuration
    let zilliz_url = env::var("ZILLIZ_PUBLIC_ENDPOINT")
        .unwrap_or_else(|_| env::var("ZILLIZ_ENDPOINT").unwrap_or_else(|_| "https://localhost:19530".to_string()));
    let collection = env::var("ZILLIZ_COLLECTION")
        .unwrap_or_else(|_| "conhub_embeddings".to_string());
    
    let top_k = req.top_k.unwrap_or(10);
    
    // Build Zilliz filter from SearchFilters
    let filter = build_search_filter(&req.filters, Some(&req.tenant_id));
    
    // Initialize Zilliz client and perform search
    let store = match VectorStoreService::new(&zilliz_url, 30).await {
        Ok(store) => store,
        Err(e) => {
            log::error!("Failed to initialize Zilliz client: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to initialize vector store",
                "details": e.to_string()
            }));
        }
    };
    
    // Define output fields to retrieve
    let output_fields = vec![
        "chunk_id".to_string(),
        "content_preview".to_string(),
        "connector_type".to_string(),
        "path".to_string(),
        "source_id".to_string(),
        "repository".to_string(),
        "metadata".to_string(),
    ];
    
    log::info!("Searching Zilliz collection '{}' (top_k: {}, filter: {:?})", 
              collection, top_k, filter);
    
    // Perform the search
    let search_results = match store.search(
        &collection,
        query_embedding,
        top_k,
        filter,
        Some(output_fields),
    ).await {
        Ok(results) => results,
        Err(e) => {
            log::error!("Zilliz search failed: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Search failed",
                "details": e.to_string()
            }));
        }
    };
    
    // Transform results to response format
    let results: Vec<SearchResult> = search_results
        .into_iter()
        .map(|r| {
            let content = r.fields
                .get("content_preview")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            let source = r.fields
                .get("connector_type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            
            SearchResult {
                chunk_id: r.id,
                content,
                score: 1.0 - r.distance, // Convert distance to similarity score
                metadata: serde_json::json!(r.fields),
                source,
            }
        })
        .collect();
    
    let total = results.len();
    let query_time_ms = start.elapsed().as_millis() as u64;
    
    log::info!("Search completed: {} results in {}ms", total, query_time_ms);
    
    HttpResponse::Ok().json(VectorSearchResponse {
        results,
        total,
        query_time_ms,
    })
}

/// Build a Zilliz-compatible filter string from SearchFilters
fn build_search_filter(filters: &Option<SearchFilters>, tenant_id: Option<&str>) -> Option<String> {
    let filters_ref = filters.as_ref();
    
    build_zilliz_filter(
        filters_ref.and_then(|f| f.connector_types.as_ref()).map(|v| v.as_slice()),
        filters_ref.and_then(|f| f.repositories.as_ref()).map(|v| v.as_slice()),
        filters_ref.and_then(|f| f.authors.as_ref()).map(|v| v.as_slice()),
        filters_ref.and_then(|f| f.tags.as_ref()).map(|v| v.as_slice()),
        tenant_id,
    )
}

/// Search by chunk IDs - retrieve specific chunks
pub async fn search_by_ids(
    req: web::Json<SearchByIdsRequest>,
) -> HttpResponse {
    log::info!("Search by IDs for {} chunks (tenant: {})", 
              req.chunk_ids.len(), req.tenant_id);
    
    // TODO: Implement Zilliz retrieval by IDs
    // This would use Zilliz's retrieve API to fetch specific points
    // For now, return empty results
    
    HttpResponse::Ok().json(SearchByIdsResponse {
        results: vec![],
        total: 0,
    })
}

/// Search by entity - use entity embeddings as query
pub async fn search_by_entity(
    entity_id: web::Path<String>,
    query: web::Query<EntitySearchQuery>,
) -> HttpResponse {
    let start = Instant::now();
    log::info!("Search by entity: {} (tenant: {})", entity_id, query.tenant_id);
    
    // TODO: Implement entity-based search
    // 1. Call graph service to get entity embedding
    // 2. Use entity embedding as query vector in Zilliz
    // 3. Apply filters and return related chunks
    
    let query_time_ms = start.elapsed().as_millis() as u64;
    
    HttpResponse::Ok().json(VectorSearchResponse {
        results: vec![],
        total: 0,
        query_time_ms,
    })
}

#[derive(Debug, Deserialize)]
pub struct EntitySearchQuery {
    pub tenant_id: String,
    pub top_k: Option<usize>,
    pub filters: Option<SearchFilters>,
}
