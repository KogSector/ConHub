use actix_web::{web, HttpResponse, Result};
use serde_json::json;

use conhub_models::{
    ApiResponse, ConnectRepositoryRequest,
    RepositoryConfig, RepositoryCredentials
};
use crate::services::{RepositoryService, CredentialValidator};
use crate::services::vcs_connector::VcsError;


pub async fn connect_repository(
    req: web::Json<ConnectRepositoryRequest>,
) -> Result<HttpResponse> {
    let repository_service = RepositoryService::new();
    
    match repository_service.connect_repository(req.into_inner()).await {
        Ok(repo_info) => Ok(HttpResponse::Created().json(ApiResponse {
            success: true,
            message: "Repository connected successfully".to_string(),
            data: Some(repo_info),
            error: None,
        })),
        Err(e) => {
            let (mut status, error_msg) = match e {
                VcsError::AuthenticationFailed(msg) => (HttpResponse::Unauthorized(), msg),
                VcsError::RepositoryNotFound(msg) => (HttpResponse::NotFound(), msg),
                VcsError::InvalidCredentials(msg) => (HttpResponse::BadRequest(), msg),
                VcsError::InvalidUrl(msg) => (HttpResponse::BadRequest(), msg),
                VcsError::PermissionDenied(msg) => (HttpResponse::Forbidden(), msg),
                _ => (HttpResponse::InternalServerError(), e.to_string()),
            };
            
            Ok(status.json(ApiResponse::<()> {
                success: false,
                message: "Failed to connect repository".to_string(),
                data: None,
                error: Some(error_msg),
            }))
        }
    }
}


pub async fn get_repository(path: web::Path<String>) -> Result<HttpResponse> {
    let repo_id = path.into_inner();
    let repository_service = RepositoryService::new();
    
    match repository_service.get_repository(&repo_id) {
        Some(repo_info) => Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: "Repository retrieved successfully".to_string(),
            data: Some(repo_info),
            error: None,
        })),
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            message: "Repository not found".to_string(),
            data: None,
            error: Some("Repository with the specified ID does not exist".to_string()),
        })),
    }
}


pub async fn list_repositories() -> Result<HttpResponse> {
    let repository_service = RepositoryService::new();
    let repositories = repository_service.list_repositories();
    
    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: format!("Retrieved {} repositories", repositories.len()),
        data: Some(repositories),
        error: None,
    }))
}


pub async fn update_repository_config(
    path: web::Path<String>,
    req: web::Json<RepositoryConfig>,
) -> Result<HttpResponse> {
    let repo_id = path.into_inner();
    let repository_service = RepositoryService::new();
    
    match repository_service.update_repository_config(&repo_id, req.into_inner()) {
        Ok(()) => Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: "Repository configuration updated successfully".to_string(),
            data: Some(json!({"updated": true})),
            error: None,
        })),
        Err(VcsError::RepositoryNotFound(msg)) => Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            message: "Repository not found".to_string(),
            data: None,
            error: Some(msg),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            message: "Failed to update repository configuration".to_string(),
            data: None,
            error: Some(e.to_string()),
        })),
    }
}


pub async fn test_repository_connection(path: web::Path<String>) -> Result<HttpResponse> {
    let repo_id = path.into_inner();
    let repository_service = RepositoryService::new();
    
    match repository_service.test_repository_connection(&repo_id).await {
        Ok(is_connected) => Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: if is_connected { 
                "Repository connection is healthy".to_string() 
            } else { 
                "Repository connection failed".to_string() 
            },
            data: Some(json!({ "connected": is_connected })),
            error: None,
        })),
        Err(VcsError::RepositoryNotFound(msg)) => Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            message: "Repository not found".to_string(),
            data: None,
            error: Some(msg),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            message: "Failed to test repository connection".to_string(),
            data: None,
            error: Some(e.to_string()),
        })),
    }
}


pub async fn sync_repository(path: web::Path<String>) -> Result<HttpResponse> {
    let repo_id = path.into_inner();
    let repository_service = RepositoryService::new();
    
    match repository_service.sync_repository(&repo_id).await {
        Ok(()) => Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: "Repository synchronized successfully".to_string(),
            data: Some(json!({ "synced": true })),
            error: None,
        })),
        Err(VcsError::RepositoryNotFound(msg)) => Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            message: "Repository not found".to_string(),
            data: None,
            error: Some(msg),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            message: "Failed to sync repository".to_string(),
            data: None,
            error: Some(e.to_string()),
        })),
    }
}


pub async fn disconnect_repository(path: web::Path<String>) -> Result<HttpResponse> {
    let repo_id = path.into_inner();
    let repository_service = RepositoryService::new();
    
    match repository_service.disconnect_repository(&repo_id) {
        Ok(()) => Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: "Repository disconnected successfully".to_string(),
            data: Some(json!({ "disconnected": true })),
            error: None,
        })),
        Err(VcsError::RepositoryNotFound(msg)) => Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            message: "Repository not found".to_string(),
            data: None,
            error: Some(msg),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            message: "Failed to disconnect repository".to_string(),
            data: None,
            error: Some(e.to_string()),
        })),
    }
}


pub async fn list_repository_branches(path: web::Path<String>) -> Result<HttpResponse> {
    let repo_id = path.into_inner();
    let repository_service = RepositoryService::new();
    
    match repository_service.list_repository_branches(&repo_id).await {
        Ok(branches) => Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: format!("Retrieved {} branches", branches.len()),
            data: Some(branches),
            error: None,
        })),
        Err(VcsError::RepositoryNotFound(msg)) => Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            message: "Repository not found".to_string(),
            data: None,
            error: Some(msg),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            message: "Failed to list repository branches".to_string(),
            data: None,
            error: Some(e.to_string()),
        })),
    }
}


pub async fn update_repository_credentials(
    path: web::Path<String>,
    req: web::Json<RepositoryCredentials>,
) -> Result<HttpResponse> {
    let repo_id = path.into_inner();
    let repository_service = RepositoryService::new();
    
    match repository_service.update_repository_credentials(&repo_id, req.into_inner()) {
        Ok(()) => Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: "Repository credentials updated successfully".to_string(),
            data: Some(json!({ "updated": true })),
            error: None,
        })),
        Err(VcsError::RepositoryNotFound(msg)) => Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            message: "Repository not found".to_string(),
            data: None,
            error: Some(msg),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            message: "Failed to update repository credentials".to_string(),
            data: None,
            error: Some(e.to_string()),
        })),
    }
}


#[derive(serde::Deserialize)]
pub struct WebhookRequest {
    pub webhook_url: String,
}

pub async fn setup_repository_webhook(
    path: web::Path<String>,
    req: web::Json<WebhookRequest>,
) -> Result<HttpResponse> {
    let repo_id = path.into_inner();
    let repository_service = RepositoryService::new();
    
    match repository_service.setup_repository_webhook(&repo_id, &req.webhook_url).await {
        Ok(webhook_id) => Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: "Webhook setup successfully".to_string(),
            data: Some(json!({ "webhook_id": webhook_id })),
            error: None,
        })),
        Err(VcsError::RepositoryNotFound(msg)) => Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            message: "Repository not found".to_string(),
            data: None,
            error: Some(msg),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            message: "Failed to setup webhook".to_string(),
            data: None,
            error: Some(e.to_string()),
        })),
    }
}


#[derive(serde::Deserialize)]
pub struct ValidateCredentialsRequest {
    pub vcs_type: crate::models::VcsType,
    pub provider: crate::models::VcsProvider,
    pub credentials: RepositoryCredentials,
}

pub async fn validate_credentials(
    req: web::Json<ValidateCredentialsRequest>,
) -> Result<HttpResponse> {
    match CredentialValidator::validate_credentials(
        &req.vcs_type,
        &req.provider,
        &req.credentials,
    ).await {
        Ok(is_valid) => Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: if is_valid { 
                "Credentials are valid".to_string() 
            } else { 
                "Credentials are invalid".to_string() 
            },
            data: Some(json!({ "valid": is_valid })),
            error: None,
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            message: "Failed to validate credentials".to_string(),
            data: None,
            error: Some(e.to_string()),
        })),
    }
}


pub async fn get_repository_stats() -> Result<HttpResponse> {
    let repository_service = RepositoryService::new();
    let repositories = repository_service.list_repositories();
    
    let mut stats = std::collections::HashMap::new();
    stats.insert("total_repositories", repositories.len());
    
    let connected_count = repositories.iter()
        .filter(|r| r.sync_status == crate::models::RepositorySyncStatus::Connected)
        .count();
    stats.insert("connected", connected_count);
    
    let syncing_count = repositories.iter()
        .filter(|r| r.sync_status == crate::models::RepositorySyncStatus::Syncing)
        .count();
    stats.insert("syncing", syncing_count);
    
    let mut vcs_types = std::collections::HashMap::new();
    for repo in &repositories {
        *vcs_types.entry(format!("{:?}", repo.vcs_type)).or_insert(0) += 1;
    }
    
    let mut providers = std::collections::HashMap::new();
    for repo in &repositories {
        *providers.entry(format!("{:?}", repo.provider)).or_insert(0) += 1;
    }
    
    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "Repository statistics retrieved successfully".to_string(),
        data: Some(json!({
            "counts": stats,
            "vcs_types": vcs_types,
            "providers": providers
        })),
        error: None,
    }))
}


#[derive(serde::Deserialize)]
pub struct ValidateUrlRequest {
    pub repo_url: String,
}

#[derive(serde::Serialize)]
pub struct ValidateUrlResponse {
    pub valid: bool,
    pub provider: Option<String>,
    pub vcs_type: Option<String>,
    pub owner: Option<String>,
    pub repo_name: Option<String>,
    pub is_public: bool,
}

pub async fn validate_repository_url(
    req: web::Json<ValidateUrlRequest>,
) -> Result<HttpResponse> {
    use crate::services::vcs_detector::VcsDetector;
    
    let url = &req.repo_url;
    
    
    if url.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            message: "Repository URL is required".to_string(),
            data: None,
            error: Some("URL cannot be empty".to_string()),
        }));
    }
    
    
    let (vcs_type, provider) = match VcsDetector::detect_from_url(url) {
        Ok(result) => result,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                message: "Invalid repository URL".to_string(),
                data: None,
                error: Some(e),
            }));
        }
    };
    
    
    let (owner, repo_name) = match VcsDetector::extract_repo_info(url) {
        Ok((o, r)) => (Some(o), Some(r)),
        Err(_) => (None, None),
    };
    
    
    let is_public = true;
    
    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "Repository URL validated successfully".to_string(),
        data: Some(ValidateUrlResponse {
            valid: true,
            provider: Some(format!("{:?}", provider)),
            vcs_type: Some(format!("{:?}", vcs_type)),
            owner,
            repo_name,
            is_public,
        }),
        error: None,
    }))
}


#[derive(serde::Deserialize)]
pub struct FetchBranchesRequest {
    pub repo_url: String,
    pub credentials: Option<RepositoryCredentials>,
}

#[derive(serde::Serialize)]
pub struct FetchBranchesResponse {
    pub branches: Vec<String>,
    pub default_branch: Option<String>,
}

pub async fn fetch_branches(
    req: web::Json<FetchBranchesRequest>,
) -> Result<HttpResponse> {
    use crate::services::vcs_connector::VcsConnectorFactory;
    use crate::services::vcs_detector::VcsDetector;
    
    let (vcs_type, _provider) = match VcsDetector::detect_from_url(&req.repo_url) {
        Ok(result) => result,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                message: "Invalid repository URL".to_string(),
                data: None,
                error: Some(e),
            }));
        }
    };
    
    let connector = VcsConnectorFactory::create_connector(&vcs_type);
    
    
    let credentials = req.credentials.as_ref().unwrap_or(&RepositoryCredentials {
        credential_type: crate::models::CredentialType::None,
        expires_at: None,
    });
    
    match connector.list_branches(&req.repo_url, credentials).await {
        Ok(branch_info) => {
            let branches: Vec<String> = branch_info.iter().map(|b| b.name.clone()).collect();
            let default_branch = branch_info.iter()
                .find(|b| b.is_default)
                .map(|b| b.name.clone())
                .or_else(|| {
                    
                    if branches.contains(&"main".to_string()) {
                        Some("main".to_string())
                    } else if branches.contains(&"master".to_string()) {
                        Some("master".to_string())
                    } else {
                        branches.first().cloned()
                    }
                });
            
            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: format!("Retrieved {} branches", branches.len()),
                data: Some(FetchBranchesResponse {
                    branches,
                    default_branch,
                }),
                error: None,
            }))
        }
        Err(e) => {
            let (mut status, error_msg) = match e {
                crate::services::vcs_connector::VcsError::AuthenticationFailed(msg) => (HttpResponse::Unauthorized(), msg),
                crate::services::vcs_connector::VcsError::RepositoryNotFound(msg) => (HttpResponse::NotFound(), msg),
                crate::services::vcs_connector::VcsError::InvalidCredentials(msg) => (HttpResponse::BadRequest(), msg),
                crate::services::vcs_connector::VcsError::InvalidUrl(msg) => (HttpResponse::BadRequest(), msg),
                crate::services::vcs_connector::VcsError::PermissionDenied(msg) => (HttpResponse::Forbidden(), msg),
                _ => (HttpResponse::InternalServerError(), e.to_string()),
            };
            
            Ok(status.json(ApiResponse::<()> {
                success: false,
                message: "Failed to fetch branches".to_string(),
                data: None,
                error: Some(error_msg),
            }))
        }
    }
}


pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/repositories")
            .route("/connect", web::post().to(connect_repository))
            .route("/validate-url", web::post().to(validate_repository_url))
            .route("/fetch-branches", web::post().to(fetch_branches))
            .route("/stats", web::get().to(get_repository_stats))
            .route("/validate-credentials", web::post().to(validate_credentials))
            .route("", web::get().to(list_repositories))
            .route("/{id}", web::get().to(get_repository))
            .route("/{id}/config", web::put().to(update_repository_config))
            .route("/{id}/test", web::post().to(test_repository_connection))
            .route("/{id}/sync", web::post().to(sync_repository))
            .route("/{id}/disconnect", web::delete().to(disconnect_repository))
            .route("/{id}/branches", web::get().to(list_repository_branches))
            .route("/{id}/credentials", web::put().to(update_repository_credentials))
            .route("/{id}/webhook", web::post().to(setup_repository_webhook))
    );
}