use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Model Context Protocol (MCP) Core Types
/// 
/// This module defines the core types and structures for implementing
/// the Model Context Protocol in ConHub, enabling standardized context
/// sharing between AI agents and various data sources.

// ============================================================================
// Core Protocol Types
// ============================================================================

/// MCP Protocol version
pub const MCP_VERSION: &str = "2024-11-05";

/// Unique identifier for MCP resources
pub type ResourceId = String;

/// Unique identifier for MCP contexts
pub type ContextId = String;

/// Unique identifier for MCP tools
pub type ToolId = String;

/// Unique identifier for MCP servers
pub type ServerId = String;

// ============================================================================
// Resource Management
// ============================================================================

/// Represents a resource that can be accessed through MCP
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

/// Annotations for MCP resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAnnotations {
    pub audience: Option<Vec<String>>,
    pub priority: Option<f32>,
    pub tags: Vec<String>,
    pub source_type: String,
    pub confidence: Option<f32>,
}

/// Access permissions for resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessPermission {
    Read,
    Write,
    Execute,
    Admin,
    ContextProvider,
}

// ============================================================================
// Context Management
// ============================================================================

/// Represents contextual information in MCP format
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

/// Types of contexts available in MCP
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

/// Resource reference within a context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextResource {
    pub resource_id: ResourceId,
    pub relevance_score: Option<f32>,
    pub content: Option<String>,
    pub content_type: Option<String>,
    pub annotations: Option<ResourceAnnotations>,
}

/// Access levels for contexts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessLevel {
    Public,
    Internal,
    Restricted,
    Private,
}

// ============================================================================
// Tool Definitions
// ============================================================================

/// MCP Tool definition
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

/// Schema definition for MCP tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub error_schema: Option<serde_json::Value>,
}

/// Capabilities that a tool provides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolCapability {
    ContextRetrieval,
    ResourceAccess,
    DataTransformation,
    Search,
    Analysis,
    Custom(String),
}

/// Security requirements for tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityRequirement {
    Authentication,
    Authorization,
    Encryption,
    Audit,
    RateLimiting,
}

// ============================================================================
// Server and Client Definitions
// ============================================================================

/// MCP Server configuration
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

/// Capabilities exposed by an MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub resources: Option<ResourceCapabilities>,
    pub tools: Option<ToolCapabilities>,
    pub prompts: Option<PromptCapabilities>,
    pub logging: Option<LoggingCapabilities>,
}

/// Resource-specific server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCapabilities {
    pub subscribe: bool,
    pub list_changed: bool,
}

/// Tool-specific server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapabilities {
    pub list_changed: bool,
}

/// Prompt-specific server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCapabilities {
    pub list_changed: bool,
}

/// Logging-specific server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingCapabilities {
    pub level: LogLevel,
}

/// Server endpoint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEndpoint {
    pub path: String,
    pub method: HttpMethod,
    pub description: String,
    pub schema: Option<serde_json::Value>,
}

/// HTTP methods supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

/// Server security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSecurity {
    pub authentication_required: bool,
    pub supported_auth_methods: Vec<AuthMethod>,
    pub rate_limiting: Option<RateLimitConfig>,
    pub encryption: EncryptionConfig,
}

/// Authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    ApiKey,
    Bearer,
    OAuth2,
    Certificate,
    Custom(String),
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: Option<u32>,
    pub per_client: bool,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub tls_required: bool,
    pub min_tls_version: String,
    pub supported_ciphers: Vec<String>,
}

/// Server status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerStatus {
    Starting,
    Ready,
    Busy,
    Error,
    Maintenance,
    Stopped,
}

/// Log levels for MCP operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

// ============================================================================
// Protocol Messages
// ============================================================================

/// Base MCP protocol message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpMessage {
    pub jsonrpc: String, // Always "2.0"
    pub id: Option<serde_json::Value>,
    pub method: Option<String>,
    pub params: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub error: Option<McpError>,
}

/// MCP protocol error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// MCP request types
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

/// MCP response types
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

// ============================================================================
// Protocol Parameters and Results
// ============================================================================

/// Initialize request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    pub client_info: ClientInfo,
}

/// Client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    pub experimental: Option<HashMap<String, serde_json::Value>>,
    pub sampling: Option<serde_json::Value>,
}

/// Client information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// Initialize response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    pub server_info: ServerInfo,
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

/// Resources list parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesListParams {
    pub cursor: Option<String>,
}

/// Resources list result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesListResult {
    pub resources: Vec<McpResource>,
    pub next_cursor: Option<String>,
}

/// Resources read parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesReadParams {
    pub uri: String,
}

/// Resources read result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesReadResult {
    pub contents: Vec<ResourceContent>,
}

/// Resource content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    pub uri: String,
    pub mime_type: Option<String>,
    pub text: Option<String>,
    pub blob: Option<String>,
}

/// Tools list parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsListParams {
    pub cursor: Option<String>,
}

/// Tools list result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsListResult {
    pub tools: Vec<McpTool>,
    pub next_cursor: Option<String>,
}

/// Tools call parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCallParams {
    pub name: String,
    pub arguments: Option<serde_json::Value>,
}

/// Tools call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCallResult {
    pub content: Vec<ToolContent>,
    pub is_error: Option<bool>,
}

/// Tool content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContent {
    pub content_type: String,
    pub text: Option<String>,
    pub annotations: Option<serde_json::Value>,
}

/// Context create parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextCreateParams {
    pub name: String,
    pub context_type: ContextType,
    pub resources: Vec<ResourceId>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Context create result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextCreateResult {
    pub context: McpContext,
}

/// Context get parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextGetParams {
    pub context_id: ContextId,
}

/// Context get result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextGetResult {
    pub context: McpContext,
}

/// Ping parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingParams {}

/// Pong result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PongResult {
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// Trait Definitions
// ============================================================================

/// Trait for MCP context providers
pub trait McpContextProvider: Send + Sync {
    /// Get the provider's unique identifier
    fn provider_id(&self) -> String;
    
    /// Get the types of contexts this provider can handle
    #[allow(dead_code)]
    fn supported_context_types(&self) -> Vec<ContextType>;
    
    /// Create a context from available resources
    #[allow(dead_code)]
    async fn create_context(
        &self,
        context_type: ContextType,
        resource_ids: Vec<ResourceId>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<McpContext, McpError>;
    
    /// Get an existing context
    #[allow(dead_code)]
    async fn get_context(&self, context_id: &ContextId) -> Result<McpContext, McpError>;
    
    /// List available resources for this provider
    #[allow(dead_code)]
    async fn list_resources(&self) -> Result<Vec<McpResource>, McpError>;
    
    /// Read content from a specific resource
    #[allow(dead_code)]
    async fn read_resource(&self, resource_id: &ResourceId) -> Result<ResourceContent, McpError>;
}

/// Trait for MCP tool providers
pub trait McpToolProvider: Send + Sync {
    /// Get the provider's unique identifier
    fn provider_id(&self) -> String;
    
    /// List available tools
    #[allow(dead_code)]
    async fn list_tools(&self) -> Result<Vec<McpTool>, McpError>;
    
    /// Execute a tool with given parameters
    #[allow(dead_code)]
    async fn call_tool(
        &self,
        tool_name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<ToolsCallResult, McpError>;
}

/// Trait for MCP servers
pub trait McpServerTrait: Send + Sync {
    /// Get server information
    fn server_info(&self) -> ServerInfo;
    
    /// Get server capabilities
    fn capabilities(&self) -> ServerCapabilities;
    
    /// Handle an MCP request
    #[allow(dead_code)]
    async fn handle_request(&self, request: McpRequest) -> McpResponse;
    
    /// Register a context provider
    #[allow(dead_code)]
    fn register_context_provider(&mut self, provider: ContextProviderWrapper);
    
    /// Register a tool provider
    #[allow(dead_code)]
    fn register_tool_provider(&mut self, provider: ToolProviderWrapper);
}

// ============================================================================
// Error Handling
// ============================================================================

/// Standard MCP error codes
pub mod error_codes {
    #[allow(dead_code)]
    pub const PARSE_ERROR: i32 = -32700;
    #[allow(dead_code)]
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
    
    // MCP-specific error codes
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

/// Enum wrapper for context providers to avoid trait object issues
#[derive(Debug, Clone)]
pub enum ContextProviderWrapper {
    Repository(crate::services::mcp_server::RepositoryContextProvider),
    Document(crate::services::mcp_server::DocumentContextProvider),
    Url(crate::services::mcp_server::UrlContextProvider),
    DataSource(crate::services::mcp_server::DataSourceContextProvider),
}

impl ContextProviderWrapper {
    pub fn provider_id(&self) -> String {
        match self {
            Self::Repository(provider) => provider.provider_id(),
            Self::Document(provider) => provider.provider_id(),
            Self::Url(provider) => provider.provider_id(),
            Self::DataSource(provider) => provider.provider_id(),
        }
    }
    
    #[allow(dead_code)]
    pub async fn create_context(&self, context_type: ContextType, resource_ids: Vec<ResourceId>, metadata: Option<HashMap<String, serde_json::Value>>) -> Result<McpContext, McpError> {
        match self {
            Self::Repository(provider) => provider.create_context(context_type, resource_ids, metadata).await,
            Self::Document(provider) => provider.create_context(context_type, resource_ids, metadata).await,
            Self::Url(provider) => provider.create_context(context_type, resource_ids, metadata).await,
            Self::DataSource(provider) => provider.create_context(context_type, resource_ids, metadata).await,
        }
    }
    
    #[allow(dead_code)]
    pub async fn get_context(&self, context_id: &ContextId) -> Result<McpContext, McpError> {
        match self {
            Self::Repository(provider) => provider.get_context(context_id).await,
            Self::Document(provider) => provider.get_context(context_id).await,
            Self::Url(provider) => provider.get_context(context_id).await,
            Self::DataSource(provider) => provider.get_context(context_id).await,
        }
    }
    
    #[allow(dead_code)]
    pub async fn list_resources(&self) -> Result<Vec<McpResource>, McpError> {
        match self {
            Self::Repository(provider) => provider.list_resources().await,
            Self::Document(provider) => provider.list_resources().await,
            Self::Url(provider) => provider.list_resources().await,
            Self::DataSource(provider) => provider.list_resources().await,
        }
    }
    
    #[allow(dead_code)]
    pub async fn read_resource(&self, resource_id: &ResourceId) -> Result<ResourceContent, McpError> {
        match self {
            Self::Repository(provider) => provider.read_resource(resource_id).await,
            Self::Document(provider) => provider.read_resource(resource_id).await,
            Self::Url(provider) => provider.read_resource(resource_id).await,
            Self::DataSource(provider) => provider.read_resource(resource_id).await,
        }
    }
}

/// Enum wrapper for tool providers to avoid trait object issues
#[derive(Debug, Clone)]
pub enum ToolProviderWrapper {
    Search(crate::services::mcp_server::SearchToolProvider),
    Analysis(crate::services::mcp_server::AnalysisToolProvider),
}

impl ToolProviderWrapper {
    pub fn provider_id(&self) -> String {
        match self {
            Self::Search(provider) => provider.provider_id(),
            Self::Analysis(provider) => provider.provider_id(),
        }
    }
    
    #[allow(dead_code)]
    pub async fn list_tools(&self) -> Result<Vec<McpTool>, McpError> {
        match self {
            Self::Search(provider) => provider.list_tools().await,
            Self::Analysis(provider) => provider.list_tools().await,
        }
    }
    
    #[allow(dead_code)]
    pub async fn call_tool(&self, tool_name: &str, arguments: Option<serde_json::Value>) -> Result<ToolsCallResult, McpError> {
        match self {
            Self::Search(provider) => provider.call_tool(tool_name, arguments).await,
            Self::Analysis(provider) => provider.call_tool(tool_name, arguments).await,
        }
    }
}