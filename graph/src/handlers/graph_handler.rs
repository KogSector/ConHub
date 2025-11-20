use actix_web::{web, HttpResponse};
use crate::models::*;
use crate::errors::GraphResult;
use sqlx::PgPool;

pub async fn traverse_graph(
    _pool: web::Data<PgPool>,
    _req: web::Json<GraphTraversalRequest>,
) -> GraphResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(GraphTraversalResponse {
        paths: Vec::new(),
        visited_nodes: 0,
    }))
}

pub async fn find_related(
    _pool: web::Data<PgPool>,
    _req: web::Json<FindRelatedRequest>,
) -> GraphResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "related_entities": []
    })))
}
