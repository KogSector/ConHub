use actix_web::{web, HttpResponse, Result};
use crate::state::AppState;

pub async fn scan_repository(
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Not implemented yet"
    })))
}

pub async fn get_security_report(
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Not implemented yet"
    })))
}

pub fn configure_security_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/security")
            .route("/scan", web::post().to(scan_repository))
            .route("/report/{id}", web::get().to(get_security_report))
    );
}
