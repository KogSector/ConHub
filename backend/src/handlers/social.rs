use actix_web::{web, HttpRequest, HttpResponse, Result};
use crate::services::social_integration_service::SocialIntegrationService;
use crate::models::social::*;
use crate::middleware::auth::extract_user_id_from_http_request;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use log::{info, error};

#[derive(Deserialize)]
pub struct AuthUrlQuery {
    pub platform: String,
    pub state: Option<String>,
}

#[derive(Serialize)]
pub struct AuthUrlResponse {
    pub auth_url: String,
    pub state: String,
}

#[derive(Deserialize)]
pub struct ConnectPlatformRequest {
    pub platform: String,
    pub code: String,
    pub state: Option<String>,
}

#[derive(Deserialize)]
pub struct DisconnectRequest {
    pub connection_id: Uuid,
}

#[derive(Deserialize)]
pub struct SyncDataRequest {
    pub connection_id: Uuid,
    pub sync_type: Option<String>,
    pub options: Option<serde_json::Value>,
}

/// Get OAuth authorization URL for a platform
pub async fn get_auth_url(
    query: web::Query<AuthUrlQuery>,
    social_service: web::Data<SocialIntegrationService>,
) -> Result<HttpResponse> {
    let platform: SocialPlatform = match query.platform.parse() {
        Ok(p) => p,
        Err(_) => return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid platform",
            "supported_platforms": ["slack", "notion", "google_drive", "gmail", "dropbox", "linkedin"]
        }))),
    };

    match social_service.get_auth_url(platform, query.state.clone()) {
        Ok((auth_url, state)) => {
            info!("Generated auth URL for platform: {}", query.platform);
            Ok(HttpResponse::Ok().json(AuthUrlResponse { auth_url, state }))
        },
        Err(e) => {
            error!("Failed to generate auth URL for {}: {}", query.platform, e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to generate authorization URL",
                "details": e.to_string()
            })))
        }
    }
}

/// Connect a social platform
pub async fn connect_platform(
    req: HttpRequest,
    request: web::Json<ConnectPlatformRequest>,
    social_service: web::Data<SocialIntegrationService>,
) -> Result<HttpResponse> {
    let user_id = match extract_user_id_from_http_request(&req) {
        Some(id) => id,
        None => return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Authentication required"
        }))),
    };
    let platform: SocialPlatform = match request.platform.parse() {
        Ok(p) => p,
        Err(_) => return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid platform"
        }))),
    };

    let connection_request = SocialConnectionRequest {
        platform: platform.clone(),
        code: request.code.clone(),
        state: request.state.clone(),
    };

    match social_service.connect_platform(user_id, connection_request).await {
        Ok(connection) => {
            info!("Successfully connected {} for user {}", platform, user_id);
            Ok(HttpResponse::Ok().json(connection))
        },
        Err(e) => {
            error!("Failed to connect {} for user {}: {}", platform, user_id, e);
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Failed to connect platform",
                "details": e.to_string()
            })))
        }
    }
}

/// Get all connected platforms for the user
pub async fn get_connections(
    req: HttpRequest,
    social_service: web::Data<SocialIntegrationService>,
) -> Result<HttpResponse> {
    let user_id = match extract_user_id_from_http_request(&req) {
        Some(id) => id,
        None => return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Authentication required"
        }))),
    };

    match social_service.get_user_connections(user_id).await {
        Ok(connections) => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "connections": connections,
                "total": connections.len()
            })))
        },
        Err(e) => {
            error!("Failed to get connections for user {}: {}", user_id, e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to retrieve connections"
            })))
        }
    }
}

/// Disconnect a platform
pub async fn disconnect_platform(
    req: HttpRequest,
    request: web::Json<DisconnectRequest>,
    social_service: web::Data<SocialIntegrationService>,
) -> Result<HttpResponse> {
    let user_id = match extract_user_id_from_http_request(&req) {
        Some(id) => id,
        None => return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Authentication required"
        }))),
    };

    match social_service.disconnect_platform(user_id, request.connection_id).await {
        Ok(_) => {
            info!("Successfully disconnected connection {} for user {}", request.connection_id, user_id);
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Platform disconnected successfully"
            })))
        },
        Err(e) => {
            error!("Failed to disconnect connection {} for user {}: {}", request.connection_id, user_id, e);
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Failed to disconnect platform",
                "details": e.to_string()
            })))
        }
    }
}

/// Sync data from a connected platform
pub async fn sync_platform_data(
    req: HttpRequest,
    request: web::Json<SyncDataRequest>,
    social_service: web::Data<SocialIntegrationService>,
) -> Result<HttpResponse> {
    let _user_id = match extract_user_id_from_http_request(&req) {
        Some(id) => id,
        None => return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Authentication required"
        }))),
    };
    let sync_request = DataSyncRequest {
        connection_id: request.connection_id,
        sync_type: request.sync_type.clone().unwrap_or_else(|| "incremental".to_string()),
        options: request.options.clone(),
    };

    match social_service.sync_platform_data(request.connection_id, sync_request).await {
        Ok(sync_response) => {
            info!("Data sync completed for connection {}: {} items processed", 
                  request.connection_id, sync_response.items_processed);
            Ok(HttpResponse::Ok().json(sync_response))
        },
        Err(e) => {
            error!("Failed to sync data for connection {}: {}", request.connection_id, e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to sync platform data",
                "details": e.to_string()
            })))
        }
    }
}

/// Get supported platforms and their capabilities
pub async fn get_supported_platforms() -> Result<HttpResponse> {
    let platforms = serde_json::json!({
        "platforms": [
            {
                "id": "slack",
                "name": "Slack",
                "description": "Connect to Slack workspaces for team communication insights",
                "capabilities": ["channels", "messages", "files", "user_profiles"],
                "scopes": ["channels:read", "chat:write", "users:read", "files:read"]
            },
            {
                "id": "notion",
                "name": "Notion",
                "description": "Access Notion pages and databases for documentation insights",
                "capabilities": ["pages", "databases", "blocks"],
                "scopes": ["read_content"]
            },
            {
                "id": "google_drive",
                "name": "Google Drive",
                "description": "Access Google Drive files and folders",
                "capabilities": ["files", "folders", "sharing_permissions"],
                "scopes": ["drive.readonly", "userinfo.profile"]
            },
            {
                "id": "gmail",
                "name": "Gmail",
                "description": "Access Gmail messages for communication context",
                "capabilities": ["emails", "threads", "labels"],
                "scopes": ["gmail.readonly", "userinfo.profile"]
            },
            {
                "id": "dropbox",
                "name": "Dropbox",
                "description": "Access Dropbox files and folders",
                "capabilities": ["files", "folders", "sharing"],
                "scopes": ["files.content.read", "files.metadata.read"]
            },
            {
                "id": "linkedin",
                "name": "LinkedIn",
                "description": "Access LinkedIn profile and professional network",
                "capabilities": ["profile", "connections", "posts"],
                "scopes": ["r_liteprofile", "r_emailaddress"]
            }
        ]
    });

    Ok(HttpResponse::Ok().json(platforms))
}

/// Health check for social integrations service
pub async fn health_check(
    _social_service: web::Data<SocialIntegrationService>,
) -> Result<HttpResponse> {
    // You could add actual health checks here
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "social_integrations",
        "timestamp": chrono::Utc::now(),
        "configured_platforms": ["slack", "notion", "google_drive", "gmail", "dropbox", "linkedin"]
    })))
}

/// Configure social integration routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/social")
            .route("/platforms", web::get().to(get_supported_platforms))
            .route("/auth-url", web::get().to(get_auth_url))
            .route("/connect", web::post().to(connect_platform))
            .route("/connections", web::get().to(get_connections))
            .route("/disconnect", web::post().to(disconnect_platform))
            .route("/sync", web::post().to(sync_platform_data))
            .route("/health", web::get().to(health_check))
    );
}