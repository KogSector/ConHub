use crate::{Plugin, PluginResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: String,
    pub content: String,
    pub role: MessageRole,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Message role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// Agent response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub message: AgentMessage,
    pub actions: Vec<AgentAction>,
    pub confidence: f32,
    pub processing_time_ms: u64,
}

/// Agent action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAction {
    pub action_type: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub description: String,
}

/// Agent capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    pub supports_chat: bool,
    pub supports_code_generation: bool,
    pub supports_code_analysis: bool,
    pub supports_file_operations: bool,
    pub supports_web_search: bool,
    pub supports_function_calling: bool,
    pub max_context_length: Option<u32>,
    pub supported_languages: Vec<String>,
}

/// Conversation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    pub conversation_id: String,
    pub messages: Vec<AgentMessage>,
    pub workspace_path: Option<String>,
    pub active_files: Vec<String>,
    pub user_preferences: HashMap<String, serde_json::Value>,
}

/// Agent plugin trait
#[async_trait]
pub trait AgentPlugin: Plugin {
    /// Get agent capabilities
    fn capabilities(&self) -> AgentCapabilities;
    
    /// Process a message and generate response
    async fn process_message(
        &self,
        message: AgentMessage,
        context: ConversationContext,
    ) -> PluginResult<AgentResponse>;
    
    /// Execute an action
    async fn execute_action(&self, action: AgentAction) -> PluginResult<serde_json::Value>;
    
    /// Get available functions/tools
    async fn get_available_functions(&self) -> PluginResult<Vec<AgentFunction>>;
    
    /// Stream response (for real-time chat)
    async fn stream_response(
        &self,
        message: AgentMessage,
        context: ConversationContext,
    ) -> PluginResult<tokio::sync::mpsc::Receiver<AgentResponseChunk>>;
}

/// Agent function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value, // JSON Schema
}

/// Streaming response chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponseChunk {
    pub chunk_type: ChunkType,
    pub content: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkType {
    Text,
    Action,
    Error,
    Complete,
}

/// Agent plugin factory
pub trait AgentPluginFactory: Send + Sync {
    fn create(&self) -> Box<dyn AgentPlugin>;
    fn agent_type(&self) -> &str;
}