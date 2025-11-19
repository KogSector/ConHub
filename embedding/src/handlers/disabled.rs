use actix_web::{HttpResponse, web};
use crate::models::EmbeddingStatus;

pub async fn disabled_handler(status: web::Data<EmbeddingStatus>) -> HttpResponse {
    let reason = status.reason.clone().unwrap_or_else(|| "disabled".to_string());
    HttpResponse::ServiceUnavailable().json(serde_json::json!({
        "error": "Embedding unavailable",
        "reason": reason,
        "available": false,
    }))
}