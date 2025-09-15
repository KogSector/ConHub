use actix_web::{web, HttpResponse, Result};
use serde_json::json;
use crate::config::AppConfig;
use crate::models::{ConnectRepoRequest, SearchRequest, ApiResponse};
use crate::services::{repository, search, ai, datasource};

async fn connect_repository(
    req: web::Json<ConnectRepoRequest>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse> {
    match repository::connect_repository(&config.http_client, &config.langchain_url, &req).await {
        Ok(result) => Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: "Repository connection initiated".to_string(),
            data: Some(result),
            error: None,
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            message: "Failed to connect repository".to_string(),
            data: None,
            error: Some(e.to_string()),
        })),
    }
}

async fn search_universal(
    req: web::Json<SearchRequest>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse> {
    let results = search::universal_search(
        &config.http_client,
        &config.langchain_url,
        &config.haystack_url,
        &config.lexor_url,
        &req,
    ).await;
    
    Ok(HttpResponse::Ok().json(json!({
        "query": req.query,
        "results": results
    })))
}

async fn ask_ai(
    req: web::Json<SearchRequest>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse> {
    match ai::ask_ai_question(&config.http_client, &config.haystack_url, &req).await {
        Ok(result) => Ok(HttpResponse::Ok().json(result)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            message: "AI service unavailable".to_string(),
            data: None,
            error: Some(e.to_string()),
        })),
    }
}

async fn connect_data_source(
    req: web::Json<datasource::DataSourceRequest>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse> {
    match datasource::connect_data_source(&config.http_client, &config.langchain_url, &req).await {
        Ok(result) => Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: "Data source connected".to_string(),
            data: Some(result),
            error: None,
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            message: "Failed to connect data source".to_string(),
            data: None,
            error: Some(e.to_string()),
        })),
    }
}

async fn list_data_sources(
    config: web::Data<AppConfig>,
) -> Result<HttpResponse> {
    match datasource::list_data_sources(&config.http_client, &config.langchain_url).await {
        Ok(result) => Ok(HttpResponse::Ok().json(result)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            message: "Failed to list data sources".to_string(),
            data: None,
            error: Some(e.to_string()),
        })),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/repositories/connect", web::post().to(connect_repository))
            .route("/search/universal", web::post().to(search_universal))
            .route("/ai/ask", web::post().to(ask_ai))
            .route("/data-sources/connect", web::post().to(connect_data_source))
            .route("/data-sources", web::get().to(list_data_sources))
            .route("/urls", web::post().to(crate::handlers::urls::create_url))
            .route("/urls", web::get().to(crate::handlers::urls::get_urls))
            .route("/urls/{id}", web::delete().to(crate::handlers::urls::delete_url))
            .route("/urls/analytics", web::get().to(crate::handlers::urls::get_url_analytics))
            .route("/documents", web::post().to(crate::handlers::documents::create_document))
            .route("/documents", web::get().to(crate::handlers::documents::get_documents))
            .route("/documents/{id}", web::delete().to(crate::handlers::documents::delete_document))
            .route("/documents/analytics", web::get().to(crate::handlers::documents::get_document_analytics))
    );
}