use actix_web::HttpResponse;

pub async fn disabled_handler() -> HttpResponse {
    HttpResponse::ServiceUnavailable().json(serde_json::json!({
        "error": "Embedding/rerank disabled by feature toggle",
        "feature": "Heavy",
    }))
}