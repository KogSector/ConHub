use actix_web::{web, HttpResponse, Result};
use crate::state::AppState;

pub async fn create_subscription(
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Not implemented yet"
    })))
}

pub async fn cancel_subscription(
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Not implemented yet"
    })))
}

pub async fn get_subscription(
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Not implemented yet"
    })))
}

pub async fn get_plans(
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Not implemented yet"
    })))
}

pub fn configure_billing_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/billing")
            .route("/subscription", web::post().to(create_subscription))
            .route("/subscription", web::get().to(get_subscription))
            .route("/subscription/{id}", web::delete().to(cancel_subscription))
            .route("/plans", web::get().to(get_plans))
    );
}
