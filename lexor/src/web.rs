use crate::LexorService;
use crate::types::*;
use actix_web::{web, HttpResponse, Result};
use serde_json::json;
use uuid::Uuid;

pub async fn search_handler(
    service: web::Data<LexorService>,
    query: web::Json<SearchQuery>,
) -> Result<HttpResponse> {
    match service.search_engine.search(&query) {
        Ok(results) => Ok(HttpResponse::Ok().json(results)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        }))),
    }
}

pub async fn add_project_handler(
    service: web::Data<LexorService>,
    project: web::Json<ProjectRequest>,
) -> Result<HttpResponse> {
    match service.indexer.add_project(
        project.name.clone(),
        project.path.clone(),
        project.description.clone(),
    ) {
        Ok(project_id) => Ok(HttpResponse::Ok().json(json!({
            "project_id": project_id,
            "message": "Project added successfully"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        }))),
    }
}

pub async fn index_project_handler(
    service: web::Data<LexorService>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let project_id = Uuid::parse_str(&path).map_err(|e| actix_web::error::ErrorBadRequest(e))?;
    
    match service.indexer.index_project(project_id) {
        Ok(stats) => Ok(HttpResponse::Ok().json(stats)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        }))),
    }
}

pub async fn get_projects_handler(
    service: web::Data<LexorService>,
) -> Result<HttpResponse> {
    let projects = service.indexer.get_projects();
    Ok(HttpResponse::Ok().json(projects))
}

pub async fn get_stats_handler(
    service: web::Data<LexorService>,
) -> Result<HttpResponse> {
    let stats = service.indexer.get_index_stats();
    Ok(HttpResponse::Ok().json(stats))
}

#[derive(serde::Deserialize)]
pub struct ProjectRequest {
    pub name: String,
    pub path: std::path::PathBuf,
    pub description: Option<String>,
}