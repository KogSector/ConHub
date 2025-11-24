use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use crate::services::FusionEmbeddingService;
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
    
    // Determine collection name
    let collection_type = req.collection_type.as_deref().unwrap_or("code");
    let collection_name = format!("tenant_{}_{}", req.tenant_id, collection_type);
    
    // Build Qdrant filter from SearchFilters
    let qdrant_filter = build_qdrant_filter(&req.filters);
    
    let top_k = req.top_k.unwrap_or(10);
    
    // TODO: Actual Qdrant search call would go here
    // For now, return empty results with proper structure
    log::info!("Searching in collection: {} (top_k: {})", collection_name, top_k);
    
    let query_time_ms = start.elapsed().as_millis() as u64;
    
    HttpResponse::Ok().json(VectorSearchResponse {
        results: vec![],
        total: 0,
        query_time_ms,
    })
}

fn build_qdrant_filter(filters: &Option<SearchFilters>) -> Option<serde_json::Value> {
    let Some(filters) = filters else {
        return None;
    };
    
    let mut conditions = Vec::new();
    
    if let Some(connector_types) = &filters.connector_types {
        conditions.push(serde_json::json!({
            "key": "connector_type",
            "match": {"any": connector_types}
        }));
    }
    
    if let Some(repositories) = &filters.repositories {
        conditions.push(serde_json::json!({
            "key": "repository",
            "match": {"any": repositories}
        }));
    }
    
    if let Some(authors) = &filters.authors {
        conditions.push(serde_json::json!({
            "key": "authors",
            "match": {"any": authors}
        }));
    }
    
    if let Some(tags) = &filters.tags {
        conditions.push(serde_json::json!({
            "key": "tags",
            "match": {"any": tags}
        }));
    }
    
    if conditions.is_empty() {
        None
    } else {
        Some(serde_json::json!({
            "must": conditions
        }))
    }
}

/// Search by chunk IDs - retrieve specific chunks
pub async fn search_by_ids(
    req: web::Json<SearchByIdsRequest>,
) -> HttpResponse {
    log::info!("Search by IDs for {} chunks (tenant: {})", 
              req.chunk_ids.len(), req.tenant_id);
    
    // TODO: Implement Qdrant retrieval by IDs
    // This would use Qdrant's retrieve API to fetch specific points
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
    // 2. Use entity embedding as query vector in Qdrant
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
