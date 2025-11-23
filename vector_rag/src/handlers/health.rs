use actix_web::{HttpResponse, web};

use crate::models::{HealthResponse, EmbeddingStatus};

pub async fn health_handler(status: web::Data<EmbeddingStatus>) -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "healthy".to_string(),
        model_loaded: status.available,
    })
}
