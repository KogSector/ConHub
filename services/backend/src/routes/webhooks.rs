use actix_web::{web, HttpResponse, Result};
use crate::state::AppState;

pub async fn stripe_webhook(
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Not implemented yet"
    })))
}

pub async fn github_webhook(
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Not implemented yet"
    })))
}

pub async fn gitlab_webhook(
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Not implemented yet"
    })))
}

pub fn configure_webhook_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/webhooks")
            .route("/stripe", web::post().to(stripe_webhook))
            .route("/github", web::post().to(github_webhook))
            .route("/gitlab", web::post().to(gitlab_webhook))
    );
}
