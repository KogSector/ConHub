use actix_web::{web, HttpResponse};
use crate::models::{CreateEntityRequest, CreateEntityResponse};
use crate::errors::GraphResult;
use sqlx::PgPool;

pub async fn create_entity(
    pool: web::Data<PgPool>,
    req: web::Json<CreateEntityRequest>,
) -> GraphResult<HttpResponse> {
    // Implementation would create entity and run resolution
    Ok(HttpResponse::Ok().json(CreateEntityResponse {
        entity_id: uuid::Uuid::new_v4(),
        canonical_id: None,
        resolved: false,
    }))
}

pub async fn get_entity(
    pool: web::Data<PgPool>,
    entity_id: web::Path<uuid::Uuid>,
) -> GraphResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({"status": "ok"})))
}
