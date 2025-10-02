use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{error, info};

use crate::services::data_source_service::DataSourceService;

#[derive(Debug, Deserialize)]
pub struct ConnectDataSourceRequest {
    #[serde(rename = "type")]
    pub source_type: String,
    pub credentials: HashMap<String, String>,
    pub config: Value,
}

#[derive(Debug, Deserialize)]
pub struct FetchBranchesRequest {
    #[serde(rename = "repoUrl")]
    pub repo_url: String,
    pub credentials: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct ConnectResponse {
    pub success: bool,
    pub message: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FetchBranchesResponse {
    pub branches: Vec<String>,
    #[serde(rename = "defaultBranch")]
    pub default_branch: Option<String>,
    pub error: Option<String>,
}

pub async fn connect_data_source(
    req: web::Json<ConnectDataSourceRequest>,
) -> Result<HttpResponse> {
    info!("Connecting data source: {}", req.source_type);

    let service = DataSourceService::new();
    
    match service.connect_data_source(&req.source_type, req.credentials.clone(), req.config.clone()).await {
        Ok(_) => {
            info!("Successfully connected to {} data source", req.source_type);
            Ok(HttpResponse::Ok().json(ConnectResponse {
                success: true,
                message: format!("Successfully connected to {} data source", req.source_type),
                error: None,
            }))
        }
        Err(e) => {
            error!("Failed to connect to {} data source: {}", req.source_type, e);
            Ok(HttpResponse::BadRequest().json(ConnectResponse {
                success: false,
                message: "Failed to connect data source".to_string(),
                error: Some(e.to_string()),
            }))
        }
    }
}

pub async fn fetch_branches(
    req: web::Json<FetchBranchesRequest>,
) -> Result<HttpResponse> {
    info!("Fetching branches for repository: {}", req.repo_url);

    // Determine source type from URL
    let source_type = if req.repo_url.contains("github.com") {
        "github"
    } else if req.repo_url.contains("bitbucket.org") {
        "bitbucket"
    } else {
        return Ok(HttpResponse::BadRequest().json(FetchBranchesResponse {
            branches: vec![],
            default_branch: None,
            error: Some("Unsupported repository URL format".to_string()),
        }));
    };

    // For now, use empty credentials - in a real implementation, you'd get these from the request or session
    let credentials = req.credentials.clone().unwrap_or_default();

    let service = DataSourceService::new();
    
    match service.fetch_branches(source_type, &req.repo_url, credentials).await {
        Ok(branches) => {
            let default_branch = if branches.contains(&"main".to_string()) {
                Some("main".to_string())
            } else if branches.contains(&"master".to_string()) {
                Some("master".to_string())
            } else {
                branches.first().cloned()
            };

            info!("Successfully fetched {} branches for repository: {}", branches.len(), req.repo_url);
            Ok(HttpResponse::Ok().json(FetchBranchesResponse {
                branches,
                default_branch,
                error: None,
            }))
        }
        Err(e) => {
            error!("Failed to fetch branches for repository {}: {}", req.repo_url, e);
            Ok(HttpResponse::BadRequest().json(FetchBranchesResponse {
                branches: vec!["main".to_string()], // Fallback
                default_branch: Some("main".to_string()),
                error: Some(e.to_string()),
            }))
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/data-sources")
            .route("/connect", web::post().to(connect_data_source))
    ).service(
        web::scope("/api/repositories")
            .route("/fetch-branches", web::post().to(fetch_branches))
    );
}