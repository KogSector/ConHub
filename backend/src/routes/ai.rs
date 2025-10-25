use actix_web::{web, HttpResponse, Result};
use crate::state::AppState;

pub async fn chat(
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Not implemented yet"
    })))
}

pub async fn complete(
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Not implemented yet"
    })))
}

pub fn configure_ai_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/ai")
            .route("/chat", web::post().to(chat))
            .route("/complete", web::post().to(complete))
    );
}
