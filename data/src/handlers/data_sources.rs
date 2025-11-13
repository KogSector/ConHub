use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{error, info};

use crate::sources::DataSourceFactory;
use crate::errors::ServiceError;
use conhub_models::{RepositoryCredentials, CredentialType};
use crate::services::data::vcs_connector::{VcsConnectorFactory, VcsConnector};
use crate::services::data::vcs_detector::VcsDetector;

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
    pub file_extensions: Option<Vec<String>>,
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

    // Detect VCS type from URL
    let (vcs_type, _provider) = match VcsDetector::detect_from_url(&req.repo_url) {
        Ok(result) => result,
        Err(e) => {
            return Err(ServiceError::BadRequest(format!("Invalid repository URL: {}", e)));
        }
    };

    // Convert frontend credentials to backend format
    let credentials = match req.credentials.as_ref() {
        Some(creds) => {
            if let Some(access_token) = creds.get("accessToken") {
                RepositoryCredentials {
                    credential_type: CredentialType::PersonalAccessToken {
                        token: access_token.clone(),
                    },
                    expires_at: None,
                }
            } else if let (Some(username), Some(app_password)) = (creds.get("username"), creds.get("appPassword")) {
                RepositoryCredentials {
                    credential_type: CredentialType::AppPassword {
                        username: username.clone(),
                        app_password: app_password.clone(),
                    },
                    expires_at: None,
                }
            } else {
                RepositoryCredentials {
                    credential_type: CredentialType::None,
                    expires_at: None,
                }
            }
        }
        None => RepositoryCredentials {
            credential_type: CredentialType::None,
            expires_at: None,
        },
    };

    // Use VCS connector directly
    let connector = VcsConnectorFactory::create_connector(&vcs_type);
    
    match connector.list_branches(&req.repo_url, &credentials).await {
        Ok(branch_info) => {
            let branches: Vec<String> = branch_info.iter().map(|b| b.name.clone()).collect();
            let default_branch = branch_info.iter()
                .find(|b| b.is_default)
                .map(|b| b.name.clone())
                .or_else(|| {
                    // Fallback logic for default branch detection
                    if branches.contains(&"main".to_string()) {
                        Some("main".to_string())
                    } else if branches.contains(&"master".to_string()) {
                        Some("master".to_string())
                    } else {
                        branches.first().cloned()
                    }
                });

            // Fetch file extensions from the default branch
            let file_extensions = if let Some(ref branch) = default_branch {
                match connector.get_file_extensions(&req.repo_url, branch, &credentials).await {
                    Ok(extensions) => Some(extensions),
                    Err(e) => {
                        info!("Could not fetch file extensions: {}", e);
                        None
                    }
                }
            } else {
                None
            };

            info!("Successfully fetched {} branches for repository: {}", branches.len(), req.repo_url);
            Ok(HttpResponse::Ok().json(FetchBranchesResponse {
                branches,
                default_branch,
                file_extensions,
                error: None,
            }))
        }
        Err(e) => {
            error!("Failed to fetch branches for repository {}: {}", req.repo_url, e);
            let error_msg = match e {
                crate::services::data::vcs_connector::VcsError::AuthenticationFailed(msg) => {
                    format!("Authentication failed: {}", msg)
                }
                crate::services::data::vcs_connector::VcsError::RepositoryNotFound(msg) => {
                    format!("Repository not found: {}", msg)
                }
                crate::services::data::vcs_connector::VcsError::PermissionDenied(msg) => {
                    format!("Permission denied: {}", msg)
                }
                _ => format!("Failed to fetch branches: {}", e),
            };
            Err(ServiceError::BadRequest(error_msg))
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