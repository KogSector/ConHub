use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{error, info};

use crate::sources::DataSourceFactory;

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
) -> Result<HttpResponse, ServiceError> {
    info!("Connecting data source: {}", req.source_type);

    match DataSourceFactory::create_connector(&req.source_type) {
        Ok(mut connector) => {
            match connector.connect(&req.credentials, &req.config).await {
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
                    Err(ServiceError::BadRequest(format!("Failed to connect data source: {}", e)))
                }
            }
        }
        Err(e) => {
            error!("Unsupported data source type: {}", req.source_type);
            Err(ServiceError::BadRequest(format!("Unsupported data source type: {}", e)))
        }
    }
}

pub async fn fetch_branches(
    req: web::Json<FetchBranchesRequest>,
) -> Result<HttpResponse, ServiceError> {
    info!("Fetching branches for repository: {}", req.repo_url);

    
    let source_type = if req.repo_url.contains("github.com") {
        "github"
    } else if req.repo_url.contains("bitbucket.org") {
        "bitbucket"
    } else {
        return Err(ServiceError::BadRequest("Unsupported repository URL format".to_string()));
    };

    
    let credentials = req.credentials.clone().unwrap_or_default();

    match DataSourceFactory::create_connector(source_type) {
        Ok(mut connector) => {
            if let Err(e) = connector.connect(&credentials, &serde_json::Value::Null).await {
                return Err(ServiceError::BadRequest(format!("Failed to connect: {}", e)));
            }
            
            match connector.fetch_branches(&req.repo_url).await {
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
                    Err(ServiceError::BadRequest(format!("Failed to fetch branches: {}", e)))
                }
            }
        }
        Err(e) => {
            error!("Unsupported repository type: {}", source_type);
            Err(ServiceError::BadRequest(format!("Unsupported repository type: {}", e)))
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