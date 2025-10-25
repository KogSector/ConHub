use actix_web::{web, HttpResponse, Result};
use crate::state::AppState;

pub async fn connect_source(
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Not implemented yet"
    })))
}

pub async fn sync_source(
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Not implemented yet"
    })))
}

pub async fn list_sources(
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Not implemented yet"
    })))
}

pub fn configure_data_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/data")
            .route("/sources", web::post().to(connect_source))
            .route("/sources", web::get().to(list_sources))
            .route("/sources/{id}/sync", web::post().to(sync_source))
    );
}
