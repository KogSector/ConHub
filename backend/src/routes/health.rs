use actix_web::{HttpResponse, Result};

pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "conhub-backend"
    })))
}

pub async fn readiness_check() -> Result<HttpResponse> {
    // TODO: Check database connectivity, Redis, Qdrant, etc.
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "ready",
        "service": "conhub-backend"
    })))
}
