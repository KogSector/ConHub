use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use crate::models::ApiResponse;
use crate::models::copilot::CopilotEnhancedContext;
use crate::services::github_copilot_integration::*;

/// Global GitHub Copilot integration instance
static mut COPILOT_INTEGRATION: Option<Arc<GitHubCopilotIntegration>> = None;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Deserialize)]
pub struct InitializeCopilotSessionRequest {
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub auth_token: String,
}

#[derive(Serialize)]
pub struct CopilotSessionResponse {
    pub session_id: String,
    pub expires_at: String,
    pub capabilities: CopilotCapabilities,
}

#[derive(Deserialize)]
pub struct CopilotContextQueryRequest {
    pub session_id: String,
    pub context_type: String,
    pub query: Option<String>,
    pub workspace_path: Option<String>,
    pub file_patterns: Option<Vec<String>>,
    pub include_dependencies: Option<bool>,
}

#[derive(Deserialize)]
pub struct CopilotToolCallRequest {
    pub session_id: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub context_id: Option<String>,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get or initialize the GitHub Copilot integration
async fn get_copilot_integration() -> Result<Arc<GitHubCopilotIntegration>, Box<dyn std::error::Error>> {
    unsafe {
        if COPILOT_INTEGRATION.is_none() {
            COPILOT_INTEGRATION = Some(Arc::new(GitHubCopilotIntegration::new()));
        }
        Ok(COPILOT_INTEGRATION.as_ref().unwrap().clone())
    }
}

// ============================================================================
// API Handlers
// ============================================================================

/// Initialize GitHub Copilot session
/// POST /api/copilot/session
pub async fn initialize_copilot_session(
    req: web::Json<InitializeCopilotSessionRequest>,
) -> Result<HttpResponse> {
    let integration = match get_copilot_integration().await {
        Ok(integration) => integration,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                format!("Failed to initialize Copilot integration: {}", e),
            )));
        }
    };

    match integration.initialize_session(
        req.user_id.clone(),
        req.workspace_id.clone(),
        req.auth_token.clone(),
    ).await {
        Ok(session_id) => {
            let capabilities = integration.get_capabilities().await;
            let expires_at = chrono::Utc::now() + chrono::Duration::seconds(3600); // 1 hour
            
            let response = CopilotSessionResponse {
                session_id,
                expires_at: expires_at.to_rfc3339(),
                capabilities,
            };

            Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
        }
        Err(e) => Ok(HttpResponse::BadRequest().json(ApiResponse::<CopilotSessionResponse>::error(
            format!("Failed to initialize session: {}", e),
        ))),
    }
}

/// Get GitHub Copilot capabilities
/// GET /api/copilot/capabilities
pub async fn get_copilot_capabilities() -> Result<HttpResponse> {
    let integration = match get_copilot_integration().await {
        Ok(integration) => integration,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(ApiResponse::<CopilotCapabilities>::error(
                format!("Failed to get Copilot integration: {}", e),
            )));
        }
    };

    let capabilities = integration.get_capabilities().await;
    Ok(HttpResponse::Ok().json(ApiResponse::success(capabilities)))
}

/// Handle Copilot context request
/// POST /api/copilot/context
pub async fn handle_copilot_context(
    req: web::Json<CopilotContextQueryRequest>,
) -> Result<HttpResponse> {
    let integration = match get_copilot_integration().await {
        Ok(integration) => integration,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(ApiResponse::<CopilotEnhancedContext>::error(
                format!("Failed to get Copilot integration: {}", e),
            )));
        }
    };

    let context_request = CopilotContextRequest {
        session_id: req.session_id.clone(),
        context_type: req.context_type.clone(),
        query: req.query.clone(),
        workspace_path: req.workspace_path.clone(),
        file_patterns: req.file_patterns.clone(),
        include_dependencies: req.include_dependencies,
    };

    match integration.handle_context_request(context_request).await {
        Ok(response) => Ok(HttpResponse::Ok().json(ApiResponse::success(response))),
        Err(e) => Ok(HttpResponse::BadRequest().json(ApiResponse::<CopilotEnhancedContext>::error(
            format!("Context request failed: {}", e),
        ))),
    }
}

/// Handle Copilot tool call
/// POST /api/copilot/tools/call
pub async fn handle_copilot_tool_call(
    req: web::Json<CopilotToolCallRequest>,
) -> Result<HttpResponse> {
    let integration = match get_copilot_integration().await {
        Ok(integration) => integration,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(ApiResponse::<CopilotResponse>::error(
                format!("Failed to get Copilot integration: {}", e),
            )));
        }
    };

    let tool_request = CopilotToolRequest {
        session_id: req.session_id.clone(),
        tool_name: req.tool_name.clone(),
        arguments: req.arguments.clone(),
        context_id: req.context_id.clone(),
    };

    match integration.handle_tool_request(tool_request).await {
        Ok(response) => Ok(HttpResponse::Ok().json(ApiResponse::success(response))),
        Err(e) => Ok(HttpResponse::BadRequest().json(ApiResponse::<CopilotResponse>::error(
            format!("Tool call failed: {}", e),
        ))),
    }
}

/// Get Copilot health status
/// GET /api/copilot/health
pub async fn get_copilot_health() -> Result<HttpResponse> {
    let integration = match get_copilot_integration().await {
        Ok(integration) => integration,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(ApiResponse::<serde_json::Value>::error(
                format!("Failed to get Copilot integration: {}", e),
            )));
        }
    };

    match integration.health_check().await {
        Ok(health_data) => Ok(HttpResponse::Ok().json(ApiResponse::success(health_data))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<serde_json::Value>::error(
            format!("Health check failed: {}", e),
        ))),
    }
}

/// List active Copilot sessions
/// GET /api/copilot/sessions
pub async fn list_copilot_sessions() -> Result<HttpResponse> {
    let integration = match get_copilot_integration().await {
        Ok(integration) => integration,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(ApiResponse::<serde_json::Value>::error(
                format!("Failed to get Copilot integration: {}", e),
            )));
        }
    };

    // Get session info (in a real implementation, this would return session summaries)
    let health_data = integration.health_check().await.unwrap_or_else(|_| json!({}));
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(json!({
        "active_sessions": health_data.get("active_sessions").unwrap_or(&json!(0)),
        "server_status": "running",
        "integration_status": "connected"
    }))))
}

/// Clean up expired Copilot sessions
/// POST /api/copilot/sessions/cleanup
pub async fn cleanup_copilot_sessions() -> Result<HttpResponse> {
    let integration = match get_copilot_integration().await {
        Ok(integration) => integration,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(ApiResponse::<serde_json::Value>::error(
                format!("Failed to get Copilot integration: {}", e),
            )));
        }
    };

    match integration.cleanup_expired_sessions().await {
        Ok(cleaned_count) => Ok(HttpResponse::Ok().json(ApiResponse::success(json!({
            "cleaned_sessions": cleaned_count,
            "message": format!("Cleaned up {} expired sessions", cleaned_count)
        })))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<serde_json::Value>::error(
            format!("Session cleanup failed: {}", e),
        ))),
    }
}

/// Get Copilot integration status and metrics
/// GET /api/copilot/status
pub async fn get_copilot_status() -> Result<HttpResponse> {
    let integration = match get_copilot_integration().await {
        Ok(integration) => integration,
        Err(e) => {
            return Ok(HttpResponse::ServiceUnavailable().json(ApiResponse::<serde_json::Value>::error(
                format!("Copilot integration not available: {}", e),
            )));
        }
    };

    let health_data = integration.health_check().await.unwrap_or_else(|_| json!({}));
    let capabilities = integration.get_capabilities().await;

    Ok(HttpResponse::Ok().json(ApiResponse::success(json!({
        "status": "running",
        "integration_type": "github_copilot",
        "mcp_protocol_version": crate::models::mcp::MCP_VERSION,
        "health": health_data,
        "capabilities": capabilities,
        "endpoints": [
            "/api/copilot/session",
            "/api/copilot/context", 
            "/api/copilot/tools/call",
            "/api/copilot/health",
            "/api/copilot/sessions",
            "/api/copilot/status"
        ]
    }))))
}

// ============================================================================
// Configuration for routes
// ============================================================================

pub fn configure_copilot_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/copilot")
            .route("/session", web::post().to(initialize_copilot_session))
            .route("/capabilities", web::get().to(get_copilot_capabilities))
            .route("/context", web::post().to(handle_copilot_context))
            .route("/tools/call", web::post().to(handle_copilot_tool_call))
            .route("/health", web::get().to(get_copilot_health))
            .route("/sessions", web::get().to(list_copilot_sessions))
            .route("/sessions/cleanup", web::post().to(cleanup_copilot_sessions))
            .route("/status", web::get().to(get_copilot_status))
    );
}