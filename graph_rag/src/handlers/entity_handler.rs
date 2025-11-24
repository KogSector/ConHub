use actix_web::{web, HttpResponse};
use crate::models::{CreateEntityRequest, CreateEntityResponse, Entity};
use crate::errors::GraphResult;
use crate::services::GraphService;
use sqlx::PgPool;
use std::sync::Arc;
use crate::graph_db::Neo4jClient;

pub async fn create_entity(
    pool: web::Data<PgPool>,
    req: web::Json<CreateEntityRequest>,
) -> GraphResult<HttpResponse> {
    let graph_service = GraphService::new(pool.get_ref().clone());
    let response = graph_service.create_entity(req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_entity(
    pool: web::Data<PgPool>,
    neo4j: web::Data<Option<Arc<Neo4jClient>>>,
    entity_id: web::Path<uuid::Uuid>,
) -> GraphResult<HttpResponse> {
    let graph_service = GraphService::new(pool.get_ref().clone());
    
    match graph_service.get_entity(*entity_id).await {
        Ok(entity) => Ok(HttpResponse::Ok().json(entity)),
        Err(e) => {
            tracing::warn!("Entity {} not found: {}", entity_id, e);
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Entity not found",
                "entity_id": entity_id.to_string()
            })))
        }
    }
}

pub async fn get_neighbors(
    pool: web::Data<PgPool>,
    neo4j: web::Data<Option<Arc<Neo4jClient>>>,
    entity_id: web::Path<uuid::Uuid>,
    query: web::Query<NeighborsQuery>,
) -> GraphResult<HttpResponse> {
    // For now return empty neighbors - full implementation would query Neo4j
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "entity_id": entity_id.to_string(),
        "neighbors": [],
        "total": 0
    })))
}

pub async fn find_paths(
    pool: web::Data<PgPool>,
    neo4j: web::Data<Option<Arc<Neo4jClient>>>,
    query: web::Query<PathQuery>,
) -> GraphResult<HttpResponse> {
    // For now return empty paths - full implementation would use Neo4j path finding
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "from": query.from_id,
        "to": query.to_id,
        "paths": [],
        "total": 0
    })))
}

pub async fn get_statistics(
    pool: web::Data<PgPool>,
    neo4j: web::Data<Option<Arc<Neo4jClient>>>,
) -> GraphResult<HttpResponse> {
    // Query entity and relationship counts
    let entity_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM entities")
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);
    
    let relationship_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM relationships")
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "total_entities": entity_count,
        "total_relationships": relationship_count,
        "neo4j_connected": neo4j.is_some()
    })))
}

#[derive(Debug, serde::Deserialize)]
pub struct NeighborsQuery {
    pub relationship_types: Option<Vec<String>>,
    pub depth: Option<usize>,
}

#[derive(Debug, serde::Deserialize)]
pub struct PathQuery {
    pub from_id: String,
    pub to_id: String,
    pub max_hops: Option<usize>,
}
