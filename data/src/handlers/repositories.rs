use actix_web::{web, HttpResponse, Result};
use serde_json::json;
use uuid::Uuid;
use sqlx::PgPool;

use conhub_config::{
    ApiResponse, ConnectRepositoryRequest,
    RepositoryConfig, RepositoryCredentials,
    VcsType, VcsProvider, RepositorySyncStatus, CredentialType
};
use crate::services::data::{RepositoryService, CredentialValidator};
use crate::services::data::vcs_connector::VcsError;
use crate::services::data::vcs_connector::{VcsConnectorFactory, VcsConnector, RepositoryMetadata};
use crate::services::data::vcs_detector::VcsDetector;
use conhub_config::auth::Claims;


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
    
    match repository_service.get_repository(&repo_id).await {
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
    let repositories = repository_service.list_repositories().await;
    
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
    
    match repository_service.update_repository_config(&repo_id, req.into_inner()).await {
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
    if let Some(repo) = repository_service.get_repository(&repo_id).await {
        use crate::services::data::vcs_connector::{VcsConnectorFactory, VcsConnector};
        let connector = VcsConnectorFactory::create_connector(&repo.vcs_type);
        let branch = repo.config.branch.clone();
        let paths = match connector.list_files(&repo.url, "", &branch, &repo.credentials, true).await {
            Ok(p) => p,
            Err(e) => {
                return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                    success: false,
                    message: "Failed to list repository files".to_string(),
                    data: None,
                    error: Some(e.to_string()),
                }))
            }
        };
        let allowed_exts = repo.config.include_file_extensions.clone();
        let max_size_mb = repo.config.max_file_size_mb as i64;
        let mut documents: Vec<crate::connectors::DocumentForEmbedding> = Vec::new();
        for doc_id in paths {
            let ext_ok = allowed_exts.is_empty() || allowed_exts.iter().any(|ext| doc_id.ends_with(ext));
            if !ext_ok { continue; }
            // extract relative file path from owner/repo/path
            let rel_path = doc_id.splitn(3, '/').skip(2).next().unwrap_or("");
            match connector.get_file_content(&repo.url, rel_path, &branch, &repo.credentials).await {
                Ok(content) => {
                    if content.size > (max_size_mb as u64) * 1024 * 1024 { continue; }
                    let text = content.content;
                    // Simple chunking (reuse size similar to Local/GitHub connector)
                    let chunks = {
                        const CHUNK: usize = 1000; const OVERLAP: usize = 200;
                        let mut out = Vec::new(); let bytes = text.as_bytes(); let mut start = 0usize; let mut idx=0usize;
                        while start < bytes.len() { let end = (start+CHUNK).min(bytes.len()); let s = String::from_utf8_lossy(&bytes[start..end]).to_string(); if !s.trim().is_empty(){ out.push(crate::connectors::DocumentChunk{chunk_number:idx,content:s,start_offset:start,end_offset:end,metadata:None}); idx+=1;} if end>=bytes.len(){break;} start = end.saturating_sub(OVERLAP);} out
                    };
                    documents.push(crate::connectors::DocumentForEmbedding{
                        id: uuid::Uuid::new_v4(),
                        source_id: uuid::Uuid::parse_str(&repo.id).unwrap_or_else(|_| uuid::Uuid::new_v4()),
                        connector_type: crate::connectors::ConnectorType::GitHub,
                        external_id: content.sha.clone(),
                        name: rel_path.split('/').last().unwrap_or("").to_string(),
                        path: Some(rel_path.to_string()),
                        content: text,
                        content_type: crate::connectors::ContentType::Code,
                        metadata: serde_json::json!({"url": content.url, "size": content.size, "branch": branch}),
                        chunks: Some(chunks),
                    });
                }
                Err(_) => { /* skip failed file */ }
            }
        }
        let embedding_url = std::env::var("EMBEDDING_SERVICE_URL").unwrap_or_else(|_| "http://localhost:8082".to_string());
        let heavy = conhub_config::feature_toggles::FeatureToggles::from_env_path().is_enabled("Heavy");
        let client = crate::services::EmbeddingClient::new(embedding_url, heavy);
        if !documents.is_empty() { let _ = client.embed_documents(documents).await; }
        Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: "Repository synchronized successfully".to_string(),
            data: Some(json!({ "synced": true })),
            error: None,
        }))
    } else {
        Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            message: "Repository not found".to_string(),
            data: None,
            error: Some("Invalid repository ID".to_string()),
        }))
    }
}


pub async fn disconnect_repository(path: web::Path<String>) -> Result<HttpResponse> {
    let repo_id = path.into_inner();
    let repository_service = RepositoryService::new();
    
    match repository_service.disconnect_repository(&repo_id).await {
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
    
    match repository_service.update_repository_credentials(&repo_id, req.into_inner()).await {
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
    pub vcs_type: VcsType,
    pub provider: VcsProvider,
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
    let repositories = repository_service.list_repositories().await;
    
    let mut stats = std::collections::HashMap::new();
    stats.insert("total_repositories", repositories.len());
    
    let connected_count = repositories.iter()
        .filter(|r| r.sync_status == RepositorySyncStatus::Connected)
        .count();
    stats.insert("connected", connected_count);
    
    let syncing_count = repositories.iter()
        .filter(|r| r.sync_status == RepositorySyncStatus::Syncing)
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
    use crate::services::data::vcs_detector::VcsDetector;
    
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
pub struct CheckRepoRequest {
    pub repo_url: String,
    pub credentials: RepositoryCredentials,
}

#[derive(serde::Serialize)]
pub struct CheckRepoResponse {
    pub owner: String,
    pub repo_name: String,
    pub provider: String,
    pub vcs_type: String,
    pub metadata: RepositoryMetadata,
    pub branches: Vec<String>,
    pub default_branch: String,
    pub clone_https: String,
    pub clone_ssh: Option<String>,
}

pub async fn check_repository(
    req: web::Json<CheckRepoRequest>,
) -> Result<HttpResponse> {
    let url = &req.repo_url;
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
        Ok((o, r)) => (o, r),
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                message: "Failed to parse repository URL".to_string(),
                data: None,
                error: Some(e),
            }));
        }
    };

    let connector = VcsConnectorFactory::create_connector(&vcs_type);
    // Fetch repository metadata
    let metadata = match connector.get_repository_metadata(url, &req.credentials).await {
        Ok(m) => m,
        Err(e) => {
            let (mut status, error_msg) = match e {
                VcsError::AuthenticationFailed(msg) => (HttpResponse::Unauthorized(), msg),
                VcsError::RepositoryNotFound(msg) => (HttpResponse::NotFound(), msg),
                VcsError::InvalidCredentials(msg) => (HttpResponse::BadRequest(), msg),
                VcsError::InvalidUrl(msg) => (HttpResponse::BadRequest(), msg),
                VcsError::PermissionDenied(msg) => (HttpResponse::Forbidden(), msg),
                _ => (HttpResponse::InternalServerError(), e.to_string()),
            };
            return Ok(status.json(ApiResponse::<()> {
                success: false,
                message: "Failed to fetch repository metadata".to_string(),
                data: None,
                error: Some(error_msg),
            }));
        }
    };

    // Fetch branches
    let branches_info = match connector.list_branches(url, &req.credentials).await {
        Ok(b) => b,
        Err(e) => {
            let (mut status, error_msg) = match e {
                VcsError::AuthenticationFailed(msg) => (HttpResponse::Unauthorized(), msg),
                VcsError::RepositoryNotFound(msg) => (HttpResponse::NotFound(), msg),
                VcsError::InvalidCredentials(msg) => (HttpResponse::BadRequest(), msg),
                VcsError::InvalidUrl(msg) => (HttpResponse::BadRequest(), msg),
                VcsError::PermissionDenied(msg) => (HttpResponse::Forbidden(), msg),
                _ => (HttpResponse::InternalServerError(), e.to_string()),
            };
            return Ok(status.json(ApiResponse::<()> {
                success: false,
                message: "Failed to fetch branches".to_string(),
                data: None,
                error: Some(error_msg),
            }));
        }
    };

    let branches: Vec<String> = branches_info.iter().map(|b| b.name.clone()).collect();
    let default_branch = metadata.default_branch.clone();

    // Clone URLs
    let clone_urls = match VcsDetector::generate_clone_urls(url, &provider) {
        Ok(c) => c,
        Err(_) => crate::services::data::vcs_detector::CloneUrls { https: url.to_string(), ssh: None, original: url.to_string() },
    };

    let result = CheckRepoResponse {
        owner,
        repo_name,
        provider: format!("{:?}", provider),
        vcs_type: format!("{:?}", vcs_type),
        metadata,
        branches,
        default_branch,
        clone_https: clone_urls.https,
        clone_ssh: clone_urls.ssh,
    };

    Ok(HttpResponse::Ok().json(ApiResponse::success(result)))
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
    use crate::services::data::vcs_connector::VcsConnectorFactory;
    use crate::services::data::vcs_detector::VcsDetector;
    
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
        credential_type: CredentialType::None,
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
                VcsError::AuthenticationFailed(msg) => (HttpResponse::Unauthorized(), msg),
                VcsError::RepositoryNotFound(msg) => (HttpResponse::NotFound(), msg),
                VcsError::InvalidCredentials(msg) => (HttpResponse::BadRequest(), msg),
                VcsError::InvalidUrl(msg) => (HttpResponse::BadRequest(), msg),
                VcsError::PermissionDenied(msg) => (HttpResponse::Forbidden(), msg),
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


#[derive(serde::Deserialize)]
pub struct OAuthBranchesQuery {
    pub provider: String,
    pub repo: String,
}

pub async fn oauth_fetch_branches(
    claims: web::ReqData<Claims>,
    pool: web::Data<Option<PgPool>>,
    query: web::Query<OAuthBranchesQuery>,
) -> Result<HttpResponse> {
    let user_id = match Uuid::parse_str(&claims.sub) { Ok(id) => id, Err(_) => {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> { success: false, message: "Invalid user ID".to_string(), data: None, error: Some("Invalid user ID".to_string()) })) } };
    let pool = match pool.get_ref() { Some(p) => p, None => {
        return Ok(HttpResponse::ServiceUnavailable().json(ApiResponse::<()> { success: false, message: "Database not available".to_string(), data: None, error: Some("No database pool".to_string()) })) } };

    let platform = query.provider.to_lowercase();
    let repo_slug = query.repo.clone();
    let repo_url = if platform == "github" {
        format!("https://github.com/{}.git", repo_slug)
    } else if platform == "gitlab" {
        format!("https://gitlab.com/{}.git", repo_slug)
    } else if platform == "bitbucket" {
        format!("https://bitbucket.org/{}", repo_slug)
    } else {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> { success: false, message: "Unsupported provider".to_string(), data: None, error: Some("provider must be github, gitlab or bitbucket".to_string()) }))
    };

    let token_row = sqlx::query_scalar::<_, String>(
        "SELECT access_token FROM social_connections WHERE user_id = $1 AND platform = $2 AND is_active = TRUE ORDER BY updated_at DESC LIMIT 1",
    )
        .bind(user_id)
        .bind(&platform)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error fetching token: {}", e);
            e
        })?;

    if token_row.is_none() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()> { success: false, message: "No active connection for provider".to_string(), data: None, error: Some("Connect provider first".to_string()) }))
    }
    let access_token = token_row.unwrap();

    let credentials = RepositoryCredentials { credential_type: CredentialType::PersonalAccessToken { token: access_token }, expires_at: None };
    let (vcs_type, _provider) = match VcsDetector::detect_from_url(&repo_url) { Ok(r) => r, Err(e) => {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> { success: false, message: "Invalid repository URL".to_string(), data: None, error: Some(e) })) } };
    let connector = VcsConnectorFactory::create_connector(&vcs_type);
    match connector.list_branches(&repo_url, &credentials).await {
        Ok(branch_info) => {
            let branches: Vec<String> = branch_info.iter().map(|b| b.name.clone()).collect();
            let default_branch = branch_info.iter().find(|b| b.is_default).map(|b| b.name.clone()).or_else(|| {
                if branches.contains(&"main".to_string()) { Some("main".to_string()) } else if branches.contains(&"master".to_string()) { Some("master".to_string()) } else { branches.first().cloned() }
            });
            Ok(HttpResponse::Ok().json(ApiResponse { success: true, message: format!("Retrieved {} branches", branches.len()), data: Some(FetchBranchesResponse { branches, default_branch }), error: None }))
        }
        Err(e) => {
            let (mut status, error_msg) = match e { VcsError::AuthenticationFailed(msg) => (HttpResponse::Unauthorized(), msg), VcsError::RepositoryNotFound(msg) => (HttpResponse::NotFound(), msg), VcsError::InvalidCredentials(msg) => (HttpResponse::BadRequest(), msg), VcsError::InvalidUrl(msg) => (HttpResponse::BadRequest(), msg), VcsError::PermissionDenied(msg) => (HttpResponse::Forbidden(), msg), _ => (HttpResponse::InternalServerError(), e.to_string()) };
            Ok(status.json(ApiResponse::<()> { success: false, message: "Failed to fetch branches".to_string(), data: None, error: Some(error_msg) }))
        }
    }
}

#[derive(serde::Deserialize)]
pub struct OAuthCheckRepoRequest {
    pub provider: String,
    pub repo_url: String,
}

pub async fn oauth_check_repository(
    claims: web::ReqData<Claims>,
    pool: web::Data<Option<PgPool>>,
    req: web::Json<OAuthCheckRepoRequest>,
) -> Result<HttpResponse> {
    let user_id = match Uuid::parse_str(&claims.sub) { Ok(id) => id, Err(_) => {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> { success: false, message: "Invalid user ID".to_string(), data: None, error: Some("Invalid user ID".to_string()) })) } };
    let pool = match pool.get_ref() { Some(p) => p, None => {
        return Ok(HttpResponse::ServiceUnavailable().json(ApiResponse::<()> { success: false, message: "Database not available".to_string(), data: None, error: Some("No database pool".to_string()) })) } };

    let platform = req.provider.to_lowercase();
    if platform != "github" && platform != "gitlab" && platform != "bitbucket" {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> { success: false, message: "Unsupported provider".to_string(), data: None, error: Some("provider must be github, gitlab or bitbucket".to_string()) }))
    }

    let token_row = sqlx::query_scalar::<_, String>(
        "SELECT access_token FROM social_connections WHERE user_id = $1 AND platform = $2 AND is_active = TRUE ORDER BY updated_at DESC LIMIT 1",
    )
        .bind(user_id)
        .bind(&platform)
        .fetch_optional(pool)
        .await
        .map_err(|e| { tracing::error!("DB error fetching token: {}", e); e })?;

    if token_row.is_none() {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()> { success: false, message: "No active connection for provider".to_string(), data: None, error: Some("Connect provider first".to_string()) }))
    }
    let access_token = token_row.unwrap();
    let credentials = RepositoryCredentials { credential_type: CredentialType::PersonalAccessToken { token: access_token }, expires_at: None };

    let (vcs_type, provider) = match VcsDetector::detect_from_url(&req.repo_url) { Ok(r) => r, Err(e) => {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> { success: false, message: "Invalid repository URL".to_string(), data: None, error: Some(e) })) } };
    let connector = VcsConnectorFactory::create_connector(&vcs_type);

    let metadata = match connector.get_repository_metadata(&req.repo_url, &credentials).await {
        Ok(m) => m,
        Err(e) => {
            let (mut status, error_msg) = match e { VcsError::AuthenticationFailed(msg) => (HttpResponse::Unauthorized(), msg), VcsError::RepositoryNotFound(msg) => (HttpResponse::NotFound(), msg), VcsError::InvalidCredentials(msg) => (HttpResponse::BadRequest(), msg), VcsError::InvalidUrl(msg) => (HttpResponse::BadRequest(), msg), VcsError::PermissionDenied(msg) => (HttpResponse::Forbidden(), msg), _ => (HttpResponse::InternalServerError(), e.to_string()) };
            return Ok(status.json(ApiResponse::<()> { success: false, message: "Failed to fetch repository metadata".to_string(), data: None, error: Some(error_msg) }))
        }
    };

    let branches_info = match connector.list_branches(&req.repo_url, &credentials).await {
        Ok(b) => b,
        Err(e) => {
            let (mut status, error_msg) = match e { VcsError::AuthenticationFailed(msg) => (HttpResponse::Unauthorized(), msg), VcsError::RepositoryNotFound(msg) => (HttpResponse::NotFound(), msg), VcsError::InvalidCredentials(msg) => (HttpResponse::BadRequest(), msg), VcsError::InvalidUrl(msg) => (HttpResponse::BadRequest(), msg), VcsError::PermissionDenied(msg) => (HttpResponse::Forbidden(), msg), _ => (HttpResponse::InternalServerError(), e.to_string()) };
            return Ok(status.json(ApiResponse::<()> { success: false, message: "Failed to fetch branches".to_string(), data: None, error: Some(error_msg) }))
        }
    };
    let branches: Vec<String> = branches_info.iter().map(|b| b.name.clone()).collect();
    let default_branch = metadata.default_branch.clone();
    let clone_urls = match VcsDetector::generate_clone_urls(&req.repo_url, &provider) { Ok(c) => c, Err(_) => crate::services::data::vcs_detector::CloneUrls { https: req.repo_url.clone(), ssh: None, original: req.repo_url.clone() } };

    let result = CheckRepoResponse {
        owner: VcsDetector::extract_repo_info(&req.repo_url).ok().map(|(o, _)| o).unwrap_or_default(),
        repo_name: VcsDetector::extract_repo_info(&req.repo_url).ok().map(|(_, r)| r).unwrap_or_default(),
        provider: format!("{:?}", provider),
        vcs_type: format!("{:?}", vcs_type),
        metadata,
        branches,
        default_branch,
        clone_https: clone_urls.https,
        clone_ssh: clone_urls.ssh,
    };
    Ok(HttpResponse::Ok().json(ApiResponse::success(result)))
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/repositories")
            .route("/connect", web::post().to(connect_repository))
            .route("/check", web::post().to(check_repository))
            .route("/oauth/check", web::post().to(oauth_check_repository))
            .route("/validate-url", web::post().to(validate_repository_url))
            .route("/fetch-branches", web::post().to(fetch_branches))
            .route("/oauth/branches", web::get().to(oauth_fetch_branches))
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
