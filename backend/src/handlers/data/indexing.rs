use actix_web::{web, HttpResponse, Result};
use serde_json::json;
use validator::Validate;
use reqwest::Client;
use std::env;

#[derive(serde::Deserialize, Validate)]
pub struct IndexRepositoryRequest {
    #[validate(url)]
    pub repository_url: String,
    pub branch: Option<String>,
    pub include_patterns: Option<Vec<String>>,
    pub exclude_patterns: Option<Vec<String>>,
    pub language_filters: Option<Vec<String>>,
    pub max_file_size: Option<usize>,
}

#[derive(serde::Deserialize, Validate)]
pub struct IndexDocumentationRequest {
    #[validate(url)]
    pub documentation_url: String,
    pub doc_type: Option<String>,
    pub crawl_depth: Option<u32>,
    pub follow_links: Option<bool>,
    pub extract_code_blocks: Option<bool>,
}

#[derive(serde::Deserialize, Validate)]
pub struct IndexUrlRequest {
    #[validate(url)]
    pub url: String,
    pub content_type: Option<String>,
    pub extract_links: Option<bool>,
}

#[derive(serde::Deserialize, Validate)]
pub struct IndexFileRequest {
    pub file_path: String,
    pub file_type: Option<String>,
}


pub async fn index_repository(request: web::Json<IndexRepositoryRequest>) -> Result<HttpResponse> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    let indexer_url = env::var("UNIFIED_INDEXER_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    let client = Client::new();
    let response = client
        .post(&format!("{}/api/index/repository", indexer_url))
        .json(&*request)
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            let result = resp.json::<serde_json::Value>().await
                .unwrap_or_else(|_| json!({"success": true, "message": "Indexing started"}));
            log::info!("Repository indexing request forwarded successfully");
            Ok(HttpResponse::Ok().json(result))
        }
        Ok(resp) => {
            let error = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            log::error!("Indexer returned error: {}", error);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Repository indexing failed",
                "message": error
            })))
        }
        Err(e) => {
            log::error!("Failed to connect to indexer: {}", e);
            Ok(HttpResponse::ServiceUnavailable().json(json!({
                "error": "Indexer service unavailable",
                "message": e.to_string()
            })))
        }
    }
}


pub async fn index_documentation(request: web::Json<IndexDocumentationRequest>) -> Result<HttpResponse> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    let indexer_url = env::var("UNIFIED_INDEXER_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    let client = Client::new();
    match client
        .post(&format!("{}/api/index/documentation", indexer_url))
        .json(&*request)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            let result = resp.json::<serde_json::Value>().await
                .unwrap_or_else(|_| json!({"success": true, "message": "Indexing started"}));
            Ok(HttpResponse::Ok().json(result))
        }
        Ok(resp) => {
            let error = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Documentation indexing failed",
                "message": error
            })))
        }
        Err(e) => {
            Ok(HttpResponse::ServiceUnavailable().json(json!({
                "error": "Indexer service unavailable",
                "message": e.to_string()
            })))
        }
    }
}


pub async fn index_url(request: web::Json<IndexUrlRequest>) -> Result<HttpResponse> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    let indexer_url = env::var("UNIFIED_INDEXER_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    let client = Client::new();
    match client
        .post(&format!("{}/api/index/url", indexer_url))
        .json(&*request)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            let result = resp.json::<serde_json::Value>().await
                .unwrap_or_else(|_| json!({"success": true, "message": "Indexing started"}));
            Ok(HttpResponse::Ok().json(result))
        }
        Ok(resp) => {
            let error = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "URL indexing failed",
                "message": error
            })))
        }
        Err(e) => {
            Ok(HttpResponse::ServiceUnavailable().json(json!({
                "error": "Indexer service unavailable",
                "message": e.to_string()
            })))
        }
    }
}


pub async fn index_file(request: web::Json<IndexFileRequest>) -> Result<HttpResponse> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    let indexer_url = env::var("UNIFIED_INDEXER_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    let client = Client::new();
    match client
        .post(&format!("{}/api/index/file", indexer_url))
        .json(&*request)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            let result = resp.json::<serde_json::Value>().await
                .unwrap_or_else(|_| json!({"success": true, "message": "Indexing started"}));
            Ok(HttpResponse::Ok().json(result))
        }
        Ok(resp) => {
            let error = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "File indexing failed",
                "message": error
            })))
        }
        Err(e) => {
            Ok(HttpResponse::ServiceUnavailable().json(json!({
                "error": "Indexer service unavailable",
                "message": e.to_string()
            })))
        }
    }
}


pub async fn get_indexing_status() -> Result<HttpResponse> {
    let indexer_url = env::var("UNIFIED_INDEXER_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    let client = Client::new();
    match client
        .get(&format!("{}/api/status", indexer_url))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            let result = resp.json::<serde_json::Value>().await
                .unwrap_or_else(|_| json!({"status": "operational"}));
            Ok(HttpResponse::Ok().json(result))
        }
        _ => {
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "status": "operational",
                "message": "Indexer service status unavailable"
            })))
        }
    }
}

pub fn configure_indexing_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/indexing")
            .route("/repository", web::post().to(index_repository))
            .route("/documentation", web::post().to(index_documentation))
            .route("/url", web::post().to(index_url))
            .route("/file", web::post().to(index_file))
            .route("/status", web::get().to(get_indexing_status))
    );
}
