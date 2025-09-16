use actix_web::{web, HttpResponse, Result};
use crate::services::data_source_proxy::{connect_data_source, list_data_sources, DataSourceRequest};
use reqwest::Client;

pub async fn connect(req: web::Json<DataSourceRequest>) -> Result<HttpResponse> {
    let client = Client::new();
    let langchain_url = std::env::var("LANGCHAIN_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:3003".to_string());
    
    match connect_data_source(&client, &langchain_url, &req).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(error) => Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": error.to_string()
        })))
    }
}

pub async fn list() -> Result<HttpResponse> {
    let client = Client::new();
    let langchain_url = std::env::var("LANGCHAIN_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:3003".to_string());
    
    match list_data_sources(&client, &langchain_url).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(error) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "message": error.to_string()
        })))
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/data-sources")
            .route("/connect", web::post().to(connect))
            .route("", web::get().to(list))
    );
}