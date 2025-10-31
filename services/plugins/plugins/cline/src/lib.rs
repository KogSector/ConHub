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
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClineConfig {
    pub api_key: String,
    pub api_url: Option<String>, // Custom API endpoint if needed
    pub model: Option<String>,   // AI model to use
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub workspace_path: Option<String>, // Path to the workspace/project
    pub allowed_commands: Option<Vec<String>>, // Allowed shell commands
    pub file_extensions: Option<Vec<String>>, // Allowed file extensions to work with
}

#[derive(Debug, Serialize, Deserialize)]
struct ClineRequest {
    message: String,
    context: Option<ConversationContext>,
    workspace_path: Option<String>,
    files: Option<Vec<String>>, // File paths to include in context
    command_history: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClineResponse {
    response: String,
    actions: Option<Vec<ClineAction>>,
    files_modified: Option<Vec<String>>,
    commands_executed: Option<Vec<String>>,
    suggestions: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClineAction {
    action_type: String,
    description: String,
    parameters: HashMap<String, Value>,
    requires_confirmation: bool,
}

pub struct ClinePlugin {
    config: Arc<RwLock<Option<ClineConfig>>>,
    client: Client,
    status: Arc<RwLock<PluginStatus>>,
    conversation_history: Arc<RwLock<Vec<AgentMessage>>>,
}

impl ClinePlugin {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(None)),
            client: Client::new(),
            status: Arc::new(RwLock::new(PluginStatus::Inactive)),
            conversation_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    async fn get_config(&self) -> Result<ClineConfig> {
        let config_guard = self.config.read().await;
        config_guard
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow!("Plugin not configured"))
    }

    async fn make_api_request(&self, request: ClineRequest) -> Result<ClineResponse> {
        let config = self.get_config().await?;
        let api_url = config.api_url.as_deref().unwrap_or("https://api.anthropic.com/v1/messages");
        
        // Build the request payload for Claude/Anthropic API
        let mut payload = json!({
            "model": config.model.as_deref().unwrap_or("claude-3-sonnet-20240229"),
            "max_tokens": config.max_tokens.unwrap_or(4096),
            "messages": [
                {
                    "role": "user",
                    "content": self.build_cline_prompt(&request).await?
                }
            ]
        });
        
        if let Some(temp) = config.temperature {
            payload["temperature"] = json!(temp);
        }
        
        let response = self.client
            .post(api_url)
            .header("x-api-key", &config.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&payload)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Cline API error {}: {}", status, error_text));
        }
        
        let api_response: Value = response.json().await?;
        
        // Parse Claude response and convert to ClineResponse
        let content = api_response["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid response format"))?;
        
        // Parse the response to extract actions, file modifications, etc.
        let cline_response = self.parse_cline_response(content).await?;
        
        Ok(cline_response)
    }

    async fn build_cline_prompt(&self, request: &ClineRequest) -> Result<String> {
        let config = self.get_config().await?;
        
        let mut prompt = String::new();
        
        // System prompt for Cline behavior
        prompt.push_str("You are Cline, an AI coding assistant that helps with software development tasks. ");
        prompt.push_str("You can read, write, and modify files, execute shell commands, and provide coding assistance. ");
        prompt.push_str("Always be helpful, accurate, and follow best practices.\n\n");
        
        // Add workspace context if available
        if let Some(workspace) = &config.workspace_path {
            prompt.push_str(&format!("Current workspace: {}\n", workspace));
        }
        
        // Add conversation context
        if let Some(context) = &request.context {
            if !context.messages.is_empty() {
                prompt.push_str("Previous conversation:\n");
                for msg in &context.messages {
                    prompt.push_str(&format!("{:?}: {}\n", msg.role, msg.content));
                }
                prompt.push_str("\n");
            }
        }
        
        // Add file context if provided
        if let Some(files) = &request.files {
            prompt.push_str("Relevant files:\n");
            for file_path in files {
                // In a real implementation, you would read the file contents
                prompt.push_str(&format!("- {}\n", file_path));
            }
            prompt.push_str("\n");
        }
        
        // Add command history if provided
        if let Some(commands) = &request.command_history {
            prompt.push_str("Recent commands:\n");
            for cmd in commands {
                prompt.push_str(&format!("$ {}\n", cmd));
            }
            prompt.push_str("\n");
        }
        
        // Add the current request
        prompt.push_str(&format!("User request: {}\n", request.message));
        
        // Add instructions for response format
        prompt.push_str("\nPlease provide your response and any actions you want to take. ");
        prompt.push_str("If you need to modify files or run commands, describe them clearly.");
        
        Ok(prompt)
    }

    async fn parse_cline_response(&self, content: &str) -> Result<ClineResponse> {
        // This is a simplified parser. In a real implementation, you would
        // have more sophisticated parsing to extract actions, file modifications, etc.
        
        let mut actions = Vec::new();
        let mut files_modified = Vec::new();
        let mut commands_executed = Vec::new();
        let mut suggestions = Vec::new();
        
        // Look for action patterns in the response
        if content.contains("```bash") || content.contains("```sh") {
            // Extract shell commands
            // This is a simplified extraction - you'd want more robust parsing
            commands_executed.push("example_command".to_string());
        }
        
        if content.contains("create file") || content.contains("modify file") {
            // Extract file operations
            files_modified.push("example_file.rs".to_string());
        }
        
        // Extract suggestions
        if content.contains("I suggest") || content.contains("You might want to") {
            suggestions.push("Consider adding error handling".to_string());
        }
        
        Ok(ClineResponse {
            response: content.to_string(),
            actions: if actions.is_empty() { None } else { Some(actions) },
            files_modified: if files_modified.is_empty() { None } else { Some(files_modified) },
            commands_executed: if commands_executed.is_empty() { None } else { Some(commands_executed) },
            suggestions: if suggestions.is_empty() { None } else { Some(suggestions) },
        })
    }

    async fn execute_cline_action(&self, action: &ClineAction) -> Result<Value> {
        let config = self.get_config().await?;
        
        match action.action_type.as_str() {
            "execute_command" => {
                let command = action.parameters.get("command")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing command parameter"))?;
                
                // Check if command is allowed
                if let Some(allowed) = &config.allowed_commands {
                    let command_name = command.split_whitespace().next().unwrap_or("");
                    if !allowed.contains(&command_name.to_string()) {
                        return Err(anyhow!("Command '{}' is not allowed", command_name));
                    }
                }
                
                // In a real implementation, you would execute the command safely
                info!("Would execute command: {}", command);
                Ok(json!({
                    "status": "simulated",
                    "command": command,
                    "output": "Command execution simulated"
                }))
            }
            "modify_file" => {
                let file_path = action.parameters.get("file_path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing file_path parameter"))?;
                
                let content = action.parameters.get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing content parameter"))?;
                
                // Check file extension if restrictions are in place
                if let Some(allowed_exts) = &config.file_extensions {
                    let ext = std::path::Path::new(file_path)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");
                    
                    if !allowed_exts.contains(&ext.to_string()) {
                        return Err(anyhow!("File extension '{}' is not allowed", ext));
                    }
                }
                
                // In a real implementation, you would modify the file
                info!("Would modify file: {}", file_path);
                Ok(json!({
                    "status": "simulated",
                    "file_path": file_path,
                    "content_length": content.len()
                }))
            }
            "read_file" => {
                let file_path = action.parameters.get("file_path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing file_path parameter"))?;
                
                // In a real implementation, you would read the file
                info!("Would read file: {}", file_path);
                Ok(json!({
                    "status": "simulated",
                    "file_path": file_path,
                    "content": "File content would be here"
                }))
            }
            _ => {
                Err(anyhow!("Unknown action type: {}", action.action_type))
            }
        }
    }
}

#[async_trait]
impl Plugin for ClinePlugin {
    fn metadata(&self) -> &PluginMetadata {
        static METADATA: std::sync::OnceLock<PluginMetadata> = std::sync::OnceLock::new();
        METADATA.get_or_init(|| PluginMetadata {
            id: "cline".to_string(),
            name: "Cline".to_string(),
            version: "1.0.0".to_string(),
            description: "AI coding assistant for software development tasks".to_string(),
            author: "ConHub Team".to_string(),
            plugin_type: conhub_plugins::PluginType::Agent,
            capabilities: vec![
                "code_generation".to_string(),
                "file_operations".to_string(),
                "command_execution".to_string(),
                "code_review".to_string(),
                "debugging".to_string(),
            ],
            config_schema: Some(json!({
                "type": "object",
                "properties": {
                    "api_key": {
                        "type": "string",
                        "description": "API key for the AI service (e.g., Anthropic Claude)",
                        "required": true
                    },
                    "api_url": {
                        "type": "string",
                        "description": "Custom API endpoint URL (optional)"
                    },
                    "model": {
                        "type": "string",
                        "description": "AI model to use",
                        "default": "claude-3-sonnet-20240229"
                    },
                    "max_tokens": {
                        "type": "integer",
                        "description": "Maximum tokens for responses",
                        "default": 4096
                    },
                    "temperature": {
                        "type": "number",
                        "description": "Response creativity (0.0 to 1.0)",
                        "default": 0.7
                    },
                    "workspace_path": {
                        "type": "string",
                        "description": "Path to the workspace/project directory"
                    },
                    "allowed_commands": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "List of allowed shell commands"
                    },
                    "file_extensions": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "Allowed file extensions to work with"
                    }
                },
                "required": ["api_key"]
            })),
        })
    }

    async fn initialize(&mut self, config: PluginConfig) -> Result<(), conhub_plugins::error::PluginError> {
        info!("Initializing Cline plugin");
        
        let cline_config: ClineConfig = serde_json::from_value(
            config.settings.get("cline").unwrap_or(&json!({})).clone()
        ).map_err(|e| conhub_plugins::error::PluginError::ConfigurationError(e.to_string()))?;
        
        let mut config_guard = self.config.write().await;
        *config_guard = Some(cline_config);
        
        let mut status_guard = self.status.write().await;
        *status_guard = PluginStatus::Inactive;
        
        info!("Cline plugin initialized successfully");
        Ok(())
    }

    async fn start(&mut self) -> Result<(), conhub_plugins::error::PluginError> {
        info!("Starting Cline plugin");
        
        let config_guard = self.config.read().await;
        if config_guard.is_none() {
            return Err(conhub_plugins::error::PluginError::InitializationFailed("Plugin not initialized".to_string()));
        }
        
        let mut status_guard = self.status.write().await;
        *status_guard = PluginStatus::Active;
        
        info!("Cline plugin started successfully");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), conhub_plugins::error::PluginError> {
        info!("Stopping Cline plugin");
        
        let mut status_guard = self.status.write().await;
        *status_guard = PluginStatus::Inactive;
        
        info!("Cline plugin stopped");
        Ok(())
    }

    fn status(&self) -> PluginStatus {
        match self.status.try_read() {
            Ok(status) => status.clone(),
            Err(_) => PluginStatus::Error("Failed to read status".to_string()),
        }
    }

    async fn health_check(&self) -> Result<bool, conhub_plugins::error::PluginError> {
        match self.get_config().await {
            Ok(config) => {
                // Test API connectivity with a minimal request
                let test_payload = json!({
                    "model": config.model.as_deref().unwrap_or("claude-3-sonnet-20240229"),
                    "max_tokens": 1,
                    "messages": [
                        {
                            "role": "user",
                            "content": "Hi"
                        }
                    ]
                });
                
                let api_url = config.api_url.as_deref().unwrap_or("https://api.anthropic.com/v1/messages");
                let response = self.client
                    .post(api_url)
                    .header("x-api-key", &config.api_key)
                    .header("anthropic-version", "2023-06-01")
                    .json(&test_payload)
                    .send()
                    .await;
                
                match response {
                    Ok(resp) => Ok(resp.status().is_success()),
                    Err(e) => Err(conhub_plugins::error::PluginError::NetworkError(e.to_string())),
                }
            }
            Err(e) => Err(conhub_plugins::error::PluginError::ConfigurationError(e.to_string())),
        }
    }

    fn validate_config(&self, config: &PluginConfig) -> Result<(), conhub_plugins::error::PluginError> {
        let _: ClineConfig = serde_json::from_value(
            config.settings.get("cline").unwrap_or(&json!({})).clone()
        ).map_err(|e| conhub_plugins::error::PluginError::ValidationError(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl AgentPlugin for ClinePlugin {
    fn capabilities(&self) -> AgentCapabilities {
        AgentCapabilities {
            supports_chat: true,
            supports_code_generation: true,
            supports_code_analysis: true,
            supports_file_operations: true,
            supports_web_search: false,
            supports_function_calling: true,
            max_context_length: Some(100000), // Claude's context window
            supported_languages: vec![
                "rust".to_string(),
                "python".to_string(),
                "javascript".to_string(),
                "typescript".to_string(),
                "java".to_string(),
                "cpp".to_string(),
                "go".to_string(),
                "html".to_string(),
                "css".to_string(),
                "sql".to_string(),
            ],
        }
    }

    async fn process_message(
        &self,
        message: AgentMessage,
        context: ConversationContext,
    ) -> PluginResult<AgentResponse> {
        info!("Processing message with Cline: {}", message.content);
        
        // Add message to conversation history
        let mut history = self.conversation_history.write().await;
        history.push(message.clone());
        
        let request = ClineRequest {
            message: message.content.clone(),
            context: Some(context),
            workspace_path: self.get_config().await.ok().and_then(|c| c.workspace_path),
            files: None, // Could be populated based on context
            command_history: None, // Could be populated from previous actions
        };
        
        let cline_response = self.make_api_request(request).await?;
        
        let mut metadata = HashMap::new();
        if let Some(actions) = &cline_response.actions {
            metadata.insert("actions".to_string(), serde_json::to_value(actions).map_err(|e| anyhow::anyhow!(e))?);
        }
        if let Some(files_modified) = &cline_response.files_modified {
            metadata.insert("files_modified".to_string(), serde_json::to_value(files_modified).map_err(|e| anyhow::anyhow!(e))?);
        }
        if let Some(commands_executed) = &cline_response.commands_executed {
            metadata.insert("commands_executed".to_string(), serde_json::to_value(commands_executed).map_err(|e| anyhow::anyhow!(e))?);
        }
        if let Some(suggestions) = &cline_response.suggestions {
            metadata.insert("suggestions".to_string(), serde_json::to_value(suggestions).map_err(|e| anyhow::anyhow!(e))?);
        }
        
        let response_message = AgentMessage {
            id: uuid::Uuid::new_v4().to_string(),
            content: cline_response.response,
            role: MessageRole::Assistant,
            timestamp: Utc::now(),
            metadata,
        };
        
        let response = AgentResponse {
            message: response_message.clone(),
            actions: vec![], // Could be populated from cline_response.actions
            confidence: 0.9, // High confidence for Cline responses
            processing_time_ms: 0,
        };
        
        // Add response to conversation history
        history.push(response_message);
        
        info!("Cline response generated successfully");
        Ok(response)
    }

    async fn get_available_functions(&self) -> PluginResult<Vec<AgentFunction>> {
        Ok(vec![
            AgentFunction {
                name: "execute_command".to_string(),
                description: "Execute a shell command in the workspace".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The shell command to execute"
                        }
                    },
                    "required": ["command"]
                }),
            },
            AgentFunction {
                name: "modify_file".to_string(),
                description: "Create or modify a file in the workspace".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "Path to the file to modify"
                        },
                        "content": {
                            "type": "string",
                            "description": "New content for the file"
                        }
                    },
                    "required": ["file_path", "content"]
                }),
            },
            AgentFunction {
                name: "read_file".to_string(),
                description: "Read the contents of a file".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "Path to the file to read"
                        }
                    },
                    "required": ["file_path"]
                }),
            },
        ])
    }

    async fn execute_action(
        &self,
        action: AgentAction,
    ) -> PluginResult<Value> {
        info!("Executing Cline action: {}", action.action_type);
        
        let cline_action = ClineAction {
            action_type: action.action_type,
            description: action.description,
            parameters: action.parameters,
            requires_confirmation: false, // Could be configurable
        };
        
        self.execute_cline_action(&cline_action).await.map_err(Into::into)
    }

    async fn stream_response(
        &self,
        message: AgentMessage,
        context: ConversationContext,
    ) -> PluginResult<tokio::sync::mpsc::Receiver<AgentResponseChunk>> {
        info!("Starting streaming response with Cline");
        
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        
        // For this implementation, we'll simulate streaming by chunking the response
        let response = self.process_message(message, context).await?;
        
        // Spawn a task to send chunks
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let words: Vec<&str> = response.message.content.split_whitespace().collect();
            let chunk_size = 5; // Words per chunk
            
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
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
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
    Box::new(ClinePlugin::new())
}