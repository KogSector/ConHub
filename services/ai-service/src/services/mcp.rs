use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use reqwest::{Client, ClientBuilder};
use serde_json::json;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::Utc;

use conhub_models::mcp::*;






#[derive(Clone)]
pub struct McpClient {
    http_client: Client,
    connections: Arc<RwLock<HashMap<ServerId, McpConnection>>>,
    config: McpClientConfig,
}


#[derive(Debug, Clone)]
pub struct McpClientConfig {
    pub timeout: Duration,
    pub max_retries: u32,
    pub retry_delay: Duration,
    #[allow(dead_code)]
    pub max_concurrent_connections: usize,
    pub connection_pool_size: usize,
    #[allow(dead_code)]
    pub heartbeat_interval: Duration,
    #[allow(dead_code)]
    pub default_auth_method: AuthMethod,
}

impl Default for McpClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_millis(1000),
            max_concurrent_connections: 100,
            connection_pool_size: 10,
            heartbeat_interval: Duration::from_secs(60),
            default_auth_method: AuthMethod::ApiKey,
        }
    }
}


#[derive(Debug, Clone)]
pub struct McpConnection {
    #[allow(dead_code)]
    pub server_id: ServerId,
    pub endpoint: String,
    pub server_info: Option<ServerInfo>,
    pub capabilities: Option<ServerCapabilities>,
    pub auth_config: AuthConfig,
    pub status: ConnectionStatus,
    #[allow(dead_code)]
    pub last_ping: Option<chrono::DateTime<Utc>>,
    pub error_count: u32,
    #[allow(dead_code)]
    pub connected_at: chrono::DateTime<Utc>,
}


#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub method: AuthMethod,
    pub credentials: serde_json::Value,
}


#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    Disconnected,
    Error(String),
    #[allow(dead_code)]
    Timeout,
}


#[derive(Debug, Clone)]
pub enum McpClientError {
    ConnectionError(String),
    AuthenticationError(String),
    TimeoutError,
    ProtocolError(McpError),
    ServerError(String),
    InvalidResponse(String),
    NetworkError(String),
}

impl std::fmt::Display for McpClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpClientError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            McpClientError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            McpClientError::TimeoutError => write!(f, "Request timeout"),
            McpClientError::ProtocolError(err) => write!(f, "Protocol error: {}", err),
            McpClientError::ServerError(msg) => write!(f, "Server error: {}", msg),
            McpClientError::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
            McpClientError::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl std::error::Error for McpClientError {}

impl std::fmt::Debug for McpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpClient")
            .field("config", &self.config)
            .field("connections", &self.connections)
            .finish_non_exhaustive()
    }
}

impl McpClient {
    
    #[allow(dead_code)]
    pub fn new() -> Result<Self, McpClientError> {
        Self::with_config(McpClientConfig::default())
    }

    
    pub fn with_config(config: McpClientConfig) -> Result<Self, McpClientError> {
        let http_client = ClientBuilder::new()
            .timeout(config.timeout)
            .pool_max_idle_per_host(config.connection_pool_size)
            .build()
            .map_err(|e| McpClientError::NetworkError(e.to_string()))?;

        Ok(Self {
            http_client,
            connections: Arc::new(RwLock::new(HashMap::new())),
            config,
        })
    }

    
    pub async fn connect(
        &self,
        endpoint: String,
        auth_config: AuthConfig,
    ) -> Result<ServerId, McpClientError> {
        let server_id = Uuid::new_v4().to_string();
        
        
        let connection = McpConnection {
            server_id: server_id.clone(),
            endpoint: endpoint.clone(),
            server_info: None,
            capabilities: None,
            auth_config,
            status: ConnectionStatus::Connecting,
            last_ping: None,
            error_count: 0,
            connected_at: Utc::now(),
        };

        
        {
            let mut connections = self.connections.write().await;
            connections.insert(server_id.clone(), connection);
        }

        
        match self.initialize_connection(&server_id).await {
            Ok(_) => {
                
                let mut connections = self.connections.write().await;
                if let Some(conn) = connections.get_mut(&server_id) {
                    conn.status = ConnectionStatus::Connected;
                }
                
                log::info!("Successfully connected to MCP server: {}", endpoint);
                Ok(server_id)
            }
            Err(e) => {
                
                let mut connections = self.connections.write().await;
                if let Some(conn) = connections.get_mut(&server_id) {
                    conn.status = ConnectionStatus::Error(e.to_string());
                    conn.error_count += 1;
                }
                
                log::error!("Failed to connect to MCP server {}: {}", endpoint, e);
                Err(e)
            }
        }
    }

    
    async fn initialize_connection(&self, server_id: &ServerId) -> Result<(), McpClientError> {
        let _endpoint = {
            let connections = self.connections.read().await;
            let connection = connections.get(server_id)
                .ok_or_else(|| McpClientError::ConnectionError("Connection not found".to_string()))?;
            connection.endpoint.clone()
        };

        
        let client_info = ClientInfo {
            name: "ConHub MCP Client".to_string(),
            version: "1.0.0".to_string(),
        };

        let client_capabilities = ClientCapabilities {
            experimental: None,
            sampling: None,
        };

        let init_params = InitializeParams {
            protocol_version: MCP_VERSION.to_string(),
            capabilities: client_capabilities,
            client_info,
        };

        let request = McpRequest::Initialize(init_params);
        
        
        let response = self.send_request(&server_id, request).await?;
        
        match response {
            McpResponse::Initialize(init_result) => {
                
                let mut connections = self.connections.write().await;
                if let Some(conn) = connections.get_mut(server_id) {
                    conn.server_info = Some(init_result.server_info);
                    conn.capabilities = Some(init_result.capabilities);
                }
                Ok(())
            }
            McpResponse::Error(error) => {
                Err(McpClientError::ProtocolError(error))
            }
            _ => {
                Err(McpClientError::InvalidResponse("Expected initialize response".to_string()))
            }
        }
    }

    
    pub async fn disconnect(&self, server_id: &ServerId) -> Result<(), McpClientError> {
        let mut connections = self.connections.write().await;
        
        if let Some(connection) = connections.get_mut(server_id) {
            connection.status = ConnectionStatus::Disconnected;
            log::info!("Disconnected from MCP server: {}", connection.endpoint);
        }
        
        connections.remove(server_id);
        Ok(())
    }

    
    pub async fn list_resources(
        &self,
        server_id: &ServerId,
        cursor: Option<String>,
    ) -> Result<ResourcesListResult, McpClientError> {
        let request = McpRequest::ResourcesList(ResourcesListParams { cursor });
        
        let response = self.send_request(server_id, request).await?;
        
        match response {
            McpResponse::ResourcesList(result) => Ok(result),
            McpResponse::Error(error) => Err(McpClientError::ProtocolError(error)),
            _ => Err(McpClientError::InvalidResponse("Expected resources list response".to_string())),
        }
    }

    
    pub async fn read_resource(
        &self,
        server_id: &ServerId,
        uri: String,
    ) -> Result<ResourcesReadResult, McpClientError> {
        let request = McpRequest::ResourcesRead(ResourcesReadParams { uri });
        
        let response = self.send_request(server_id, request).await?;
        
        match response {
            McpResponse::ResourcesRead(result) => Ok(result),
            McpResponse::Error(error) => Err(McpClientError::ProtocolError(error)),
            _ => Err(McpClientError::InvalidResponse("Expected resources read response".to_string())),
        }
    }

    
    #[allow(dead_code)]
    pub async fn list_tools(
        &self,
        server_id: &ServerId,
        cursor: Option<String>,
    ) -> Result<ToolsListResult, McpClientError> {
        let request = McpRequest::ToolsList(ToolsListParams { cursor });
        
        let response = self.send_request(server_id, request).await?;
        
        match response {
            McpResponse::ToolsList(result) => Ok(result),
            McpResponse::Error(error) => Err(McpClientError::ProtocolError(error)),
            _ => Err(McpClientError::InvalidResponse("Expected tools list response".to_string())),
        }
    }

    
    pub async fn call_tool(
        &self,
        server_id: &ServerId,
        name: String,
        arguments: Option<serde_json::Value>,
    ) -> Result<ToolsCallResult, McpClientError> {
        let request = McpRequest::ToolsCall(ToolsCallParams { name, arguments });
        
        let response = self.send_request(server_id, request).await?;
        
        match response {
            McpResponse::ToolsCall(result) => Ok(result),
            McpResponse::Error(error) => Err(McpClientError::ProtocolError(error)),
            _ => Err(McpClientError::InvalidResponse("Expected tools call response".to_string())),
        }
    }

    
    #[allow(dead_code)]
    pub async fn create_context(
        &self,
        server_id: &ServerId,
        name: String,
        context_type: ContextType,
        resources: Vec<ResourceId>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<ContextCreateResult, McpClientError> {
        let request = McpRequest::ContextCreate(ContextCreateParams {
            name,
            context_type,
            resources,
            metadata,
        });
        
        let response = self.send_request(server_id, request).await?;
        
        match response {
            McpResponse::ContextCreate(result) => Ok(result),
            McpResponse::Error(error) => Err(McpClientError::ProtocolError(error)),
            _ => Err(McpClientError::InvalidResponse("Expected context create response".to_string())),
        }
    }

    
    #[allow(dead_code)]
    pub async fn get_context(
        &self,
        server_id: &ServerId,
        context_id: ContextId,
    ) -> Result<ContextGetResult, McpClientError> {
        let request = McpRequest::ContextGet(ContextGetParams { context_id });
        
        let response = self.send_request(server_id, request).await?;
        
        match response {
            McpResponse::ContextGet(result) => Ok(result),
            McpResponse::Error(error) => Err(McpClientError::ProtocolError(error)),
            _ => Err(McpClientError::InvalidResponse("Expected context get response".to_string())),
        }
    }

    
    #[allow(dead_code)]
    pub async fn ping(&self, server_id: &ServerId) -> Result<PongResult, McpClientError> {
        let request = McpRequest::Ping(PingParams {});
        
        let response = self.send_request(server_id, request).await?;
        
        match response {
            McpResponse::Pong(result) => {
                
                let mut connections = self.connections.write().await;
                if let Some(conn) = connections.get_mut(server_id) {
                    conn.last_ping = Some(Utc::now());
                }
                Ok(result)
            }
            McpResponse::Error(error) => Err(McpClientError::ProtocolError(error)),
            _ => Err(McpClientError::InvalidResponse("Expected pong response".to_string())),
        }
    }

    
    async fn send_request(
        &self,
        server_id: &ServerId,
        request: McpRequest,
    ) -> Result<McpResponse, McpClientError> {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.config.max_retries {
            match self.send_request_once(server_id, &request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    attempts += 1;
                    
                    if attempts < self.config.max_retries {
                        log::warn!("Request attempt {} failed, retrying in {:?}", attempts, self.config.retry_delay);
                        tokio::time::sleep(self.config.retry_delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| McpClientError::NetworkError("Unknown error".to_string())))
    }

    
    async fn send_request_once(
        &self,
        server_id: &ServerId,
        request: &McpRequest,
    ) -> Result<McpResponse, McpClientError> {
        let (endpoint, auth_config) = {
            let connections = self.connections.read().await;
            let connection = connections.get(server_id)
                .ok_or_else(|| McpClientError::ConnectionError("Connection not found".to_string()))?;
            
            
            match &connection.status {
                ConnectionStatus::Connected => {},
                ConnectionStatus::Disconnected => {
                    return Err(McpClientError::ConnectionError("Connection is disconnected".to_string()));
                }
                ConnectionStatus::Error(msg) => {
                    return Err(McpClientError::ConnectionError(format!("Connection error: {}", msg)));
                }
                ConnectionStatus::Timeout => {
                    return Err(McpClientError::TimeoutError);
                }
                ConnectionStatus::Connecting => {
                    return Err(McpClientError::ConnectionError("Connection is still connecting".to_string()));
                }
            }
            
            (connection.endpoint.clone(), connection.auth_config.clone())
        };

        
        let message = McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(Uuid::new_v4().to_string())),
            method: Some(self.get_method_name(request)),
            params: Some(serde_json::to_value(request).map_err(|e| {
                McpClientError::InvalidResponse(format!("Failed to serialize request: {}", e))
            })?),
            result: None,
            error: None,
        };

        
        let mut http_request = self.http_client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .json(&message);

        
        http_request = self.add_authentication(http_request, &auth_config)?;

        
        let response = http_request
            .send()
            .await
            .map_err(|e| McpClientError::NetworkError(e.to_string()))?;

        
        if !response.status().is_success() {
            return Err(McpClientError::ServerError(format!(
                "HTTP error: {}", 
                response.status()
            )));
        }

        
        let response_message: McpMessage = response
            .json()
            .await
            .map_err(|e| McpClientError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        
        if let Some(error) = response_message.error {
            return Err(McpClientError::ProtocolError(error));
        }

        
        let result = response_message.result
            .ok_or_else(|| McpClientError::InvalidResponse("Missing result in response".to_string()))?;

        self.parse_response(request, result)
    }

    
    fn add_authentication(
        &self,
        mut request: reqwest::RequestBuilder,
        auth_config: &AuthConfig,
    ) -> Result<reqwest::RequestBuilder, McpClientError> {
        match &auth_config.method {
            AuthMethod::ApiKey => {
                let api_key = auth_config.credentials.get("api_key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpClientError::AuthenticationError("Missing API key".to_string()))?;
                
                request = request.header("X-API-Key", api_key);
            }
            AuthMethod::Bearer => {
                let token = auth_config.credentials.get("token")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpClientError::AuthenticationError("Missing bearer token".to_string()))?;
                
                request = request.header("Authorization", format!("Bearer {}", token));
            }
            AuthMethod::OAuth2 => {
                let access_token = auth_config.credentials.get("access_token")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpClientError::AuthenticationError("Missing OAuth2 access token".to_string()))?;
                
                request = request.header("Authorization", format!("Bearer {}", access_token));
            }
            AuthMethod::Certificate => {
                
                return Err(McpClientError::AuthenticationError("Certificate authentication not implemented".to_string()));
            }
            AuthMethod::Custom(_) => {
                
                return Err(McpClientError::AuthenticationError("Custom authentication not implemented".to_string()));
            }
        }

        Ok(request)
    }

    
    fn get_method_name(&self, request: &McpRequest) -> String {
        match request {
            McpRequest::Initialize(_) => "initialize".to_string(),
            McpRequest::ResourcesList(_) => "resources/list".to_string(),
            McpRequest::ResourcesRead(_) => "resources/read".to_string(),
            McpRequest::ToolsList(_) => "tools/list".to_string(),
            McpRequest::ToolsCall(_) => "tools/call".to_string(),
            McpRequest::ContextCreate(_) => "contexts/create".to_string(),
            McpRequest::ContextGet(_) => "contexts/get".to_string(),
            McpRequest::Ping(_) => "ping".to_string(),
        }
    }

    
    fn parse_response(
        &self,
        request: &McpRequest,
        result: serde_json::Value,
    ) -> Result<McpResponse, McpClientError> {
        match request {
            McpRequest::Initialize(_) => {
                let init_result: InitializeResult = serde_json::from_value(result)
                    .map_err(|e| McpClientError::InvalidResponse(e.to_string()))?;
                Ok(McpResponse::Initialize(init_result))
            }
            McpRequest::ResourcesList(_) => {
                let list_result: ResourcesListResult = serde_json::from_value(result)
                    .map_err(|e| McpClientError::InvalidResponse(e.to_string()))?;
                Ok(McpResponse::ResourcesList(list_result))
            }
            McpRequest::ResourcesRead(_) => {
                let read_result: ResourcesReadResult = serde_json::from_value(result)
                    .map_err(|e| McpClientError::InvalidResponse(e.to_string()))?;
                Ok(McpResponse::ResourcesRead(read_result))
            }
            McpRequest::ToolsList(_) => {
                let tools_result: ToolsListResult = serde_json::from_value(result)
                    .map_err(|e| McpClientError::InvalidResponse(e.to_string()))?;
                Ok(McpResponse::ToolsList(tools_result))
            }
            McpRequest::ToolsCall(_) => {
                let call_result: ToolsCallResult = serde_json::from_value(result)
                    .map_err(|e| McpClientError::InvalidResponse(e.to_string()))?;
                Ok(McpResponse::ToolsCall(call_result))
            }
            McpRequest::ContextCreate(_) => {
                let create_result: ContextCreateResult = serde_json::from_value(result)
                    .map_err(|e| McpClientError::InvalidResponse(e.to_string()))?;
                Ok(McpResponse::ContextCreate(create_result))
            }
            McpRequest::ContextGet(_) => {
                let get_result: ContextGetResult = serde_json::from_value(result)
                    .map_err(|e| McpClientError::InvalidResponse(e.to_string()))?;
                Ok(McpResponse::ContextGet(get_result))
            }
            McpRequest::Ping(_) => {
                let pong_result: PongResult = serde_json::from_value(result)
                    .map_err(|e| McpClientError::InvalidResponse(e.to_string()))?;
                Ok(McpResponse::Pong(pong_result))
            }
        }
    }

    
    #[allow(dead_code)]
    pub async fn get_connection_status(&self, server_id: &ServerId) -> Option<ConnectionStatus> {
        let connections = self.connections.read().await;
        connections.get(server_id).map(|conn| conn.status.clone())
    }

    
    #[allow(dead_code)]
    pub async fn list_connections(&self) -> Vec<McpConnection> {
        let connections = self.connections.read().await;
        connections.values().cloned().collect()
    }

    
    #[allow(dead_code)]
    pub async fn start_health_monitoring(&self) {
        let connections = Arc::clone(&self.connections);
        let heartbeat_interval = self.config.heartbeat_interval;
        let client = self.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(heartbeat_interval);
            
            loop {
                interval.tick().await;
                
                let server_ids: Vec<ServerId> = {
                    let conns = connections.read().await;
                    conns.keys().cloned().collect()
                };

                for server_id in server_ids {
                    if let Err(e) = client.ping(&server_id).await {
                        log::warn!("Health check failed for server {}: {}", server_id, e);
                        
                        
                        let mut conns = connections.write().await;
                        if let Some(conn) = conns.get_mut(&server_id) {
                            conn.error_count += 1;
                            if conn.error_count > 3 {
                                conn.status = ConnectionStatus::Error("Health check failures".to_string());
                            }
                        }
                    }
                }
            }
        });
    }
}


impl AuthConfig {
    pub fn api_key(api_key: String) -> Self {
        Self {
            method: AuthMethod::ApiKey,
            credentials: json!({ "api_key": api_key }),
        }
    }

    pub fn bearer(token: String) -> Self {
        Self {
            method: AuthMethod::Bearer,
            credentials: json!({ "token": token }),
        }
    }

    pub fn oauth2(access_token: String, refresh_token: Option<String>) -> Self {
        let mut credentials = json!({ "access_token": access_token });
        if let Some(refresh_token) = refresh_token {
            credentials["refresh_token"] = json!(refresh_token);
        }
        
        Self {
            method: AuthMethod::OAuth2,
            credentials,
        }
    }
}