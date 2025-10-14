use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::models::ApiResponse;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUrlRequest {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UrlResponse {
    pub id: String,
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub created_at: String,
}

pub async fn create_url(req: web::Json<CreateUrlRequest>) -> Result<HttpResponse> {
    let url_response = UrlResponse {
        id: uuid::Uuid::new_v4().to_string(),
        url: req.url.clone(),
        title: req.title.clone(),
        description: req.description.clone(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "URL created successfully".to_string(),
        data: Some(url_response),
        error: None,
    }))
}

pub async fn get_urls() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "urls": []
    })))
}

pub async fn delete_url(path: web::Path<String>) -> Result<HttpResponse> {
    let _id = path.into_inner();
    
    Ok(HttpResponse::Ok().json(ApiResponse::<()> {
        success: true,
        message: "URL deleted successfully".to_string(),
        data: None,
        error: None,
    }))
}

pub async fn get_url_analytics() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "total_urls": 0,
        "active_urls": 0,
        "analytics": []
    })))
}