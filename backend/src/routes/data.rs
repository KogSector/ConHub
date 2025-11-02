use actix_web::{web, HttpResponse, Result, HttpRequest};
use crate::state::AppState;
use serde_json::Value;

const DATA_SERVICE_URL: &str = "http://localhost:3013";

pub async fn connect_source(
    req: HttpRequest,
    body: web::Json<Value>,
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let client = reqwest::Client::new();
    
    match client
        .post(&format!("{}/api/data/sources", DATA_SERVICE_URL))
        .json(&body.into_inner())
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            match response.json::<Value>().await {
                Ok(json_body) => Ok(HttpResponse::build(status).json(json_body)),
                Err(_) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to parse response from data service"
                })))
            }
        }
        Err(e) => {
            log::error!("Failed to connect to data service: {}", e);
            Ok(HttpResponse::BadGateway().json(serde_json::json!({
                "error": "Data service unavailable"
            })))
        }
    }
}

pub async fn sync_source(
    path: web::Path<String>,
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let source_id = path.into_inner();
    let client = reqwest::Client::new();
    
    match client
        .post(&format!("{}/api/data/sources/{}/sync", DATA_SERVICE_URL, source_id))
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            match response.json::<Value>().await {
                Ok(json_body) => Ok(HttpResponse::build(status).json(json_body)),
                Err(_) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to parse response from data service"
                })))
            }
        }
        Err(e) => {
            log::error!("Failed to connect to data service: {}", e);
            Ok(HttpResponse::BadGateway().json(serde_json::json!({
                "error": "Data service unavailable"
            })))
        }
    }
}

pub async fn list_sources(
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let client = reqwest::Client::new();
    
    match client
        .get(&format!("{}/api/data/sources", DATA_SERVICE_URL))
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            match response.json::<Value>().await {
                Ok(json_body) => Ok(HttpResponse::build(status).json(json_body)),
                Err(_) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to parse response from data service"
                })))
            }
        }
        Err(e) => {
            log::error!("Failed to connect to data service: {}", e);
            Ok(HttpResponse::BadGateway().json(serde_json::json!({
                "error": "Data service unavailable"
            })))
        }
    }
}

pub async fn connect_source_endpoint(
    body: web::Json<Value>,
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let client = reqwest::Client::new();
    
    match client
        .post(&format!("{}/api/data/sources", DATA_SERVICE_URL))
        .json(&body.into_inner())
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            match response.json::<Value>().await {
                Ok(json_body) => Ok(HttpResponse::build(status).json(json_body)),
                Err(_) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to parse response from data service"
                })))
            }
        }
        Err(e) => {
            log::error!("Failed to connect to data service: {}", e);
            Ok(HttpResponse::BadGateway().json(serde_json::json!({
                "error": "Data service unavailable"
            })))
        }
    }
}

pub async fn delete_source(
    path: web::Path<String>,
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let source_id = path.into_inner();
    let client = reqwest::Client::new();
    
    match client
        .delete(&format!("{}/api/data/sources/{}", DATA_SERVICE_URL, source_id))
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            match response.json::<Value>().await {
                Ok(json_body) => Ok(HttpResponse::build(status).json(json_body)),
                Err(_) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to parse response from data service"
                })))
            }
        }
        Err(e) => {
            log::error!("Failed to connect to data service: {}", e);
            Ok(HttpResponse::BadGateway().json(serde_json::json!({
                "error": "Data service unavailable"
            })))
        }
    }
}

pub fn configure_data_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/data")
            .route("/sources", web::post().to(connect_source))
            .route("/sources", web::get().to(list_sources))
            .route("/sources/{id}/sync", web::post().to(sync_source))
    )
    .service(
        web::scope("/data-sources")
            .route("", web::get().to(list_sources))
            .route("/connect", web::post().to(connect_source_endpoint))
            .route("/{id}/sync", web::post().to(sync_source))
            .route("/{id}", web::delete().to(delete_source))
    );
}
