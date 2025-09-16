use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::Utc;
use serde_json::json;
use tokio::sync::RwLock as AsyncRwLock;

use crate::models::mcp::*;

/// MCP Server implementation for ConHub
/// 
/// This server provides a standardized Model Context Protocol interface
/// for AI agents to access and interact with various data sources including
/// repositories, documents, URLs, and other contextual information.
#[derive(Clone)]
pub struct ConHubMcpServer {
    server_info: ServerInfo,
    capabilities: ServerCapabilities,
    context_providers: Arc<RwLock<HashMap<String, Box<dyn McpContextProvider>>>>,
    tool_providers: Arc<RwLock<HashMap<String, Box<dyn McpToolProvider>>>>,
    contexts: Arc<AsyncRwLock<HashMap<ContextId, McpContext>>>,
    resources: Arc<AsyncRwLock<HashMap<ResourceId, McpResource>>>,
    security_config: ServerSecurity,
}

impl ConHubMcpServer {
    /// Create a new ConHub MCP server instance
    pub fn new() -> Self {
        let server_info = ServerInfo {
            name: "ConHub MCP Server".to_string(),
            version: "1.0.0".to_string(),
        };

        let capabilities = ServerCapabilities {
            resources: Some(ResourceCapabilities {
                subscribe: true,
                list_changed: true,
            }),
            tools: Some(ToolCapabilities {
                list_changed: true,
            }),
            prompts: Some(PromptCapabilities {
                list_changed: true,
            }),
            logging: Some(LoggingCapabilities {
                level: LogLevel::Info,
            }),
        };

        let security_config = ServerSecurity {
            authentication_required: true,
            supported_auth_methods: vec![
                AuthMethod::ApiKey,
                AuthMethod::Bearer,
            ],
            rate_limiting: Some(RateLimitConfig {
                requests_per_minute: 1000,
                burst_size: Some(100),
                per_client: true,
            }),
            encryption: EncryptionConfig {
                tls_required: true,
                min_tls_version: "1.2".to_string(),
                supported_ciphers: vec![
                    "TLS_AES_256_GCM_SHA384".to_string(),
                    "TLS_CHACHA20_POLY1305_SHA256".to_string(),
                ],
            },
        };

        Self {
            server_info,
            capabilities,
            context_providers: Arc::new(RwLock::new(HashMap::new())),
            tool_providers: Arc::new(RwLock::new(HashMap::new())),
            contexts: Arc::new(AsyncRwLock::new(HashMap::new())),
            resources: Arc::new(AsyncRwLock::new(HashMap::new())),
            security_config,
        }
    }

    /// Initialize the server with default context and tool providers
    pub async fn initialize(&mut self) -> Result<(), McpError> {
        // Register default context providers
        self.register_repository_provider().await?;
        self.register_document_provider().await?;
        self.register_url_provider().await?;
        self.register_datasource_provider().await?;

        // Register default tools
        self.register_search_tools().await?;
        self.register_analysis_tools().await?;

        Ok(())
    }

    /// Register repository context provider
    async fn register_repository_provider(&mut self) -> Result<(), McpError> {
        let provider = RepositoryContextProvider::new().await?;
        let provider_id = provider.provider_id();
        
        // Store the provider
        // Note: This is simplified - in a real implementation, we'd need to handle
        // the trait object storage more carefully
        log::info!("Registered repository context provider: {}", provider_id);
        Ok(())
    }

    /// Register document context provider
    async fn register_document_provider(&mut self) -> Result<(), McpError> {
        let provider = DocumentContextProvider::new().await?;
        let provider_id = provider.provider_id();
        
        log::info!("Registered document context provider: {}", provider_id);
        Ok(())
    }

    /// Register URL context provider
    async fn register_url_provider(&mut self) -> Result<(), McpError> {
        let provider = UrlContextProvider::new().await?;
        let provider_id = provider.provider_id();
        
        log::info!("Registered URL context provider: {}", provider_id);
        Ok(())
    }

    /// Register data source context provider
    async fn register_datasource_provider(&mut self) -> Result<(), McpError> {
        let provider = DataSourceContextProvider::new().await?;
        let provider_id = provider.provider_id();
        
        log::info!("Registered data source context provider: {}", provider_id);
        Ok(())
    }

    /// Register search tools
    async fn register_search_tools(&mut self) -> Result<(), McpError> {
        let provider = SearchToolProvider::new().await?;
        let provider_id = provider.provider_id();
        
        log::info!("Registered search tool provider: {}", provider_id);
        Ok(())
    }

    /// Register analysis tools
    async fn register_analysis_tools(&mut self) -> Result<(), McpError> {
        let provider = AnalysisToolProvider::new().await?;
        let provider_id = provider.provider_id();
        
        log::info!("Registered analysis tool provider: {}", provider_id);
        Ok(())
    }

    /// Handle MCP protocol messages
    pub async fn handle_message(&self, message: McpMessage) -> McpMessage {
        match self.process_request(message).await {
            Ok(response) => response,
            Err(error) => McpMessage {
                jsonrpc: "2.0".to_string(),
                id: None,
                method: None,
                params: None,
                result: None,
                error: Some(error),
            },
        }
    }

    /// Process incoming MCP requests
    async fn process_request(&self, message: McpMessage) -> Result<McpMessage, McpError> {
        let method = message.method.as_ref()
            .ok_or_else(|| McpError::new(error_codes::INVALID_REQUEST, "Missing method".to_string()))?;

        let response_result = match method.as_str() {
            "initialize" => self.handle_initialize(message.params).await?,
            "resources/list" => self.handle_resources_list(message.params).await?,
            "resources/read" => self.handle_resources_read(message.params).await?,
            "tools/list" => self.handle_tools_list(message.params).await?,
            "tools/call" => self.handle_tools_call(message.params).await?,
            "contexts/create" => self.handle_context_create(message.params).await?,
            "contexts/get" => self.handle_context_get(message.params).await?,
            "ping" => self.handle_ping(message.params).await?,
            _ => return Err(McpError::new(
                error_codes::METHOD_NOT_FOUND,
                format!("Method not found: {}", method),
            )),
        };

        Ok(McpMessage {
            jsonrpc: "2.0".to_string(),
            id: message.id,
            method: None,
            params: None,
            result: Some(response_result),
            error: None,
        })
    }

    /// Handle initialize request
    async fn handle_initialize(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value, McpError> {
        let _params: InitializeParams = match params {
            Some(p) => serde_json::from_value(p)
                .map_err(|e| McpError::new(error_codes::INVALID_PARAMS, e.to_string()))?,
            None => return Err(McpError::new(error_codes::INVALID_PARAMS, "Missing initialize params".to_string())),
        };

        let result = InitializeResult {
            protocol_version: MCP_VERSION.to_string(),
            capabilities: self.capabilities.clone(),
            server_info: self.server_info.clone(),
        };

        serde_json::to_value(result)
            .map_err(|e| McpError::internal_error(&e.to_string()))
    }

    /// Handle resources list request
    async fn handle_resources_list(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value, McpError> {
        let _params: ResourcesListParams = match params {
            Some(p) => serde_json::from_value(p)
                .map_err(|e| McpError::new(error_codes::INVALID_PARAMS, e.to_string()))?,
            None => ResourcesListParams { cursor: None },
        };

        // Collect resources from all providers
        let mut all_resources = Vec::new();
        
        // For now, return a simple list - in production, implement proper pagination
        let resources = self.resources.read().await;
        for resource in resources.values() {
            all_resources.push(resource.clone());
        }

        let result = ResourcesListResult {
            resources: all_resources,
            next_cursor: None,
        };

        serde_json::to_value(result)
            .map_err(|e| McpError::internal_error(&e.to_string()))
    }

    /// Handle resources read request
    async fn handle_resources_read(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value, McpError> {
        let params: ResourcesReadParams = match params {
            Some(p) => serde_json::from_value(p)
                .map_err(|e| McpError::new(error_codes::INVALID_PARAMS, e.to_string()))?,
            None => return Err(McpError::new(error_codes::INVALID_PARAMS, "Missing read params".to_string())),
        };

        // Find the resource by URI
        let resources = self.resources.read().await;
        let resource = resources.values()
            .find(|r| r.uri == params.uri)
            .ok_or_else(|| McpError::resource_not_found(&params.uri))?;

        // Create resource content based on the resource type
        let content = ResourceContent {
            uri: resource.uri.clone(),
            mime_type: resource.mime_type.clone(),
            text: Some(format!("Content for resource: {}", resource.name)),
            blob: None,
        };

        let result = ResourcesReadResult {
            contents: vec![content],
        };

        serde_json::to_value(result)
            .map_err(|e| McpError::internal_error(&e.to_string()))
    }

    /// Handle tools list request
    async fn handle_tools_list(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value, McpError> {
        let _params: ToolsListParams = match params {
            Some(p) => serde_json::from_value(p)
                .map_err(|e| McpError::new(error_codes::INVALID_PARAMS, e.to_string()))?,
            None => ToolsListParams { cursor: None },
        };

        // Return available tools
        let tools = vec![
            self.create_search_tool(),
            self.create_analysis_tool(),
            self.create_context_tool(),
        ];

        let result = ToolsListResult {
            tools,
            next_cursor: None,
        };

        serde_json::to_value(result)
            .map_err(|e| McpError::internal_error(&e.to_string()))
    }

    /// Handle tools call request
    async fn handle_tools_call(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value, McpError> {
        let params: ToolsCallParams = match params {
            Some(p) => serde_json::from_value(p)
                .map_err(|e| McpError::new(error_codes::INVALID_PARAMS, e.to_string()))?,
            None => return Err(McpError::new(error_codes::INVALID_PARAMS, "Missing tool call params".to_string())),
        };

        let result = match params.name.as_str() {
            "search" => self.execute_search_tool(params.arguments).await?,
            "analyze" => self.execute_analysis_tool(params.arguments).await?,
            "create_context" => self.execute_context_tool(params.arguments).await?,
            _ => return Err(McpError::tool_not_found(&params.name)),
        };

        serde_json::to_value(result)
            .map_err(|e| McpError::internal_error(&e.to_string()))
    }

    /// Handle context create request
    async fn handle_context_create(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value, McpError> {
        let params: ContextCreateParams = match params {
            Some(p) => serde_json::from_value(p)
                .map_err(|e| McpError::new(error_codes::INVALID_PARAMS, e.to_string()))?,
            None => return Err(McpError::new(error_codes::INVALID_PARAMS, "Missing context create params".to_string())),
        };

        let context_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let context = McpContext {
            id: context_id.clone(),
            name: params.name,
            description: None,
            context_type: params.context_type,
            resources: params.resources.into_iter().map(|resource_id| {
                ContextResource {
                    resource_id,
                    relevance_score: Some(1.0),
                    content: None,
                    content_type: None,
                    annotations: None,
                }
            }).collect(),
            metadata: params.metadata.unwrap_or_default(),
            created_at: now,
            expires_at: None,
            access_level: AccessLevel::Internal,
        };

        // Store the context
        {
            let mut contexts = self.contexts.write().await;
            contexts.insert(context_id, context.clone());
        }

        let result = ContextCreateResult { context };

        serde_json::to_value(result)
            .map_err(|e| McpError::internal_error(&e.to_string()))
    }

    /// Handle context get request
    async fn handle_context_get(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value, McpError> {
        let params: ContextGetParams = match params {
            Some(p) => serde_json::from_value(p)
                .map_err(|e| McpError::new(error_codes::INVALID_PARAMS, e.to_string()))?,
            None => return Err(McpError::new(error_codes::INVALID_PARAMS, "Missing context get params".to_string())),
        };

        let contexts = self.contexts.read().await;
        let context = contexts.get(&params.context_id)
            .ok_or_else(|| McpError::context_not_found(&params.context_id))?
            .clone();

        let result = ContextGetResult { context };

        serde_json::to_value(result)
            .map_err(|e| McpError::internal_error(&e.to_string()))
    }

    /// Handle ping request
    async fn handle_ping(&self, _params: Option<serde_json::Value>) -> Result<serde_json::Value, McpError> {
        let result = PongResult {
            timestamp: Utc::now(),
        };

        serde_json::to_value(result)
            .map_err(|e| McpError::internal_error(&e.to_string()))
    }

    /// Create search tool definition
    fn create_search_tool(&self) -> McpTool {
        McpTool {
            id: "search".to_string(),
            name: "Search".to_string(),
            description: "Search across repositories, documents, and URLs".to_string(),
            version: "1.0.0".to_string(),
            schema: ToolSchema {
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {"type": "string"},
                        "sources": {
                            "type": "array",
                            "items": {"type": "string", "enum": ["repositories", "documents", "urls"]}
                        }
                    },
                    "required": ["query"]
                }),
                output_schema: json!({
                    "type": "object",
                    "properties": {
                        "results": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "id": {"type": "string"},
                                    "title": {"type": "string"},
                                    "content": {"type": "string"},
                                    "source": {"type": "string"},
                                    "relevance": {"type": "number"}
                                }
                            }
                        }
                    }
                }),
                error_schema: None,
            },
            capabilities: vec![ToolCapability::Search],
            metadata: HashMap::new(),
            security_requirements: vec![SecurityRequirement::Authentication],
        }
    }

    /// Create analysis tool definition
    fn create_analysis_tool(&self) -> McpTool {
        McpTool {
            id: "analyze".to_string(),
            name: "Analyze".to_string(),
            description: "Analyze code, documents, or data patterns".to_string(),
            version: "1.0.0".to_string(),
            schema: ToolSchema {
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "content": {"type": "string"},
                        "analysis_type": {"type": "string", "enum": ["code", "document", "data"]}
                    },
                    "required": ["content", "analysis_type"]
                }),
                output_schema: json!({
                    "type": "object",
                    "properties": {
                        "analysis": {
                            "type": "object",
                            "properties": {
                                "summary": {"type": "string"},
                                "insights": {"type": "array", "items": {"type": "string"}},
                                "recommendations": {"type": "array", "items": {"type": "string"}}
                            }
                        }
                    }
                }),
                error_schema: None,
            },
            capabilities: vec![ToolCapability::Analysis],
            metadata: HashMap::new(),
            security_requirements: vec![SecurityRequirement::Authentication],
        }
    }

    /// Create context tool definition
    fn create_context_tool(&self) -> McpTool {
        McpTool {
            id: "create_context".to_string(),
            name: "Create Context".to_string(),
            description: "Create a new context from available resources".to_string(),
            version: "1.0.0".to_string(),
            schema: ToolSchema {
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "type": {"type": "string"},
                        "resources": {"type": "array", "items": {"type": "string"}}
                    },
                    "required": ["name", "type"]
                }),
                output_schema: json!({
                    "type": "object",
                    "properties": {
                        "context_id": {"type": "string"},
                        "message": {"type": "string"}
                    }
                }),
                error_schema: None,
            },
            capabilities: vec![ToolCapability::ContextRetrieval],
            metadata: HashMap::new(),
            security_requirements: vec![SecurityRequirement::Authentication],
        }
    }

    /// Execute search tool
    async fn execute_search_tool(&self, arguments: Option<serde_json::Value>) -> Result<ToolsCallResult, McpError> {
        let args = arguments.ok_or_else(|| McpError::new(error_codes::INVALID_PARAMS, "Missing search arguments".to_string()))?;
        
        let query = args.get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::new(error_codes::INVALID_PARAMS, "Missing query parameter".to_string()))?;

        // Simulate search results
        let content = ToolContent {
            content_type: "application/json".to_string(),
            text: Some(json!({
                "results": [
                    {
                        "id": "result1",
                        "title": format!("Search result for: {}", query),
                        "content": "Sample search result content",
                        "source": "repositories",
                        "relevance": 0.95
                    }
                ]
            }).to_string()),
            annotations: None,
        };

        Ok(ToolsCallResult {
            content: vec![content],
            is_error: Some(false),
        })
    }

    /// Execute analysis tool
    async fn execute_analysis_tool(&self, arguments: Option<serde_json::Value>) -> Result<ToolsCallResult, McpError> {
        let args = arguments.ok_or_else(|| McpError::new(error_codes::INVALID_PARAMS, "Missing analysis arguments".to_string()))?;
        
        let content = args.get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::new(error_codes::INVALID_PARAMS, "Missing content parameter".to_string()))?;

        let analysis_type = args.get("analysis_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::new(error_codes::INVALID_PARAMS, "Missing analysis_type parameter".to_string()))?;

        // Simulate analysis results
        let result_content = ToolContent {
            content_type: "application/json".to_string(),
            text: Some(json!({
                "analysis": {
                    "summary": format!("Analysis of {} content: {}", analysis_type, &content[..content.len().min(100)]),
                    "insights": [
                        "Content appears to be well-structured",
                        "No major issues detected"
                    ],
                    "recommendations": [
                        "Consider adding more documentation",
                        "Review for optimization opportunities"
                    ]
                }
            }).to_string()),
            annotations: None,
        };

        Ok(ToolsCallResult {
            content: vec![result_content],
            is_error: Some(false),
        })
    }

    /// Execute context creation tool
    async fn execute_context_tool(&self, arguments: Option<serde_json::Value>) -> Result<ToolsCallResult, McpError> {
        let args = arguments.ok_or_else(|| McpError::new(error_codes::INVALID_PARAMS, "Missing context arguments".to_string()))?;
        
        let name = args.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::new(error_codes::INVALID_PARAMS, "Missing name parameter".to_string()))?;

        let context_type_str = args.get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::new(error_codes::INVALID_PARAMS, "Missing type parameter".to_string()))?;

        let context_type = match context_type_str {
            "repository" => ContextType::Repository,
            "document" => ContextType::Document,
            "url" => ContextType::Url,
            "data_source" => ContextType::DataSource,
            other => ContextType::Custom(other.to_string()),
        };

        let resources = args.get("resources")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        // Create the context using the existing method
        let params = ContextCreateParams {
            name: name.to_string(),
            context_type,
            resources,
            metadata: None,
        };

        let create_result = self.handle_context_create(Some(serde_json::to_value(params).unwrap())).await?;
        let context_result: ContextCreateResult = serde_json::from_value(create_result)
            .map_err(|e| McpError::internal_error(&e.to_string()))?;

        let result_content = ToolContent {
            content_type: "application/json".to_string(),
            text: Some(json!({
                "context_id": context_result.context.id,
                "message": format!("Successfully created context: {}", context_result.context.name)
            }).to_string()),
            annotations: None,
        };

        Ok(ToolsCallResult {
            content: vec![result_content],
            is_error: Some(false),
        })
    }

    /// Add a resource to the server
    pub async fn add_resource(&self, resource: McpResource) {
        let mut resources = self.resources.write().await;
        resources.insert(resource.id.clone(), resource);
    }

    /// Get server security configuration
    pub fn security_config(&self) -> &ServerSecurity {
        &self.security_config
    }
}

impl McpServerTrait for ConHubMcpServer {
    fn server_info(&self) -> ServerInfo {
        self.server_info.clone()
    }

    fn capabilities(&self) -> ServerCapabilities {
        self.capabilities.clone()
    }

    async fn handle_request(&self, request: McpRequest) -> McpResponse {
        match request {
            McpRequest::Initialize(params) => {
                let result = InitializeResult {
                    protocol_version: MCP_VERSION.to_string(),
                    capabilities: self.capabilities.clone(),
                    server_info: self.server_info.clone(),
                };
                McpResponse::Initialize(result)
            }
            McpRequest::ResourcesList(_params) => {
                let resources = self.resources.read().await;
                let result = ResourcesListResult {
                    resources: resources.values().cloned().collect(),
                    next_cursor: None,
                };
                McpResponse::ResourcesList(result)
            }
            McpRequest::Ping(_) => {
                let result = PongResult {
                    timestamp: Utc::now(),
                };
                McpResponse::Pong(result)
            }
            _ => McpResponse::Error(McpError::new(
                error_codes::METHOD_NOT_FOUND,
                "Method not implemented".to_string(),
            )),
        }
    }

    fn register_context_provider(&mut self, _provider: Box<dyn McpContextProvider>) {
        // Implementation would store the provider
        // For now, this is a placeholder
    }

    fn register_tool_provider(&mut self, _provider: Box<dyn McpToolProvider>) {
        // Implementation would store the provider
        // For now, this is a placeholder
    }
}

// Placeholder context provider implementations
// These would be fully implemented with actual data access

struct RepositoryContextProvider;
impl RepositoryContextProvider {
    async fn new() -> Result<Self, McpError> {
        Ok(Self)
    }
}

impl McpContextProvider for RepositoryContextProvider {
    fn provider_id(&self) -> String {
        "repository_provider".to_string()
    }

    fn supported_context_types(&self) -> Vec<ContextType> {
        vec![ContextType::Repository]
    }

    async fn create_context(
        &self,
        _context_type: ContextType,
        _resource_ids: Vec<ResourceId>,
        _metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<McpContext, McpError> {
        // Implementation would create actual repository context
        Err(McpError::new(error_codes::INTERNAL_ERROR, "Not implemented".to_string()))
    }

    async fn get_context(&self, _context_id: &ContextId) -> Result<McpContext, McpError> {
        Err(McpError::new(error_codes::CONTEXT_NOT_FOUND, "Not implemented".to_string()))
    }

    async fn list_resources(&self) -> Result<Vec<McpResource>, McpError> {
        Ok(vec![])
    }

    async fn read_resource(&self, _resource_id: &ResourceId) -> Result<ResourceContent, McpError> {
        Err(McpError::new(error_codes::RESOURCE_NOT_FOUND, "Not implemented".to_string()))
    }
}

// Similar placeholder implementations for other providers
struct DocumentContextProvider;
impl DocumentContextProvider {
    async fn new() -> Result<Self, McpError> { Ok(Self) }
}
impl McpContextProvider for DocumentContextProvider {
    fn provider_id(&self) -> String { "document_provider".to_string() }
    fn supported_context_types(&self) -> Vec<ContextType> { vec![ContextType::Document] }
    async fn create_context(&self, _: ContextType, _: Vec<ResourceId>, _: Option<HashMap<String, serde_json::Value>>) -> Result<McpContext, McpError> { Err(McpError::new(error_codes::INTERNAL_ERROR, "Not implemented".to_string())) }
    async fn get_context(&self, _: &ContextId) -> Result<McpContext, McpError> { Err(McpError::new(error_codes::CONTEXT_NOT_FOUND, "Not implemented".to_string())) }
    async fn list_resources(&self) -> Result<Vec<McpResource>, McpError> { Ok(vec![]) }
    async fn read_resource(&self, _: &ResourceId) -> Result<ResourceContent, McpError> { Err(McpError::new(error_codes::RESOURCE_NOT_FOUND, "Not implemented".to_string())) }
}

struct UrlContextProvider;
impl UrlContextProvider {
    async fn new() -> Result<Self, McpError> { Ok(Self) }
}
impl McpContextProvider for UrlContextProvider {
    fn provider_id(&self) -> String { "url_provider".to_string() }
    fn supported_context_types(&self) -> Vec<ContextType> { vec![ContextType::Url] }
    async fn create_context(&self, _: ContextType, _: Vec<ResourceId>, _: Option<HashMap<String, serde_json::Value>>) -> Result<McpContext, McpError> { Err(McpError::new(error_codes::INTERNAL_ERROR, "Not implemented".to_string())) }
    async fn get_context(&self, _: &ContextId) -> Result<McpContext, McpError> { Err(McpError::new(error_codes::CONTEXT_NOT_FOUND, "Not implemented".to_string())) }
    async fn list_resources(&self) -> Result<Vec<McpResource>, McpError> { Ok(vec![]) }
    async fn read_resource(&self, _: &ResourceId) -> Result<ResourceContent, McpError> { Err(McpError::new(error_codes::RESOURCE_NOT_FOUND, "Not implemented".to_string())) }
}

struct DataSourceContextProvider;
impl DataSourceContextProvider {
    async fn new() -> Result<Self, McpError> { Ok(Self) }
}
impl McpContextProvider for DataSourceContextProvider {
    fn provider_id(&self) -> String { "datasource_provider".to_string() }
    fn supported_context_types(&self) -> Vec<ContextType> { vec![ContextType::DataSource] }
    async fn create_context(&self, _: ContextType, _: Vec<ResourceId>, _: Option<HashMap<String, serde_json::Value>>) -> Result<McpContext, McpError> { Err(McpError::new(error_codes::INTERNAL_ERROR, "Not implemented".to_string())) }
    async fn get_context(&self, _: &ContextId) -> Result<McpContext, McpError> { Err(McpError::new(error_codes::CONTEXT_NOT_FOUND, "Not implemented".to_string())) }
    async fn list_resources(&self) -> Result<Vec<McpResource>, McpError> { Ok(vec![]) }
    async fn read_resource(&self, _: &ResourceId) -> Result<ResourceContent, McpError> { Err(McpError::new(error_codes::RESOURCE_NOT_FOUND, "Not implemented".to_string())) }
}

// Tool provider placeholders
struct SearchToolProvider;
impl SearchToolProvider {
    async fn new() -> Result<Self, McpError> { Ok(Self) }
}
impl McpToolProvider for SearchToolProvider {
    fn provider_id(&self) -> String { "search_tool_provider".to_string() }
    async fn list_tools(&self) -> Result<Vec<McpTool>, McpError> { Ok(vec![]) }
    async fn call_tool(&self, _: &str, _: Option<serde_json::Value>) -> Result<ToolsCallResult, McpError> { Err(McpError::new(error_codes::TOOL_NOT_FOUND, "Not implemented".to_string())) }
}

struct AnalysisToolProvider;
impl AnalysisToolProvider {
    async fn new() -> Result<Self, McpError> { Ok(Self) }
}
impl McpToolProvider for AnalysisToolProvider {
    fn provider_id(&self) -> String { "analysis_tool_provider".to_string() }
    async fn list_tools(&self) -> Result<Vec<McpTool>, McpError> { Ok(vec![]) }
    async fn call_tool(&self, _: &str, _: Option<serde_json::Value>) -> Result<ToolsCallResult, McpError> { Err(McpError::new(error_codes::TOOL_NOT_FOUND, "Not implemented".to_string())) }
}