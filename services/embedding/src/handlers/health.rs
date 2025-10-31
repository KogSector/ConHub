use actix_web::{HttpResponse};

use crate::models::HealthResponse;

pub async fn health_handler() -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "healthy".to_string(),
        model_loaded: true,
    })
}
