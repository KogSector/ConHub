use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::Utc;
use lazy_static::lazy_static;

use conhub_models::mcp::*;
use conhub_models::ApiResponse;
use crate::services::mcp_server::ConHubMcpServer;
use crate::services::mcp_client::{McpClient, AuthConfig, McpClientConfig};


lazy_static! {
    static ref MCP_SERVER: Arc<Mutex<Option<Arc<ConHubMcpServer>>>> = Arc::new(Mutex::new(None));
    static ref MCP_CLIENT_MANAGER: Arc<Mutex<Option<Arc<McpClientManager>>>> = Arc::new(Mutex::new(None));
}


pub struct McpClientManager {
    clients: tokio::sync::RwLock<HashMap<String, McpClient>>,
    client_configs: tokio::sync::RwLock<HashMap<String, McpClientConnection>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpClientConnection {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub auth_method: String,
    pub status: String,
    pub connected_at: Option<chrono::DateTime<Utc>>,
    pub last_ping: Option<chrono::DateTime<Utc>>,
    pub error_count: u32,
}





#[derive(Deserialize)]
pub struct CreateMcpServerRequest {
    pub name: String,
    #[allow(dead_code)]
    pub description: Option<String>,
    #[allow(dead_code)]
    pub enable_auth: Option<bool>,
    #[allow(dead_code)]
    pub rate_limit: Option<u32>,
}

#[derive(Serialize)]
pub struct McpServerInfo {
    pub server_info: ServerInfo,
    pub capabilities: ServerCapabilities,
    pub status: String,
    pub connections: u32,
    pub uptime: String,
}

#[derive(Deserialize)]
pub struct ConnectExternalMcpRequest {
    pub name: String,
    pub endpoint: String,
    pub auth_method: String,
    pub credentials: serde_json::Value,
}

#[derive(Deserialize)]
pub struct CreateContextRequest {
    pub name: String,
    pub context_type: String,
    pub resources: Vec<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Deserialize)]
pub struct CallToolRequest {
    pub server_id: Option<String>,
    pub tool_name: String,
    pub arguments: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct ListResourcesRequest {
    pub server_id: Option<String>,
    pub cursor: Option<String>,
}

#[derive(Deserialize)]
pub struct ReadResourceRequest {
    pub server_id: Option<String>,
    pub uri: String,
}

impl McpClientManager {
    pub fn new() -> Self {
        Self {
            clients: tokio::sync::RwLock::new(HashMap::new()),
            client_configs: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    pub async fn add_client(&self, connection: McpClientConnection, client: McpClient) {
        let mut clients = self.clients.write().await;
        let mut configs = self.client_configs.write().await;
        
        clients.insert(connection.id.clone(), client);
        configs.insert(connection.id.clone(), connection);
    }

    pub async fn get_client(&self, client_id: &str) -> Option<McpClient> {
        let clients = self.clients.read().await;
        clients.get(client_id).cloned()
    }

    pub async fn list_connections(&self) -> Vec<McpClientConnection> {
        let configs = self.client_configs.read().await;
        configs.values().cloned().collect()
    }

    pub async fn remove_client(&self, client_id: &str) -> bool {
        let mut clients = self.clients.write().await;
        let mut configs = self.client_configs.write().await;
        
        let removed_client = clients.remove(client_id).is_some();
        let removed_config = configs.remove(client_id).is_some();
        
        removed_client || removed_config
    }
}






pub async fn initialize_mcp_server(req: web::Json<CreateMcpServerRequest>) -> Result<HttpResponse> {
    let server_guard = (*MCP_SERVER).lock().await;
    if server_guard.is_some() {
        return Ok(HttpResponse::Conflict().json(ApiResponse::<()> {
            success: false,
            message: "MCP server already initialized".to_string(),
            data: None,
            error: Some("Server is already running".to_string()),
        }));
    }
    drop(server_guard);

    let mut server = ConHubMcpServer::new();
    match server.initialize().await {
        Ok(_) => {
            {
                let mut server_guard = (*MCP_SERVER).lock().await;
                *server_guard = Some(Arc::new(server));
            }
            {
                let mut client_guard = (*MCP_CLIENT_MANAGER).lock().await;
                *client_guard = Some(Arc::new(McpClientManager::new()));
            }

            log::info!("MCP server initialized successfully: {}", req.name);

            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: "MCP server initialized successfully".to_string(),
                data: Some(json!({
                    "server_id": "conhub-mcp-server",
                    "name": req.name,
                    "status": "running",
                    "initialized_at": Utc::now()
                })),
                error: None,
            }))
        }
        Err(e) => {
            log::error!("Failed to initialize MCP server: {}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                message: "Failed to initialize MCP server".to_string(),
                data: None,
                error: Some(e.to_string()),
            }))
        }
    }
}


pub async fn get_mcp_server_status() -> Result<HttpResponse> {
    let server_guard = (*MCP_SERVER).lock().await;
    match server_guard.as_ref() {
        Some(server) => {
            let server_info = McpServerInfo {
                server_info: server.server_info(),
                capabilities: server.capabilities(),
                status: "running".to_string(),
                connections: 0, 
                uptime: "N/A".to_string(), 
            };

            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: "MCP server status retrieved".to_string(),
                data: Some(server_info),
                error: None,
            }))
        }
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            message: "MCP server not initialized".to_string(),
            data: None,
            error: Some("Server not running".to_string()),
        })),
    }
}


pub async fn stop_mcp_server() -> Result<HttpResponse> {
    let mut server_guard = (*MCP_SERVER).lock().await;
    if server_guard.is_some() {
        *server_guard = None;
        drop(server_guard);
        
        let mut client_guard = (*MCP_CLIENT_MANAGER).lock().await;
        *client_guard = None;

        log::info!("MCP server stopped");

        Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: "MCP server stopped successfully".to_string(),
            data: Some(json!({
                "stopped_at": Utc::now()
            })),
            error: None,
        }))
    } else {
        Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            message: "MCP server not running".to_string(),
            data: None,
            error: Some("Server not initialized".to_string()),
        }))
    }
}


pub async fn connect_external_mcp(req: web::Json<ConnectExternalMcpRequest>) -> Result<HttpResponse> {
    let manager_guard = (*MCP_CLIENT_MANAGER).lock().await;
    match manager_guard.as_ref() {
        Some(manager) => {
            
            let auth_config = match req.auth_method.as_str() {
                "api_key" => {
                    let api_key = match req.credentials.get("api_key")
                        .and_then(|v| v.as_str()) {
                            Some(key) => key,
                            None => {
                                return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                                    "Missing API key".to_string(),
                                )));
                            }
                        };
                        AuthConfig::api_key(api_key.to_string())
                    }
                    "bearer" => {
                        let token = match req.credentials.get("token")
                            .and_then(|v| v.as_str()) {
                            Some(token) => token,
                            None => {
                                return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                                    "Missing bearer token".to_string(),
                                )));
                            }
                        };
                        AuthConfig::bearer(token.to_string())
                    }
                    "oauth2" => {
                        let access_token = match req.credentials.get("access_token")
                            .and_then(|v| v.as_str()) {
                            Some(token) => token,
                            None => {
                                return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                                    "Missing access token".to_string(),
                                )));
                            }
                        };
                        let refresh_token = req.credentials.get("refresh_token")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        AuthConfig::oauth2(access_token.to_string(), refresh_token)
                    }
                    _ => {
                        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
                            success: false,
                            message: "Unsupported authentication method".to_string(),
                            data: None,
                            error: Some(format!("Auth method '{}' not supported", req.auth_method)),
                        }));
                    }
                };

                
                let client = match McpClient::with_config(McpClientConfig::default()) {
                    Ok(client) => client,
                    Err(e) => {
                        return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                            success: false,
                            message: "Failed to create MCP client".to_string(),
                            data: None,
                            error: Some(e.to_string()),
                        }));
                    }
                };

                
                let server_id = match client.connect(req.endpoint.clone(), auth_config).await {
                    Ok(id) => id,
                    Err(e) => {
                        return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                            success: false,
                            message: "Failed to connect to external MCP server".to_string(),
                            data: None,
                            error: Some(e.to_string()),
                        }));
                    }
                };

                
                let connection = McpClientConnection {
                    id: server_id.clone(),
                    name: req.name.clone(),
                    endpoint: req.endpoint.clone(),
                    auth_method: req.auth_method.clone(),
                    status: "connected".to_string(),
                    connected_at: Some(Utc::now()),
                    last_ping: None,
                    error_count: 0,
                };

                
                manager.add_client(connection.clone(), client).await;

                log::info!("Connected to external MCP server: {} ({})", req.name, req.endpoint);

                Ok(HttpResponse::Ok().json(ApiResponse {
                    success: true,
                    message: "Connected to external MCP server successfully".to_string(),
                    data: Some(connection),
                    error: None,
                }))
        }
        None => Ok(HttpResponse::ServiceUnavailable().json(ApiResponse::<()> {
            success: false,
            message: "MCP client manager not available".to_string(),
            data: None,
            error: Some("MCP server not initialized".to_string()),
        })),
    }
}


pub async fn list_external_mcp_connections() -> Result<HttpResponse> {
    let manager_guard = (*MCP_CLIENT_MANAGER).lock().await;
    match manager_guard.as_ref() {
        Some(manager) => {
            let connections = manager.list_connections().await;

            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: "External MCP connections retrieved".to_string(),
                data: Some(connections),
                error: None,
            }))
        }
        None => Ok(HttpResponse::ServiceUnavailable().json(ApiResponse::<()> {
            success: false,
            message: "MCP client manager not available".to_string(),
            data: None,
            error: Some("MCP server not initialized".to_string()),
        })),
    }
}


pub async fn disconnect_external_mcp(path: web::Path<String>) -> Result<HttpResponse> {
    let connection_id = path.into_inner();

    let manager_guard = (*MCP_CLIENT_MANAGER).lock().await;
    match manager_guard.as_ref() {
        Some(manager) => {
            if let Some(client) = manager.get_client(&connection_id).await {
                
                if let Err(e) = client.disconnect(&connection_id).await {
                    log::warn!("Error during disconnection: {}", e);
                }

                
                let removed = manager.remove_client(&connection_id).await;

                if removed {
                    log::info!("Disconnected from external MCP server: {}", connection_id);

                    Ok(HttpResponse::Ok().json(ApiResponse {
                        success: true,
                        message: "Disconnected from external MCP server".to_string(),
                        data: Some(json!({
                            "connection_id": connection_id,
                            "disconnected_at": Utc::now()
                        })),
                        error: None,
                    }))
                } else {
                    Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
                        success: false,
                        message: "MCP connection not found".to_string(),
                        data: None,
                        error: Some(format!("Connection {} not found", connection_id)),
                    }))
                }
            } else {
                Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
                    success: false,
                    message: "MCP connection not found".to_string(),
                    data: None,
                    error: Some(format!("Connection {} not found", connection_id)),
                }))
            }
        }
        None => Ok(HttpResponse::ServiceUnavailable().json(ApiResponse::<()> {
            success: false,
            message: "MCP client manager not available".to_string(),
            data: None,
            error: Some("MCP server not initialized".to_string()),
        })),
    }
}


pub async fn create_mcp_context(req: web::Json<CreateContextRequest>) -> Result<HttpResponse> {
    let server_guard = (*MCP_SERVER).lock().await;
    match server_guard.as_ref() {
        Some(server) => {
            let context_type = match req.context_type.as_str() {
                "repository" => ContextType::Repository,
                "document" => ContextType::Document,
                "url" => ContextType::Url,
                "data_source" => ContextType::DataSource,
                "agent" => ContextType::Agent,
                "conversation" => ContextType::Conversation,
                "tool" => ContextType::Tool,
                other => ContextType::Custom(other.to_string()),
            };

            
            let params = ContextCreateParams {
                name: req.name.clone(),
                context_type,
                resources: req.resources.clone(),
                metadata: req.metadata.clone(),
            };

            
            let params_value = match serde_json::to_value(params) {
                Ok(v) => v,
                Err(e) => {
                    return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                        format!("Failed to serialize params: {}", e),
                    )));
                }
            };
            
            match server.handle_context_create(Some(params_value)).await {
                Ok(result) => {
                    let context_result: ContextCreateResult = match serde_json::from_value(result) {
                        Ok(result) => result,
                        Err(e) => {
                            return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                                format!("Failed to parse context result: {}", e),
                            )));
                        }
                    };

                    Ok(HttpResponse::Ok().json(ApiResponse {
                        success: true,
                        message: "MCP context created successfully".to_string(),
                        data: Some(context_result.context),
                        error: None,
                    }))
                }
                Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                    success: false,
                    message: "Failed to create MCP context".to_string(),
                    data: None,
                    error: Some(e.to_string()),
                })),
            }
        }
        None => Ok(HttpResponse::ServiceUnavailable().json(ApiResponse::<()> {
            success: false,
            message: "MCP server not available".to_string(),
            data: None,
            error: Some("MCP server not initialized".to_string()),
        })),
    }
}


pub async fn list_mcp_resources(query: web::Query<ListResourcesRequest>) -> Result<HttpResponse> {
    if let Some(server_id) = &query.server_id {
        
        let manager_guard = (*MCP_CLIENT_MANAGER).lock().await;
        match manager_guard.as_ref() {
            Some(manager) => {
                if let Some(client) = manager.get_client(server_id).await {
                    match client.list_resources(server_id, query.cursor.clone()).await {
                        Ok(result) => Ok(HttpResponse::Ok().json(ApiResponse {
                            success: true,
                            message: "Resources listed successfully".to_string(),
                            data: Some(result),
                            error: None,
                        })),
                        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                            success: false,
                            message: "Failed to list resources from external server".to_string(),
                            data: None,
                            error: Some(e.to_string()),
                        })),
                    }
                } else {
                    Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
                        success: false,
                        message: "External MCP server not found".to_string(),
                        data: None,
                        error: Some(format!("Server {} not found", server_id)),
                    }))
                }
            }
            None => Ok(HttpResponse::ServiceUnavailable().json(ApiResponse::<()> {
                success: false,
                message: "MCP client manager not available".to_string(),
                data: None,
                error: Some("MCP server not initialized".to_string()),
            })),
        }
    } else {
        
        let server_guard = (*MCP_SERVER).lock().await;
        match server_guard.as_ref() {
            Some(server) => {
                let params = ResourcesListParams {
                    cursor: query.cursor.clone(),
                };

                let params_value = match serde_json::to_value(params) {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                            format!("Failed to serialize params: {}", e),
                        )));
                    }
                };

                match server.handle_resources_list(Some(params_value)).await {
                    Ok(result) => {
                        let resources_result: ResourcesListResult = match serde_json::from_value(result) {
                            Ok(result) => result,
                            Err(e) => {
                                return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                                    format!("Failed to parse resources result: {}", e),
                                )));
                            }
                        };

                        Ok(HttpResponse::Ok().json(ApiResponse {
                            success: true,
                            message: "Resources listed successfully".to_string(),
                            data: Some(resources_result),
                            error: None,
                        }))
                    }
                    Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                        success: false,
                        message: "Failed to list resources".to_string(),
                        data: None,
                        error: Some(e.to_string()),
                    })),
                }
            }
            None => Ok(HttpResponse::ServiceUnavailable().json(ApiResponse::<()> {
                success: false,
                message: "MCP server not available".to_string(),
                data: None,
                error: Some("MCP server not initialized".to_string()),
            })),
        }
    }
}


pub async fn read_mcp_resource(req: web::Json<ReadResourceRequest>) -> Result<HttpResponse> {
    if let Some(server_id) = &req.server_id {
        
        let manager_guard = (*MCP_CLIENT_MANAGER).lock().await;
        match manager_guard.as_ref() {
            Some(manager) => {
                if let Some(client) = manager.get_client(server_id).await {
                    match client.read_resource(server_id, req.uri.clone()).await {
                        Ok(result) => Ok(HttpResponse::Ok().json(ApiResponse {
                            success: true,
                            message: "Resource read successfully".to_string(),
                            data: Some(result),
                            error: None,
                        })),
                        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                            success: false,
                            message: "Failed to read resource from external server".to_string(),
                            data: None,
                            error: Some(e.to_string()),
                        })),
                    }
                } else {
                    Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
                        success: false,
                        message: "External MCP server not found".to_string(),
                        data: None,
                        error: Some(format!("Server {} not found", server_id)),
                    }))
                }
            }
            None => Ok(HttpResponse::ServiceUnavailable().json(ApiResponse::<()> {
                success: false,
                message: "MCP client manager not available".to_string(),
                data: None,
                error: Some("MCP server not initialized".to_string()),
            })),
        }
    } else {
        
        let server_guard = (*MCP_SERVER).lock().await;
        match server_guard.as_ref() {
            Some(server) => {
                let params = ResourcesReadParams {
                    uri: req.uri.clone(),
                };

                let params_value = match serde_json::to_value(params) {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                            format!("Failed to serialize params: {}", e),
                        )));
                    }
                };

                match server.handle_resources_read(Some(params_value)).await {
                    Ok(result) => {
                        let read_result: ResourcesReadResult = match serde_json::from_value(result) {
                            Ok(result) => result,
                            Err(e) => {
                                return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                                    format!("Failed to parse read result: {}", e),
                                )));
                            }
                        };

                        Ok(HttpResponse::Ok().json(ApiResponse {
                            success: true,
                            message: "Resource read successfully".to_string(),
                            data: Some(read_result),
                            error: None,
                        }))
                    }
                    Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                        success: false,
                        message: "Failed to read resource".to_string(),
                        data: None,
                        error: Some(e.to_string()),
                    })),
                }
            }
            None => Ok(HttpResponse::ServiceUnavailable().json(ApiResponse::<()> {
                success: false,
                message: "MCP server not available".to_string(),
                data: None,
                error: Some("MCP server not initialized".to_string()),
            })),
        }
    }
}


pub async fn call_mcp_tool(req: web::Json<CallToolRequest>) -> Result<HttpResponse> {
    if let Some(server_id) = &req.server_id {
        
        let manager_guard = (*MCP_CLIENT_MANAGER).lock().await;
        match manager_guard.as_ref() {
            Some(manager) => {
                if let Some(client) = manager.get_client(server_id).await {
                    match client.call_tool(server_id, req.tool_name.clone(), req.arguments.clone()).await {
                        Ok(result) => Ok(HttpResponse::Ok().json(ApiResponse {
                            success: true,
                            message: "Tool called successfully".to_string(),
                            data: Some(result),
                            error: None,
                        })),
                        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                            success: false,
                            message: "Failed to call tool on external server".to_string(),
                            data: None,
                            error: Some(e.to_string()),
                        })),
                    }
                } else {
                    Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
                        success: false,
                        message: "External MCP server not found".to_string(),
                        data: None,
                        error: Some(format!("Server {} not found", server_id)),
                    }))
                }
            }
            None => Ok(HttpResponse::ServiceUnavailable().json(ApiResponse::<()> {
                success: false,
                message: "MCP client manager not available".to_string(),
                data: None,
                error: Some("MCP server not initialized".to_string()),
            })),
        }
    } else {
        
        let server_guard = (*MCP_SERVER).lock().await;
        match server_guard.as_ref() {
            Some(server) => {
                let params = ToolsCallParams {
                    name: req.tool_name.clone(),
                    arguments: req.arguments.clone(),
                };

                let params_value = match serde_json::to_value(params) {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                            format!("Failed to serialize params: {}", e),
                        )));
                    }
                };

                match server.handle_tools_call(Some(params_value)).await {
                    Ok(result) => {
                        let call_result: ToolsCallResult = match serde_json::from_value(result) {
                            Ok(result) => result,
                            Err(e) => {
                                return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                                    format!("Failed to parse tool result: {}", e),
                                )));
                            }
                        };

                        Ok(HttpResponse::Ok().json(ApiResponse {
                            success: true,
                            message: "Tool called successfully".to_string(),
                            data: Some(call_result),
                            error: None,
                        }))
                    }
                    Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                        success: false,
                        message: "Failed to call tool".to_string(),
                        data: None,
                        error: Some(e.to_string()),
                    })),
                }
            }
            None => Ok(HttpResponse::ServiceUnavailable().json(ApiResponse::<()> {
                success: false,
                message: "MCP server not available".to_string(),
                data: None,
                error: Some("MCP server not initialized".to_string()),
            })),
        }
    }
}


pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/mcp")
            
            .route("/server/initialize", web::post().to(initialize_mcp_server))
            .route("/server/status", web::get().to(get_mcp_server_status))
            .route("/server/stop", web::post().to(stop_mcp_server))
            
            
            .route("/external/connect", web::post().to(connect_external_mcp))
            .route("/external/connections", web::get().to(list_external_mcp_connections))
            .route("/external/{connection_id}/disconnect", web::delete().to(disconnect_external_mcp))
            
            
            .route("/contexts", web::post().to(create_mcp_context))
            
            
            .route("/resources", web::get().to(list_mcp_resources))
            .route("/resources/read", web::post().to(read_mcp_resource))
            
            
            .route("/tools/call", web::post().to(call_mcp_tool))
    );
}
