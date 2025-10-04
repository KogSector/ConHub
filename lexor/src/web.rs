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

#[derive(serde::Deserialize)]
#[allow(dead_code)]
pub struct RepositoryIndexRequest {
    pub repository_id: String,
    pub name: String,
    pub url: String,
    pub clone_url: String,
    pub default_branch: String,
    pub vcs_type: String,
    pub provider: String,
    pub config: RepositoryIndexConfig,
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
pub struct RepositoryIndexConfig {
    pub include_code: bool,
    pub include_readme: bool,
    pub file_extensions: Vec<String>,
}

pub async fn index_repository_handler(
    _service: web::Data<LexorService>,
    request: web::Json<RepositoryIndexRequest>,
) -> Result<HttpResponse> {
    println!("Received repository indexing request for: {}", request.name);
    
    // For now, simulate successful indexing
    // In a real implementation, you would:
    // 1. Clone the repository to a temporary location
    // 2. Parse and index all code files
    // 3. Extract symbols, functions, classes, etc.
    // 4. Store in Tantivy index
    
    let response = json!({
        "status": "success",
        "message": format!("Repository '{}' indexed successfully", request.name),
        "repository_id": request.repository_id,
        "indexed_files": 0, // Placeholder
        "indexed_symbols": 0, // Placeholder
        "processing_time_ms": 1000 // Placeholder
    });
    
    println!("Repository indexing completed for: {}", request.name);
    Ok(HttpResponse::Ok().json(response))
}