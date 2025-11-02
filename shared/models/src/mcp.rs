use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};












pub const MCP_VERSION: &str = "2024-11-05";


pub type ResourceId = String;


pub type ContextId = String;


pub type ToolId = String;


pub type ServerId = String;






#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub id: ResourceId,
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
    pub annotations: Option<ResourceAnnotations>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub access_permissions: Vec<AccessPermission>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAnnotations {
    pub audience: Option<Vec<String>>,
    pub priority: Option<f32>,
    pub tags: Vec<String>,
    pub source_type: String,
    pub confidence: Option<f32>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessPermission {
    Read,
    Write,
    Execute,
    Admin,
    ContextProvider,
}






#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpContext {
    pub id: ContextId,
    pub name: String,
    pub description: Option<String>,
    pub context_type: ContextType,
    pub resources: Vec<ContextResource>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub access_level: AccessLevel,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextType {
    Repository,
    Document,
    Url,
    DataSource,
    Agent,
    Conversation,
    Tool,
    Custom(String),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextResource {
    pub resource_id: ResourceId,
    pub relevance_score: Option<f32>,
    pub content: Option<String>,
    pub content_type: Option<String>,
    pub annotations: Option<ResourceAnnotations>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessLevel {
    Public,
    Internal,
    Restricted,
    Private,
}






#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub id: ToolId,
    pub name: String,
    pub description: String,
    pub version: String,
    pub schema: ToolSchema,
    pub capabilities: Vec<ToolCapability>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub security_requirements: Vec<SecurityRequirement>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub error_schema: Option<serde_json::Value>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolCapability {
    ContextRetrieval,
    ResourceAccess,
    DataTransformation,
    Search,
    Analysis,
    Custom(String),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityRequirement {
    Authentication,
    Authorization,
    Encryption,
    Audit,
    RateLimiting,
}






#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub id: ServerId,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub capabilities: ServerCapabilities,
    pub endpoints: Vec<ServerEndpoint>,
    pub security: ServerSecurity,
    pub metadata: HashMap<String, serde_json::Value>,
    pub status: ServerStatus,
    pub created_at: DateTime<Utc>,
    pub last_heartbeat: Option<DateTime<Utc>>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub resources: Option<ResourceCapabilities>,
    pub tools: Option<ToolCapabilities>,
    pub prompts: Option<PromptCapabilities>,
    pub logging: Option<LoggingCapabilities>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCapabilities {
    pub subscribe: bool,
    pub list_changed: bool,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapabilities {
    pub list_changed: bool,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCapabilities {
    pub list_changed: bool,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingCapabilities {
    pub level: LogLevel,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEndpoint {
    pub path: String,
    pub method: HttpMethod,
    pub description: String,
    pub schema: Option<serde_json::Value>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSecurity {
    pub authentication_required: bool,
    pub supported_auth_methods: Vec<AuthMethod>,
    pub rate_limiting: Option<RateLimitConfig>,
    pub encryption: EncryptionConfig,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthMethod {
    ApiKey,
    Bearer,
    OAuth2,
    Certificate,
    Custom(String),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: Option<u32>,
    pub per_client: bool,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub tls_required: bool,
    pub min_tls_version: String,
    pub supported_ciphers: Vec<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerStatus {
    Starting,
    Ready,
    Busy,
    Error,
    Maintenance,
    Stopped,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}






#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpMessage {
    pub jsonrpc: String, 
    pub id: Option<serde_json::Value>,
    pub method: Option<String>,
    pub params: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub error: Option<McpError>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum McpRequest {
    Initialize(InitializeParams),
    ResourcesList(ResourcesListParams),
    ResourcesRead(ResourcesReadParams),
    ToolsList(ToolsListParams),
    ToolsCall(ToolsCallParams),
    ContextCreate(ContextCreateParams),
    ContextGet(ContextGetParams),
    Ping(PingParams),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpResponse {
    Initialize(InitializeResult),
    ResourcesList(ResourcesListResult),
    ResourcesRead(ResourcesReadResult),
    ToolsList(ToolsListResult),
    ToolsCall(ToolsCallResult),
    ContextCreate(ContextCreateResult),
    ContextGet(ContextGetResult),
    Pong(PongResult),
    Error(McpError),
}






#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    pub client_info: ClientInfo,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    pub experimental: Option<HashMap<String, serde_json::Value>>,
    pub sampling: Option<serde_json::Value>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    pub server_info: ServerInfo,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesListParams {
    pub cursor: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesListResult {
    pub resources: Vec<McpResource>,
    pub next_cursor: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesReadParams {
    pub uri: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesReadResult {
    pub contents: Vec<ResourceContent>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    pub uri: String,
    pub mime_type: Option<String>,
    pub text: Option<String>,
    pub blob: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsListParams {
    pub cursor: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsListResult {
    pub tools: Vec<McpTool>,
    pub next_cursor: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCallParams {
    pub name: String,
    pub arguments: Option<serde_json::Value>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCallResult {
    pub content: Vec<ToolContent>,
    pub is_error: Option<bool>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContent {
    pub content_type: String,
    pub text: Option<String>,
    pub annotations: Option<serde_json::Value>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextCreateParams {
    pub name: String,
    pub context_type: ContextType,
    pub resources: Vec<ResourceId>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextCreateResult {
    pub context: McpContext,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextGetParams {
    pub context_id: ContextId,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextGetResult {
    pub context: McpContext,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingParams {}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PongResult {
    pub timestamp: DateTime<Utc>,
}






pub trait McpContextProvider: Send + Sync {
    
    fn provider_id(&self) -> String;
    
    
    #[allow(dead_code)]
    fn supported_context_types(&self) -> Vec<ContextType>;
    
    
    #[allow(dead_code)]
    async fn create_context(
        &self,
        context_type: ContextType,
        resource_ids: Vec<ResourceId>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<McpContext, McpError>;
    
    
    #[allow(dead_code)]
    async fn get_context(&self, context_id: &ContextId) -> Result<McpContext, McpError>;
    
    
    #[allow(dead_code)]
    async fn list_resources(&self) -> Result<Vec<McpResource>, McpError>;
    
    
    #[allow(dead_code)]
    async fn read_resource(&self, resource_id: &ResourceId) -> Result<ResourceContent, McpError>;
}


pub trait McpToolProvider: Send + Sync {
    
    fn provider_id(&self) -> String;
    
    
    #[allow(dead_code)]
    async fn list_tools(&self) -> Result<Vec<McpTool>, McpError>;
    
    
    #[allow(dead_code)]
    async fn call_tool(
        &self,
        tool_name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<ToolsCallResult, McpError>;
}


pub trait McpServerTrait: Send + Sync {
    
    fn server_info(&self) -> ServerInfo;
    
    
    fn capabilities(&self) -> ServerCapabilities;
    
    
    #[allow(dead_code)]
    async fn handle_request(&self, request: McpRequest) -> McpResponse;
}






pub mod error_codes {
    #[allow(dead_code)]
    pub const PARSE_ERROR: i32 = -32700;
    #[allow(dead_code)]
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
    
    
    pub const RESOURCE_NOT_FOUND: i32 = -32001;
    pub const TOOL_NOT_FOUND: i32 = -32002;
    pub const CONTEXT_NOT_FOUND: i32 = -32003;
    #[allow(dead_code)]
    pub const ACCESS_DENIED: i32 = -32004;
    #[allow(dead_code)]
    pub const RATE_LIMITED: i32 = -32005;
    #[allow(dead_code)]
    pub const SERVER_UNAVAILABLE: i32 = -32006;
}

impl McpError {
    pub fn new(code: i32, message: String) -> Self {
        Self {
            code,
            message,
            data: None,
        }
    }
    
    #[allow(dead_code)]
    pub fn with_data(code: i32, message: String, data: serde_json::Value) -> Self {
        Self {
            code,
            message,
            data: Some(data),
        }
    }
    
    pub fn resource_not_found(resource_id: &str) -> Self {
        Self::new(
            error_codes::RESOURCE_NOT_FOUND,
            format!("Resource not found: {}", resource_id),
        )
    }
    
    pub fn tool_not_found(tool_name: &str) -> Self {
        Self::new(
            error_codes::TOOL_NOT_FOUND,
            format!("Tool not found: {}", tool_name),
        )
    }
    
    #[allow(dead_code)]
    pub fn context_not_found(context_id: &str) -> Self {
        Self::new(
            error_codes::CONTEXT_NOT_FOUND,
            format!("Context not found: {}", context_id),
        )
    }
    
    #[allow(dead_code)]
    pub fn access_denied(reason: &str) -> Self {
        Self::new(
            error_codes::ACCESS_DENIED,
            format!("Access denied: {}", reason),
        )
    }
    
    pub fn internal_error(message: &str) -> Self {
        Self::new(
            error_codes::INTERNAL_ERROR,
            format!("Internal error: {}", message),
        )
    }
}

impl std::fmt::Display for McpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for McpError {}
