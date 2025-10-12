use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;

use crate::models::{AgentRecord, AgentContext, AgentInvokeRequest, AgentInvokeResponse, AgentInvokeUsage};
use crate::models::mcp::*;
use crate::services::mcp_server::ConHubMcpServer;
use crate::services::mcp_client::{McpClient, AuthConfig};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AIAgent {
    pub id: String,
    pub agent_type: String,
    pub is_connected: bool,
}

#[async_trait]
pub trait AIAgentConnector: Send + Sync {
    async fn connect(&self, credentials: &HashMap<String, String>) -> Result<bool, Box<dyn Error>>;
    #[allow(dead_code)]
    async fn disconnect(&self) -> Result<bool, Box<dyn Error>>;
    async fn query(&self, prompt: &str, context: Option<&str>) -> Result<String, Box<dyn Error>>;
    fn get_agent(&self) -> AIAgent;
}

pub struct AIAgentManager {
    agents: Arc<Mutex<HashMap<String, Box<dyn AIAgentConnector>>>>,
    agent_service: AgentService,
}

impl AIAgentManager {
    pub fn new() -> Self {
        AIAgentManager {
            agents: Arc::new(Mutex::new(HashMap::new())),
            agent_service: AgentService::new(),
        }
    }

    pub fn create_agent(&self, agent_type: &str) -> Result<AIAgent, Box<dyn Error>> {
        let agent_id = format!("{}-{}", agent_type, Uuid::new_v4());
        let agent: Box<dyn AIAgentConnector> = match agent_type {
            "github_copilot" => Box::new(GitHubCopilotConnector::new(&agent_id)),
            "amazon_q" => Box::new(AmazonQConnector::new(&agent_id)),
            "cline" => Box::new(ClineConnector::new(&agent_id)),
            "cursor_ide" => Box::new(CursorIDEConnector::new(&agent_id)),
            _ => return Err(format!("Unsupported agent type: {}", agent_type).into()),
        };

        let agent_info = agent.get_agent();
        let mut agents = self.agents.lock().unwrap();
        agents.insert(agent_id, agent);
        Ok(agent_info)
    }

    #[allow(dead_code)]
    pub fn get_agent(&self, agent_id: &str) -> Option<AIAgent> {
        let agents = self.agents.lock().unwrap();
        agents.get(agent_id).map(|agent| agent.get_agent())
    }

    pub fn list_agents(&self) -> Vec<AIAgent> {
        let agents = self.agents.lock().unwrap();
        agents.values().map(|agent| agent.get_agent()).collect()
    }

    pub async fn connect_agent(&self, agent_id: &str, credentials: &HashMap<String, String>) -> Result<bool, Box<dyn Error>> {
        let agents = self.agents.lock().unwrap();
        if let Some(agent) = agents.get(agent_id) {
            agent.connect(credentials).await
        } else {
            Err(format!("Agent {} not found", agent_id).into())
        }
    }

    pub async fn query_agent(&self, agent_id: &str, prompt: &str, context: Option<&str>) -> Result<String, Box<dyn Error>> {
        let agents = self.agents.lock().unwrap();
        if let Some(agent) = agents.get(agent_id) {
            agent.query(prompt, context).await
        } else {
            Err(format!("Agent {} not found", agent_id).into())
        }
    }

    pub async fn invoke_agent(
        &self,
        agent: &AgentRecord,
        request: &AgentInvokeRequest,
        context: Option<&AgentContext>,
    ) -> Result<AgentInvokeResponse, Box<dyn std::error::Error>> {
        self.agent_service.invoke_agent(agent, request, context).await
    }
}

// --- GitHub Copilot Connector ---

struct GitHubCopilotConnector {
    agent: AIAgent,
}

impl GitHubCopilotConnector {
    pub fn new(agent_id: &str) -> Self {
        Self {
            agent: AIAgent {
                id: agent_id.to_string(),
                agent_type: "github_copilot".to_string(),
                is_connected: false,
            },
        }
    }
}

#[async_trait]
impl AIAgentConnector for GitHubCopilotConnector {
    async fn connect(&self, credentials: &HashMap<String, String>) -> Result<bool, Box<dyn Error>> {
        // In a real implementation, you would use the credentials to authenticate with the GitHub Copilot API.
        // For now, we'll just simulate a successful connection.
        println!("Connecting to GitHub Copilot with credentials: {:?}", credentials);
        Ok(true)
    }

    async fn disconnect(&self) -> Result<bool, Box<dyn Error>> {
        Ok(true)
    }

    async fn query(&self, prompt: &str, context: Option<&str>) -> Result<String, Box<dyn Error>> {
        // In a real implementation, you would send the prompt and context to the GitHub Copilot API.
        // For now, we'll just return a simulated response.
        let response = format!(
            "GitHub Copilot response for: '{}' with context: '{}'",
            prompt,
            context.unwrap_or("None")
        );
        Ok(response)
    }

    fn get_agent(&self) -> AIAgent {
        self.agent.clone()
    }
}

// --- Cline Connector ---

struct ClineConnector {
    agent: AIAgent,
}

impl ClineConnector {
    pub fn new(agent_id: &str) -> Self {
        Self {
            agent: AIAgent {
                id: agent_id.to_string(),
                agent_type: "cline".to_string(),
                is_connected: false,
            },
        }
    }
}

#[async_trait]
impl AIAgentConnector for ClineConnector {
    async fn connect(&self, credentials: &HashMap<String, String>) -> Result<bool, Box<dyn Error>> {
        // In a real implementation, you would use the credentials to authenticate with the Cline API.
        // For now, we'll just simulate a successful connection.
        println!("Connecting to Cline with credentials: {:?}", credentials);
        Ok(true)
    }

    async fn disconnect(&self) -> Result<bool, Box<dyn Error>> {
        Ok(true)
    }

    async fn query(&self, prompt: &str, context: Option<&str>) -> Result<String, Box<dyn Error>> {
        // In a real implementation, you would send the prompt and context to the Cline API.
        // For now, we'll just return a simulated response.
        let response = format!(
            "Cline response for: '{}' with context: '{}'",
            prompt,
            context.unwrap_or("None")
        );
        Ok(response)
    }

    fn get_agent(&self) -> AIAgent {
        self.agent.clone()
    }
}

// --- Amazon Q Connector ---

struct AmazonQConnector {
    agent: AIAgent,
}

// --- Cursor IDE Connector ---

struct CursorIDEConnector {
    agent: AIAgent,
}

impl CursorIDEConnector {
    pub fn new(agent_id: &str) -> Self {
        Self {
            agent: AIAgent {
                id: agent_id.to_string(),
                agent_type: "cursor_ide".to_string(),
                is_connected: false,
            },
        }
    }
}

#[async_trait]
impl AIAgentConnector for CursorIDEConnector {
    async fn connect(&self, credentials: &HashMap<String, String>) -> Result<bool, Box<dyn Error>> {
        println!("Connecting to Cursor IDE with credentials: {:?}", credentials);
        Ok(true)
    }

    async fn disconnect(&self) -> Result<bool, Box<dyn Error>> {
        Ok(true)
    }

    async fn query(&self, prompt: &str, context: Option<&str>) -> Result<String, Box<dyn Error>> {
        let response = format!(
            "Cursor IDE response for: '{}' with context: '{}'",
            prompt,
            context.unwrap_or("None")
        );
        Ok(response)
    }

    fn get_agent(&self) -> AIAgent {
        self.agent.clone()
    }
}

impl AmazonQConnector {
    pub fn new(agent_id: &str) -> Self {
        Self {
            agent: AIAgent {
                id: agent_id.to_string(),
                agent_type: "amazon_q".to_string(),
                is_connected: false,
            },
        }
    }
}

#[async_trait]
impl AIAgentConnector for AmazonQConnector {
    async fn connect(&self, credentials: &HashMap<String, String>) -> Result<bool, Box<dyn Error>> {
        // In a real implementation, you would use the credentials to authenticate with the Amazon Q API.
        // For now, we'll just simulate a successful connection.
        println!("Connecting to Amazon Q with credentials: {:?}", credentials);
        Ok(true)
    }

    async fn disconnect(&self) -> Result<bool, Box<dyn Error>> {
        Ok(true)
    }

    async fn query(&self, prompt: &str, context: Option<&str>) -> Result<String, Box<dyn Error>> {
        // In a real implementation, you would send the prompt and context to the Amazon Q API.
        // For now, we'll just return a simulated response.
        let response = format!(
            "Amazon Q response for: '{}' with context: '{}'",
            prompt,
            context.unwrap_or("None")
        );
        Ok(response)
    }

    fn get_agent(&self) -> AIAgent {
        self.agent.clone()
    }
}

#[derive(Debug, Clone)]
pub struct AgentService {
    client: Client,
    #[allow(dead_code)]
    mcp_server: Arc<ConHubMcpServer>,
    #[allow(dead_code)]
    mcp_clients: Arc<tokio::sync::RwLock<HashMap<String, McpClient>>>,
}

/// MCP-enhanced context for AI agents
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct McpEnhancedContext {
    pub traditional_context: Option<AgentContext>,
    pub mcp_contexts: Vec<McpContext>,
    pub mcp_resources: Vec<McpResource>,
    pub context_metadata: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

#[derive(Serialize, Deserialize)]
pub struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
pub struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Serialize, Deserialize)]
pub struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Serialize, Deserialize)]
pub struct OpenAIUsage {
    total_tokens: u32,
}

#[derive(Serialize, Deserialize)]
pub struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    temperature: Option<f32>,
}

#[derive(Serialize, Deserialize)]
pub struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
pub struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    usage: AnthropicUsage,
}

#[derive(Serialize, Deserialize)]
pub struct AnthropicContent {
    text: String,
}

#[derive(Serialize, Deserialize)]
pub struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

impl AgentService {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        let mcp_server = Arc::new(ConHubMcpServer::new());
        let mcp_clients = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
        
        Self { 
            client, 
            mcp_server,
            mcp_clients,
        }
    }

    /// Initialize the service with MCP server setup
    #[allow(dead_code)]
    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize the MCP server
        Arc::get_mut(&mut self.mcp_server)
            .ok_or("Failed to get mutable reference to MCP server")?
            .initialize()
            .await
            .map_err(|e| format!("Failed to initialize MCP server: {}", e))?;

        log::info!("AgentService initialized with MCP support");
        Ok(())
    }

    /// Connect to an external MCP server
    #[allow(dead_code)]
    pub async fn connect_external_mcp_server(
        &self,
        name: String,
        endpoint: String,
        auth_config: AuthConfig,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mcp_client = McpClient::new()
            .map_err(|e| format!("Failed to create MCP client: {}", e))?;

        let server_id = mcp_client
            .connect(endpoint, auth_config)
            .await
            .map_err(|e| format!("Failed to connect to MCP server: {}", e))?;

        // Store the client
        {
            let mut clients = self.mcp_clients.write().await;
            clients.insert(name.clone(), mcp_client);
        }

        log::info!("Connected to external MCP server: {} (ID: {})", name, server_id);
        Ok(server_id)
    }

    /// Create MCP-enhanced context for an agent
    #[allow(dead_code)]
    pub async fn create_mcp_context(
        &self,
        agent: &AgentRecord,
        traditional_context: Option<AgentContext>,
    ) -> Result<McpEnhancedContext, Box<dyn std::error::Error>> {
        let mut mcp_contexts = Vec::new();
        let mcp_resources = Vec::new();
        let mut context_metadata = HashMap::new();

        // Create contexts based on agent permissions
        if agent.permissions.contains(&"repositories".to_string()) {
            if let Ok(repo_context) = self.create_repository_context().await {
                mcp_contexts.push(repo_context);
            }
        }

        if agent.permissions.contains(&"documents".to_string()) {
            if let Ok(doc_context) = self.create_document_context().await {
                mcp_contexts.push(doc_context);
            }
        }

        if agent.permissions.contains(&"urls".to_string()) {
            if let Ok(url_context) = self.create_url_context().await {
                mcp_contexts.push(url_context);
            }
        }

        // Add metadata about context creation
        context_metadata.insert("created_at".to_string(), json!(chrono::Utc::now()));
        context_metadata.insert("agent_id".to_string(), json!(agent.id));
        context_metadata.insert("permissions".to_string(), json!(agent.permissions));

        Ok(McpEnhancedContext {
            traditional_context,
            mcp_contexts,
            mcp_resources,
            context_metadata,
        })
    }

    /// Create repository context using MCP
    #[allow(dead_code)]
    async fn create_repository_context(&self) -> Result<McpContext, McpError> {
        let context_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        // In a real implementation, this would fetch actual repository data
        let context = McpContext {
            id: context_id,
            name: "Repository Context".to_string(),
            description: Some("Context containing repository information and code".to_string()),
            context_type: ContextType::Repository,
            resources: vec![
                ContextResource {
                    resource_id: "repo_1".to_string(),
                    relevance_score: Some(0.9),
                    content: Some("ConHub main repository".to_string()),
                    content_type: Some("application/vnd.conhub.repository".to_string()),
                    annotations: Some(ResourceAnnotations {
                        audience: Some(vec!["developers".to_string()]),
                        priority: Some(0.9),
                        tags: vec!["repository".to_string(), "code".to_string()],
                        source_type: "git".to_string(),
                        confidence: Some(0.95),
                    }),
                }
            ],
            metadata: HashMap::new(),
            created_at: now,
            expires_at: None,
            access_level: AccessLevel::Internal,
        };

        Ok(context)
    }

    /// Create document context using MCP
    #[allow(dead_code)]
    async fn create_document_context(&self) -> Result<McpContext, McpError> {
        let context_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        let context = McpContext {
            id: context_id,
            name: "Document Context".to_string(),
            description: Some("Context containing documents and documentation".to_string()),
            context_type: ContextType::Document,
            resources: vec![
                ContextResource {
                    resource_id: "doc_1".to_string(),
                    relevance_score: Some(0.8),
                    content: Some("API documentation and guides".to_string()),
                    content_type: Some("text/markdown".to_string()),
                    annotations: Some(ResourceAnnotations {
                        audience: Some(vec!["users".to_string(), "developers".to_string()]),
                        priority: Some(0.8),
                        tags: vec!["documentation".to_string(), "api".to_string()],
                        source_type: "markdown".to_string(),
                        confidence: Some(0.9),
                    }),
                }
            ],
            metadata: HashMap::new(),
            created_at: now,
            expires_at: None,
            access_level: AccessLevel::Internal,
        };

        Ok(context)
    }

    /// Create URL context using MCP
    #[allow(dead_code)]
    async fn create_url_context(&self) -> Result<McpContext, McpError> {
        let context_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        let context = McpContext {
            id: context_id,
            name: "URL Context".to_string(),
            description: Some("Context containing web resources and URLs".to_string()),
            context_type: ContextType::Url,
            resources: vec![
                ContextResource {
                    resource_id: "url_1".to_string(),
                    relevance_score: Some(0.7),
                    content: Some("External web resources and references".to_string()),
                    content_type: Some("text/html".to_string()),
                    annotations: Some(ResourceAnnotations {
                        audience: Some(vec!["researchers".to_string()]),
                        priority: Some(0.7),
                        tags: vec!["web".to_string(), "reference".to_string()],
                        source_type: "url".to_string(),
                        confidence: Some(0.8),
                    }),
                }
            ],
            metadata: HashMap::new(),
            created_at: now,
            expires_at: None,
            access_level: AccessLevel::Internal,
        };

        Ok(context)
    }

    #[allow(dead_code)]
    pub async fn invoke_agent(
        &self,
        agent: &AgentRecord,
        request: &AgentInvokeRequest,
        context: Option<&AgentContext>,
    ) -> Result<AgentInvokeResponse, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        // Create MCP-enhanced context
        let mcp_context = self.create_mcp_context(agent, context.cloned()).await?;
        
        let response = match agent.agent_type.as_str() {
            "openai" => self.invoke_openai_with_mcp(agent, request, &mcp_context).await?,
            "anthropic" => self.invoke_anthropic_with_mcp(agent, request, &mcp_context).await?,
            "custom" => self.invoke_custom_with_mcp(agent, request, &mcp_context).await?,
            _ => return Err("Unsupported agent type".into()),
        };
        
        let response_time = start_time.elapsed().as_millis() as u64;
        
        Ok(AgentInvokeResponse {
            response: response.response,
            usage: AgentInvokeUsage {
                tokens_used: response.usage.tokens_used,
                response_time_ms: response_time,
            },
            context_used: response.context_used,
        })
    }

    /// OpenAI invocation with MCP-enhanced context
    #[allow(dead_code)]
    async fn invoke_openai_with_mcp(
        &self,
        agent: &AgentRecord,
        request: &AgentInvokeRequest,
        mcp_context: &McpEnhancedContext,
    ) -> Result<AgentInvokeResponse, Box<dyn std::error::Error>> {
        let model = agent.config.model.as_ref().unwrap_or(&"gpt-4".to_string()).clone();
        let temperature = agent.config.temperature.unwrap_or(0.7);
        let max_tokens = agent.config.max_tokens.unwrap_or(1000);
        
        let mut messages = vec![];
        
        // Add system message with MCP-enhanced context
        let context_summary = self.format_mcp_context_for_openai(mcp_context);
        if !context_summary.is_empty() {
            messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: format!(
                    "You are an AI assistant with access to structured context through the Model Context Protocol (MCP):\n{}",
                    context_summary
                ),
            });
        }

        // Add custom instructions if any
        if let Some(instructions) = &agent.config.custom_instructions {
            messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: instructions.clone(),
            });
        }

        // Add user message
        messages.push(OpenAIMessage {
            role: "user".to_string(),
            content: request.message.clone(),
        });

        let payload = OpenAIRequest {
            model,
            messages,
            max_tokens: Some(max_tokens),
            temperature: Some(temperature),
        };

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", agent.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("OpenAI API error: {}", error_text).into());
        }

        let openai_response: OpenAIResponse = response.json().await?;

        let response_text = openai_response.choices
            .first()
            .ok_or("No response from OpenAI")?
            .message
            .content
            .clone();

        let context_used = mcp_context.mcp_contexts.iter()
            .map(|ctx| ctx.name.clone())
            .collect::<Vec<_>>();

        Ok(AgentInvokeResponse {
            response: response_text,
            usage: AgentInvokeUsage {
                tokens_used: openai_response.usage.total_tokens,
                response_time_ms: 0, // Will be set by caller
            },
            context_used,
        })
    }

    /// Anthropic invocation with MCP-enhanced context
    #[allow(dead_code)]
    async fn invoke_anthropic_with_mcp(
        &self,
        agent: &AgentRecord,
        request: &AgentInvokeRequest,
        mcp_context: &McpEnhancedContext,
    ) -> Result<AgentInvokeResponse, Box<dyn std::error::Error>> {
        let model = agent.config.model.as_ref().unwrap_or(&"claude-3-sonnet-20240229".to_string()).clone();
        let temperature = agent.config.temperature.unwrap_or(0.7);
        let max_tokens = agent.config.max_tokens.unwrap_or(1000);

        // Format MCP context for Anthropic
        let context_summary = self.format_mcp_context_for_anthropic(mcp_context);
        
        let mut content = String::new();
        if !context_summary.is_empty() {
            content.push_str(&format!(
                "Context available through Model Context Protocol (MCP):\n{}\n\nUser request: ",
                context_summary
            ));
        }
        content.push_str(&request.message);

        let messages = vec![AnthropicMessage {
            role: "user".to_string(),
            content,
        }];

        let payload = AnthropicRequest {
            model,
            messages,
            max_tokens,
            temperature: Some(temperature),
        };

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &agent.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Anthropic API error: {}", error_text).into());
        }

        let anthropic_response: AnthropicResponse = response.json().await?;

        let response_text = anthropic_response.content
            .first()
            .ok_or("No response from Anthropic")?
            .text
            .clone();

        let total_tokens = anthropic_response.usage.input_tokens + anthropic_response.usage.output_tokens;
        let context_used = mcp_context.mcp_contexts.iter()
            .map(|ctx| ctx.name.clone())
            .collect::<Vec<_>>();

        Ok(AgentInvokeResponse {
            response: response_text,
            usage: AgentInvokeUsage {
                tokens_used: total_tokens,
                response_time_ms: 0, // Will be set by caller
            },
            context_used,
        })
    }

    /// Custom agent invocation with MCP-enhanced context
    #[allow(dead_code)]
    async fn invoke_custom_with_mcp(
        &self,
        agent: &AgentRecord,
        request: &AgentInvokeRequest,
        mcp_context: &McpEnhancedContext,
    ) -> Result<AgentInvokeResponse, Box<dyn std::error::Error>> {
        let endpoint = agent.endpoint.as_ref()
            .ok_or("Custom agent endpoint not configured")?;

        // Create MCP-aware payload
        let context_summary = self.format_mcp_context_for_custom(mcp_context);
        
        let payload = json!({
            "message": request.message,
            "mcp_context": {
                "contexts": mcp_context.mcp_contexts,
                "resources": mcp_context.mcp_resources,
                "metadata": mcp_context.context_metadata,
                "formatted_summary": context_summary
            },
            "config": agent.config,
            "include_history": request.include_history.unwrap_or(false)
        });

        let response = self.client
            .post(endpoint)
            .header("Authorization", format!("Bearer {}", agent.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Custom agent API error: {}", error_text).into());
        }

        let response_json: Value = response.json().await?;
        
        let response_text = response_json["response"]
            .as_str()
            .unwrap_or("No response from custom agent")
            .to_string();

        let tokens_used = response_json["usage"]["tokens_used"]
            .as_u64()
            .unwrap_or(0) as u32;

        let context_used = mcp_context.mcp_contexts.iter()
            .map(|ctx| ctx.name.clone())
            .collect::<Vec<_>>();

        Ok(AgentInvokeResponse {
            response: response_text,
            usage: AgentInvokeUsage {
                tokens_used,
                response_time_ms: 0, // Will be set by caller
            },
            context_used,
        })
    }

    /// Format MCP context for OpenAI
    #[allow(dead_code)]
    fn format_mcp_context_for_openai(&self, mcp_context: &McpEnhancedContext) -> String {
        let mut context_str = String::new();

        // Add MCP contexts
        if !mcp_context.mcp_contexts.is_empty() {
            context_str.push_str("## MCP Contexts:\n");
            for context in &mcp_context.mcp_contexts {
                context_str.push_str(&format!(
                    "### {} ({})\n",
                    context.name,
                    match context.context_type {
                        ContextType::Repository => "Repository",
                        ContextType::Document => "Document", 
                        ContextType::Url => "URL",
                        ContextType::DataSource => "Data Source",
                        ContextType::Agent => "Agent",
                        ContextType::Conversation => "Conversation",
                        ContextType::Tool => "Tool",
                        ContextType::Custom(ref name) => name,
                    }
                ));
                
                if let Some(description) = &context.description {
                    context_str.push_str(&format!("Description: {}\n", description));
                }

                for resource in &context.resources {
                    context_str.push_str(&format!(
                        "- Resource: {} (relevance: {:.2})\n",
                        resource.resource_id,
                        resource.relevance_score.unwrap_or(0.0)
                    ));
                    
                    if let Some(content) = &resource.content {
                        context_str.push_str(&format!("  Content: {}\n", content));
                    }
                }
                context_str.push('\n');
            }
        }

        // Add traditional context if available for backward compatibility
        if let Some(traditional) = &mcp_context.traditional_context {
            context_str.push_str(&self.format_context_for_openai(traditional));
        }

        context_str
    }

    /// Format MCP context for Anthropic
    #[allow(dead_code)]
    fn format_mcp_context_for_anthropic(&self, mcp_context: &McpEnhancedContext) -> String {
        let mut context_str = String::new();

        if !mcp_context.mcp_contexts.is_empty() {
            context_str.push_str("Available MCP Contexts:\n");
            for context in &mcp_context.mcp_contexts {
                context_str.push_str(&format!(
                    "- {}: {}\n",
                    context.name,
                    context.description.as_ref().unwrap_or(&"No description".to_string())
                ));
                
                for resource in &context.resources {
                    if let Some(content) = &resource.content {
                        context_str.push_str(&format!("  * {}\n", content));
                    }
                }
            }
        }

        // Add traditional context if available
        if let Some(traditional) = &mcp_context.traditional_context {
            context_str.push_str(&self.format_context_for_anthropic(traditional));
        }

        context_str
    }

    /// Format MCP context for custom agents
    #[allow(dead_code)]
    fn format_mcp_context_for_custom(&self, mcp_context: &McpEnhancedContext) -> serde_json::Value {
        json!({
            "mcp_contexts": mcp_context.mcp_contexts.len(),
            "context_types": mcp_context.mcp_contexts.iter().map(|c| &c.context_type).collect::<Vec<_>>(),
            "resources_count": mcp_context.mcp_resources.len(),
            "metadata": mcp_context.context_metadata,
            "traditional_context_available": mcp_context.traditional_context.is_some()
        })
    }

    #[allow(dead_code)]
    async fn invoke_openai(
        &self,
        agent: &AgentRecord,
        request: &AgentInvokeRequest,
        context: Option<&AgentContext>,
    ) -> Result<AgentInvokeResponse, Box<dyn std::error::Error>> {
        let model = agent.config.model.as_ref().unwrap_or(&"gpt-4".to_string()).clone();
        let temperature = agent.config.temperature.unwrap_or(0.7);
        let max_tokens = agent.config.max_tokens.unwrap_or(1000);
        
        // Use include_history flag (placeholder for future conversation history feature)
        let _include_history = request.include_history.unwrap_or(false);
        
        let mut messages = vec![];
        
        // Add system message with context if available
        if let Some(ctx) = context {
            let context_summary = self.format_context_for_openai(ctx);
            messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: format!(
                    "You are an AI assistant with access to the following context:\n{}",
                    context_summary
                ),
            });
        }
        
        // Add custom instructions if any
        if let Some(instructions) = &agent.config.custom_instructions {
            messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: instructions.clone(),
            });
        }
        
        // Add user message
        messages.push(OpenAIMessage {
            role: "user".to_string(),
            content: request.message.clone(),
        });
        
        let payload = OpenAIRequest {
            model,
            messages,
            max_tokens: Some(max_tokens),
            temperature: Some(temperature),
        };
        
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", agent.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("OpenAI API error: {}", error_text).into());
        }
        
        let openai_response: OpenAIResponse = response.json().await?;
        
        let response_text = openai_response.choices
            .first()
            .ok_or("No response from OpenAI")?
            .message
            .content
            .clone();
        
        let context_used = context.map(|_| vec!["repositories".to_string(), "documents".to_string(), "urls".to_string()])
            .unwrap_or_default();
        
        Ok(AgentInvokeResponse {
            response: response_text,
            usage: AgentInvokeUsage {
                tokens_used: openai_response.usage.total_tokens,
                response_time_ms: 0, // Will be set by caller
            },
            context_used,
        })
    }

    #[allow(dead_code)]
    async fn invoke_anthropic(
        &self,
        agent: &AgentRecord,
        request: &AgentInvokeRequest,
        context: Option<&AgentContext>,
    ) -> Result<AgentInvokeResponse, Box<dyn std::error::Error>> {
        let model = agent.config.model.as_ref().unwrap_or(&"claude-3-sonnet-20240229".to_string()).clone();
        let temperature = agent.config.temperature.unwrap_or(0.7);
        let max_tokens = agent.config.max_tokens.unwrap_or(1000);
        
        let mut content = request.message.clone();
        
        // Add context if available
        if let Some(ctx) = context {
            let context_summary = self.format_context_for_anthropic(ctx);
            content = format!("Context:\n{}\n\nUser request: {}", context_summary, content);
        }
        
        // Add custom instructions if any
        if let Some(instructions) = &agent.config.custom_instructions {
            content = format!("Instructions: {}\n\n{}", instructions, content);
        }
        
        let messages = vec![AnthropicMessage {
            role: "user".to_string(),
            content,
        }];
        
        let payload = AnthropicRequest {
            model,
            messages,
            max_tokens,
            temperature: Some(temperature),
        };
        
        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &agent.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Anthropic API error: {}", error_text).into());
        }
        
        let anthropic_response: AnthropicResponse = response.json().await?;
        
        let response_text = anthropic_response.content
            .first()
            .ok_or("No response from Anthropic")?
            .text
            .clone();
        
        let total_tokens = anthropic_response.usage.input_tokens + anthropic_response.usage.output_tokens;
        let context_used = context.map(|_| vec!["repositories".to_string(), "documents".to_string(), "urls".to_string()])
            .unwrap_or_default();
        
        Ok(AgentInvokeResponse {
            response: response_text,
            usage: AgentInvokeUsage {
                tokens_used: total_tokens,
                response_time_ms: 0, // Will be set by caller
            },
            context_used,
        })
    }

    #[allow(dead_code)]
    async fn invoke_custom(
        &self,
        agent: &AgentRecord,
        request: &AgentInvokeRequest,
        context: Option<&AgentContext>,
    ) -> Result<AgentInvokeResponse, Box<dyn std::error::Error>> {
        let endpoint = agent.endpoint.as_ref().ok_or("Custom agent requires endpoint")?;
        
        let mut payload = json!({
            "message": request.message,
            "config": agent.config
        });
        
        if let Some(ctx) = context {
            payload["context"] = json!(ctx);
        }
        
        let mut request_builder = self.client.post(endpoint)
            .header("Content-Type", "application/json")
            .json(&payload);
        
        // Add authentication if API key is provided
        if !agent.api_key.is_empty() {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", agent.api_key));
        }
        
        let response = request_builder.send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Custom agent API error: {}", error_text).into());
        }
        
        let response_json: Value = response.json().await?;
        
        let response_text = response_json.get("response")
            .and_then(|v| v.as_str())
            .unwrap_or("No response from custom agent")
            .to_string();
        
        let tokens_used = response_json.get("tokens_used")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        
        let context_used = context.map(|_| vec!["custom".to_string()])
            .unwrap_or_default();
        
        Ok(AgentInvokeResponse {
            response: response_text,
            usage: AgentInvokeUsage {
                tokens_used,
                response_time_ms: 0, // Will be set by caller
            },
            context_used,
        })
    }

    fn format_context_for_openai(&self, context: &AgentContext) -> String {
        let mut context_str = String::new();
        
        if !context.repositories.is_empty() {
            context_str.push_str("## Repositories:\n");
            for repo in &context.repositories {
                context_str.push_str(&format!(
                    "- {}: {} ({})\n  Recent files: {}\n  Recent commits: {}\n",
                    repo.name,
                    repo.description.as_ref().unwrap_or(&"No description".to_string()),
                    repo.language,
                    repo.recent_files.join(", "),
                    repo.recent_commits.join(", ")
                ));
            }
        }
        
        if !context.documents.is_empty() {
            context_str.push_str("\n## Documents:\n");
            for doc in &context.documents {
                context_str.push_str(&format!(
                    "- {}: {} ({})\n  Summary: {}\n  Tags: {}\n",
                    doc.name,
                    doc.doc_type,
                    doc.id,
                    doc.summary.as_ref().unwrap_or(&"No summary".to_string()),
                    doc.tags.join(", ")
                ));
            }
        }
        
        if !context.urls.is_empty() {
            context_str.push_str("\n## URLs:\n");
            for url in &context.urls {
                context_str.push_str(&format!(
                    "- {}: {}\n  Summary: {}\n  Tags: {}\n",
                    url.title.as_ref().unwrap_or(&"No title".to_string()),
                    url.url,
                    url.summary.as_ref().unwrap_or(&"No summary".to_string()),
                    url.tags.join(", ")
                ));
            }
        }
        
        context_str
    }

    fn format_context_for_anthropic(&self, context: &AgentContext) -> String {
        // Similar to OpenAI but potentially with different formatting preferences
        self.format_context_for_openai(context)
    }

    #[allow(dead_code)]
    pub async fn test_agent_connection(&self, agent: &AgentRecord) -> Result<bool, Box<dyn std::error::Error>> {
        match agent.agent_type.as_str() {
            "openai" => self.test_openai_connection(agent).await,
            "anthropic" => self.test_anthropic_connection(agent).await,
            "custom" => self.test_custom_connection(agent).await,
            _ => Err("Unsupported agent type".into()),
        }
    }

    #[allow(dead_code)]
    async fn test_openai_connection(&self, agent: &AgentRecord) -> Result<bool, Box<dyn std::error::Error>> {
        let response = self.client
            .get("https://api.openai.com/v1/models")
            .header("Authorization", format!("Bearer {}", agent.api_key))
            .send()
            .await?;
        
        Ok(response.status().is_success())
    }

    #[allow(dead_code)]
    async fn test_anthropic_connection(&self, agent: &AgentRecord) -> Result<bool, Box<dyn std::error::Error>> {
        // Anthropic doesn't have a simple health check endpoint, so we'll make a minimal request
        let messages = vec![AnthropicMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];
        
        let payload = AnthropicRequest {
            model: "claude-3-haiku-20240307".to_string(), // Use the cheapest model for testing
            messages,
            max_tokens: 10,
            temperature: Some(0.0),
        };
        
        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &agent.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;
        
        Ok(response.status().is_success())
    }

    #[allow(dead_code)]
    async fn test_custom_connection(&self, agent: &AgentRecord) -> Result<bool, Box<dyn std::error::Error>> {
        let endpoint = agent.endpoint.as_ref().ok_or("Custom agent requires endpoint")?;
        
        // Try a simple health check or minimal request
        let health_url = format!("{}/health", endpoint.trim_end_matches('/'));
        let response = self.client
            .get(&health_url)
            .send()
            .await;
        
        match response {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => {
                // If health endpoint doesn't exist, try the main endpoint with a test request
                let test_payload = json!({
                    "message": "test",
                    "test": true
                });
                
                let response = self.client
                    .post(endpoint)
                    .header("Content-Type", "application/json")
                    .json(&test_payload)
                    .send()
                    .await?;
                
                Ok(response.status().is_success())
            }
        }
    }
}
