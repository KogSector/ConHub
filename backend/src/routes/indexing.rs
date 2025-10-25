use actix_web::{web, HttpResponse, Result};
use crate::state::AppState;
use crate::models::indexing_dto::{IndexRepositoryRequest, IndexDocumentationRequest, SearchRequest, IndexingResponse, SearchResponse};
use validator::Validate;

pub async fn index_repository(
    state: web::Data<AppState>,
    body: web::Json<IndexRepositoryRequest>,
) -> Result<HttpResponse> {
    // Validate request
    body.validate()
        .map_err(|e| actix_web::error::ErrorBadRequest(e))?;

    // Call indexing service (which calls indexers library directly - NO HTTP)
    match state.indexing_service.index_repository(
        &body.repository_url,
        body.branch.as_deref(),
        body.include_patterns.as_ref(),
        body.exclude_patterns.as_ref(),
    ).await {
        Ok(result) => {
            let response = IndexingResponse {
                job_id: result.job_id,
                status: result.status,
                message: result.message,
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            log::error!("Repository indexing failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("{}", e)
            })))
        }
    }
}

pub async fn index_documentation(
    state: web::Data<AppState>,
    body: web::Json<IndexDocumentationRequest>,
) -> Result<HttpResponse> {
    // Validate request
    body.validate()
        .map_err(|e| actix_web::error::ErrorBadRequest(e))?;

    // Call indexing service
    match state.indexing_service.index_documentation(&body.url).await {
        Ok(result) => {
            let response = IndexingResponse {
                job_id: result.job_id,
                status: result.status,
                message: result.message,
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            log::error!("Documentation indexing failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("{}", e)
            })))
        }
    }
}

pub async fn search(
    state: web::Data<AppState>,
    body: web::Json<SearchRequest>,
) -> Result<HttpResponse> {
    // Validate request
    body.validate()
        .map_err(|e| actix_web::error::ErrorBadRequest(e))?;

    // Call indexing service
    match state.indexing_service.search(&body.query).await {
        Ok(results) => {
            let response = SearchResponse {
                results: results.into_iter().map(|content| crate::models::indexing_dto::SearchResult {
                    content,
                    score: 0.0,
                    metadata: serde_json::json!({}),
                }).collect(),
                total: 0,
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            log::error!("Search failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("{}", e)
            })))
        }
    }
}

pub async fn get_status(
    _state: web::Data<AppState>,
    _job_id: web::Path<String>,
) -> Result<HttpResponse> {
    // TODO: Implement status tracking
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Not implemented yet"
    })))
}

pub fn configure_indexing_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/index")
            .route("/repository", web::post().to(index_repository))
            .route("/documentation", web::post().to(index_documentation))
    )
    .service(
        web::scope("/")
            .route("/search", web::post().to(search))
            .route("/index/status/{job_id}", web::get().to(get_status))
    );
}
