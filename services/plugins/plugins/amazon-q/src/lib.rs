use anyhow::{anyhow, Result};
use async_stream::stream;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use conhub_plugins::{
    Plugin, PluginConfig, PluginMetadata, PluginStatus, PluginResult,
    agents::{AgentAction, AgentCapabilities, AgentFunction, AgentMessage, AgentPlugin, AgentResponse, AgentResponseChunk, ChunkType, ConversationContext, MessageRole},
};
use futures_util::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmazonQConfig {
    pub api_endpoint: String,
    pub api_key: Option<String>,
    pub model_id: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub application_id: Option<String>,
    pub index_id: Option<String>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AmazonQRequest {
    message: String,
    context: Option<ConversationContext>,
    application_id: Option<String>,
    user_id: String,
    conversation_id: Option<String>,
    parent_message_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AmazonQResponse {
    response: String,
    conversation_id: String,
    message_id: String,
    source_attributions: Option<Vec<SourceAttribution>>,
    system_message: Option<String>,
    user_message_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SourceAttribution {
    title: String,
    snippet: String,
    url: Option<String>,
    updated_at: Option<DateTime<Utc>>,
}

pub struct AmazonQPlugin {
    config: Arc<RwLock<Option<AmazonQConfig>>>,
    http_client: Arc<RwLock<Option<Client>>>,
    status: Arc<RwLock<PluginStatus>>,
    conversation_history: Arc<RwLock<HashMap<String, Vec<AgentMessage>>>>,
}

impl AmazonQPlugin {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(None)),
            http_client: Arc::new(RwLock::new(None)),
            status: Arc::new(RwLock::new(PluginStatus::Inactive)),
            conversation_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn get_config(&self) -> Result<AmazonQConfig> {
        let config_guard = self.config.read().await;
        config_guard
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow!("Plugin not configured"))
    }

    async fn get_http_client(&self) -> Result<Client> {
        let client_guard = self.http_client.read().await;
        client_guard
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow!("HTTP client not initialized"))
    }

    async fn initialize_http_client(&self, config: &AmazonQConfig) -> Result<Client> {
        let timeout = std::time::Duration::from_secs(config.timeout_seconds.unwrap_or(30));
        
        let mut builder = Client::builder()
            .timeout(timeout)
            .user_agent("ConHub-AmazonQ-Plugin/1.0.0");
        
        let client = builder.build()?;
        
        Ok(client)
    }

    async fn invoke_amazon_q_api(&self, prompt: &str) -> Result<String> {
        let config = self.get_config().await?;
        let client = self.get_http_client().await?;
        
        let model_id = config.model_id.as_deref().unwrap_or("amazon-q-default");
        
        // Prepare the request body for Amazon Q API
        let request_body = json!({
            "model": model_id,
            "max_tokens": config.max_tokens.unwrap_or(4096),
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": config.temperature.unwrap_or(0.7),
            "top_p": config.top_p.unwrap_or(0.9)
        });
        
        let mut request_builder = client
            .post(&config.api_endpoint)
            .header("Content-Type", "application/json")
            .json(&request_body);
        
        if let Some(api_key) = &config.api_key {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = request_builder.send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Amazon Q API request failed: {}", response.status()));
        }
        
        let response_json: Value = response.json().await?;
        
        // Parse Amazon Q response
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .or_else(|| response_json["response"].as_str())
            .or_else(|| response_json["content"].as_str())
            .ok_or_else(|| anyhow!("Invalid response format from Amazon Q API"))?;
        
        Ok(content.to_string())
    }

    async fn query_amazon_q(&self, request: AmazonQRequest) -> Result<AmazonQResponse> {
        let config = self.get_config().await?;
        
        // Use Amazon Q API directly
        let prompt = self.build_amazon_q_prompt(&request).await?;
        let response_content = self.invoke_amazon_q_api(&prompt).await?;
        
        // Generate conversation and message IDs
        let conversation_id = request.conversation_id
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let message_id = Uuid::new_v4().to_string();
        
        Ok(AmazonQResponse {
            response: response_content,
            conversation_id,
            message_id,
            source_attributions: None, // Would be populated from Amazon Q knowledge base
            system_message: None,
            user_message_id: None,
        })
    }

    async fn build_amazon_q_prompt(&self, request: &AmazonQRequest) -> Result<String> {
        let mut prompt = String::new();
        
        // System prompt for Amazon Q behavior
        prompt.push_str("You are Amazon Q, an AI assistant that helps with various tasks including ");
        prompt.push_str("coding, AWS services, general questions, and business intelligence. ");
        prompt.push_str("Provide helpful, accurate, and professional responses.\n\n");
        
        // Add conversation context if available
        if let Some(context) = &request.context {
            if !context.messages.is_empty() {
                prompt.push_str("Previous conversation:\n");
                for msg in &context.messages {
                    prompt.push_str(&format!("{:?}: {}\n", msg.role, msg.content));
                }
                prompt.push_str("\n");
            }
        }
        
        // Add the current request
        prompt.push_str(&format!("User: {}\n", request.message));
        prompt.push_str("Amazon Q:");
        
        Ok(prompt)
    }

    async fn execute_amazon_q_action(&self, action: &AgentAction) -> Result<Value> {
        match action.action_type.as_str() {
            "aws_service_info" => {
                let service_name = action.parameters.get("service_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing service_name parameter"))?;
                
                // In a real implementation, you would query AWS service information
                info!("Getting AWS service info for: {}", service_name);
                Ok(json!({
                    "service": service_name,
                    "description": format!("Information about AWS {}", service_name),
                    "documentation_url": format!("https://docs.aws.amazon.com/{}/", service_name.to_lowercase())
                }))
            }
            "code_analysis" => {
                let code = action.parameters.get("code")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing code parameter"))?;
                
                let language = action.parameters.get("language")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                
                // Analyze code using Amazon Q capabilities
                info!("Analyzing {} code", language);
                Ok(json!({
                    "language": language,
                    "code_length": code.len(),
                    "analysis": "Code analysis would be performed here",
                    "suggestions": ["Consider adding error handling", "Add documentation"]
                }))
            }
            "knowledge_search" => {
                let query = action.parameters.get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing query parameter"))?;
                
                // Search Amazon Q knowledge base
                info!("Searching knowledge base for: {}", query);
                Ok(json!({
                    "query": query,
                    "results": [
                        {
                            "title": "Sample Knowledge Article",
                            "snippet": "This is a sample knowledge base result",
                            "relevance_score": 0.85
                        }
                    ]
                }))
            }
            _ => {
                Err(anyhow!("Unknown action type: {}", action.action_type))
            }
        }
    }
}

#[async_trait]
impl Plugin for AmazonQPlugin {
    fn metadata(&self) -> &PluginMetadata {
        // We need to store metadata as a field in the struct
        // For now, let's create a static one
        static METADATA: std::sync::OnceLock<PluginMetadata> = std::sync::OnceLock::new();
        METADATA.get_or_init(|| PluginMetadata {
            id: "amazon-q".to_string(),
            name: "Amazon Q".to_string(),
            version: "0.1.0".to_string(),
            description: "Amazon Q AI assistant integration".to_string(),
            author: "ConHub Team".to_string(),
            plugin_type: conhub_plugins::PluginType::Agent,
            capabilities: vec![
                "chat".to_string(),
                "code_analysis".to_string(),
                "knowledge_search".to_string(),
                "aws_integration".to_string(),
            ],
            config_schema: Some(json!({
                "type": "object",
                "properties": {
                    "api_endpoint": {
                        "type": "string",
                        "description": "Amazon Q API endpoint URL"
                    },
                    "api_key": {
                        "type": "string",
                        "description": "API key for authentication"
                    },
                    "model_id": {
                        "type": "string",
                        "description": "Model ID to use for responses",
                        "default": "anthropic.claude-3-sonnet-20240229-v1:0"
                    },
                    "max_tokens": {
                        "type": "integer",
                        "description": "Maximum tokens in response",
                        "default": 4000
                    },
                    "temperature": {
                        "type": "number",
                        "description": "Response creativity (0.0-1.0)",
                        "default": 0.7
                    },
                    "top_p": {
                        "type": "number",
                        "description": "Nucleus sampling parameter",
                        "default": 0.9
                    },
                    "application_id": {
                        "type": "string",
                        "description": "Amazon Q application ID"
                    },
                    "index_id": {
                        "type": "string",
                        "description": "Amazon Q index ID for knowledge search"
                    },
                    "timeout_seconds": {
                        "type": "integer",
                        "description": "Request timeout in seconds",
                        "default": 30
                    }
                },
                "required": ["api_endpoint"]
            })),
        })
    }

    async fn initialize(&mut self, config: PluginConfig) -> Result<(), conhub_plugins::error::PluginError> {
        info!("Initializing Amazon Q plugin");
        
        let amazon_q_config: AmazonQConfig = serde_json::from_value(
            config.settings.get("amazon_q").unwrap_or(&json!({})).clone()
        ).map_err(|e| conhub_plugins::error::PluginError::ConfigurationError(e.to_string()))?;
        
        let http_client = self.initialize_http_client(&amazon_q_config).await
            .map_err(|e| conhub_plugins::error::PluginError::InitializationFailed(e.to_string()))?;
        
        {
            let mut config_guard = self.config.write().await;
            *config_guard = Some(amazon_q_config);
        }
        
        {
            let mut client_guard = self.http_client.write().await;
            *client_guard = Some(http_client);
        }
        
        {
            let mut status_guard = self.status.write().await;
            *status_guard = PluginStatus::Inactive;
        }
        
        info!("Amazon Q plugin initialized successfully");
        Ok(())
    }

    async fn start(&mut self) -> Result<(), conhub_plugins::error::PluginError> {
        info!("Starting Amazon Q plugin");
        
        let mut status_guard = self.status.write().await;
        *status_guard = PluginStatus::Active;
        
        info!("Amazon Q plugin started");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), conhub_plugins::error::PluginError> {
        info!("Stopping Amazon Q plugin");
        
        let mut status_guard = self.status.write().await;
        *status_guard = PluginStatus::Inactive;
        
        info!("Amazon Q plugin stopped");
        Ok(())
    }

    fn status(&self) -> PluginStatus {
        // We need to make this synchronous, so we'll use try_read
        match self.status.try_read() {
            Ok(status) => status.clone(),
            Err(_) => PluginStatus::Error("Failed to read status".to_string()),
        }
    }

    async fn health_check(&self) -> Result<bool, conhub_plugins::error::PluginError> {
        // Test Amazon Q API connectivity
        match self.invoke_amazon_q_api("Health check").await {
            Ok(response) => Ok(!response.is_empty()),
            Err(e) => Err(conhub_plugins::error::PluginError::NetworkError(e.to_string())),
        }
    }

    fn validate_config(&self, config: &PluginConfig) -> Result<(), conhub_plugins::error::PluginError> {
        let _: AmazonQConfig = serde_json::from_value(
            config.settings.get("amazon_q").unwrap_or(&json!({})).clone()
        ).map_err(|e| conhub_plugins::error::PluginError::ValidationError(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl AgentPlugin for AmazonQPlugin {
    fn capabilities(&self) -> AgentCapabilities {
        AgentCapabilities {
            supports_chat: true,
            supports_code_generation: false,
            supports_code_analysis: true,
            supports_file_operations: false,
            supports_web_search: true,
            supports_function_calling: true,
            max_context_length: Some(4096),
            supported_languages: vec![
                "english".to_string(),
                "spanish".to_string(),
                "french".to_string(),
                "german".to_string(),
                "italian".to_string(),
                "portuguese".to_string(),
                "russian".to_string(),
                "japanese".to_string(),
                "korean".to_string(),
                "chinese".to_string(),
            ],
        }
    }

    async fn process_message(
        &self,
        message: AgentMessage,
        context: ConversationContext,
    ) -> PluginResult<AgentResponse> {
        info!("Processing message with Amazon Q: {}", message.content);
        
        let user_id = "default_user".to_string(); // Default user ID since ConversationContext doesn't have user_id
        let conversation_id = context.conversation_id.clone();
        
        // Add message to conversation history
        let mut history = self.conversation_history.write().await;
        let conv_history = history.entry(conversation_id.clone()).or_insert_with(Vec::new);
        conv_history.push(message.clone());
        
        let request = AmazonQRequest {
            message: message.content.clone(),
            context: Some(context),
            application_id: self.get_config().await.ok().and_then(|c| c.application_id),
            user_id,
            conversation_id: Some(conversation_id.clone()),
            parent_message_id: None,
        };
        
        let amazon_q_response = self.query_amazon_q(request).await?;
        
        let mut metadata = HashMap::new();
        metadata.insert("conversation_id".to_string(), serde_json::Value::String(amazon_q_response.conversation_id.clone()));
        metadata.insert("message_id".to_string(), serde_json::Value::String(amazon_q_response.message_id.clone()));
        if let Some(source_attributions) = &amazon_q_response.source_attributions {
            metadata.insert("source_attributions".to_string(), serde_json::to_value(source_attributions).map_err(|e| anyhow::anyhow!(e))?);
        }
        if let Some(system_message) = &amazon_q_response.system_message {
            metadata.insert("system_message".to_string(), serde_json::Value::String(system_message.clone()));
        }
        
        let response_message = AgentMessage {
            id: amazon_q_response.message_id.clone(),
            content: amazon_q_response.response,
            role: MessageRole::Assistant,
            timestamp: Utc::now(),
            metadata,
        };

        let response = AgentResponse {
            message: response_message.clone(),
            actions: Vec::new(),
            confidence: 0.95, // High confidence for Amazon Q responses
            processing_time_ms: 0, // TODO: Track actual processing time
        };
        
        // Add response to conversation history
        conv_history.push(response_message);
        
        info!("Amazon Q response generated successfully");
        Ok(response)
    }

    async fn get_available_functions(&self) -> PluginResult<Vec<AgentFunction>> {
        Ok(vec![
            AgentFunction {
                name: "aws_service_info".to_string(),
                description: "Get information about AWS services".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "service_name": {
                            "type": "string",
                            "description": "Name of the AWS service"
                        }
                    },
                    "required": ["service_name"]
                }),
            },
            AgentFunction {
                name: "code_analysis".to_string(),
                description: "Analyze code for best practices and improvements".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "Code to analyze"
                        },
                        "language": {
                            "type": "string",
                            "description": "Programming language of the code"
                        }
                    },
                    "required": ["code"]
                }),
            },
            AgentFunction {
                name: "knowledge_search".to_string(),
                description: "Search the Amazon Q knowledge base".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query for the knowledge base"
                        }
                    },
                    "required": ["query"]
                }),
            },
        ])
    }

    async fn execute_action(
        &self,
        action: AgentAction,
    ) -> PluginResult<Value> {
        info!("Executing Amazon Q action: {}", action.action_type);
        self.execute_amazon_q_action(&action).await.map_err(Into::into)
    }

    async fn stream_response(
        &self,
        message: AgentMessage,
        context: ConversationContext,
    ) -> PluginResult<tokio::sync::mpsc::Receiver<AgentResponseChunk>> {
        info!("Starting streaming response with Amazon Q");
        
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        
        // For this implementation, we'll simulate streaming by chunking the response
        let response = self.process_message(message, context).await?;
        
        // Spawn a task to send chunks
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let words: Vec<&str> = response.message.content.split_whitespace().collect();
            let chunk_size = 8; // Words per chunk
            
            for chunk in words.chunks(chunk_size) {
                let chunk_content = chunk.join(" ");
                let mut metadata = HashMap::new();
                metadata.insert("streaming".to_string(), serde_json::Value::Bool(true));
                metadata.insert("chunk".to_string(), serde_json::Value::Bool(true));
                
                let chunk = AgentResponseChunk {
                    chunk_type: ChunkType::Text,
                    content: chunk_content,
                    metadata: Some(metadata),
                };
                
                if tx_clone.send(chunk).await.is_err() {
                    break;
                }
                
                // Small delay to simulate streaming
                tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
            }
            
            // Send completion chunk
            let completion_chunk = AgentResponseChunk {
                chunk_type: ChunkType::Complete,
                content: String::new(),
                metadata: Some(response.message.metadata.clone()),
            };
            
            let _ = tx_clone.send(completion_chunk).await;
        });
        
        Ok(rx)
    }
}

// Export the plugin factory function
#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin + Send + Sync> {
    Box::new(AmazonQPlugin::new())
}