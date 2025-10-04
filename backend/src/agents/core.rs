use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

/// Core trait for all AI agent connectors
#[async_trait]
pub trait AIAgentConnector: Send + Sync {
    /// Connect to the AI agent with credentials
    async fn connect(&self, credentials: &HashMap<String, String>) -> Result<bool, Box<dyn Error>>;
    
    /// Disconnect from the AI agent
    #[allow(dead_code)]
    async fn disconnect(&self) -> Result<bool, Box<dyn Error>>;
    
    /// Query the AI agent with a prompt and optional context
    async fn query(&self, prompt: &str, context: Option<&str>) -> Result<String, Box<dyn Error>>;
    
    /// Get agent information
    fn get_agent(&self) -> AIAgent;
    
    /// Test the connection to the AI agent
    async fn test_connection(&self) -> Result<bool, Box<dyn Error>>;
}

/// AI Agent information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AIAgent {
    pub id: String,
    pub agent_type: String,
    pub name: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub is_connected: bool,
    pub status: AgentStatus,
}

/// Agent connection status
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AgentStatus {
    Connected,
    Disconnected,
    Error(String),
    Connecting,
}

/// Agent query request
#[derive(Debug, Deserialize)]
pub struct AgentQueryRequest {
    pub prompt: String,
    pub context: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

/// Agent query response
#[derive(Debug, Serialize)]
pub struct AgentQueryResponse {
    pub response: String,
    pub usage: AgentUsage,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Usage statistics for agent queries
#[derive(Debug, Serialize)]
pub struct AgentUsage {
    pub tokens_used: u32,
    pub response_time_ms: u64,
    pub model: Option<String>,
}

/// Factory for creating AI agent connectors
pub struct AIAgentFactory;

impl AIAgentFactory {
    pub fn create_agent(agent_type: &str) -> Result<Box<dyn AIAgentConnector>, Box<dyn Error + Send + Sync>> {
        match agent_type {
            "github_copilot" => Ok(Box::new(crate::agents::githubcopilot::GitHubCopilotAgent::new())),
            "amazon_q" => Ok(Box::new(crate::agents::amazonq::AmazonQAgent::new())),
            "cursor_ide" => Ok(Box::new(crate::agents::cursoride::CursorIDEAgent::new())),
            "openai" => Ok(Box::new(crate::agents::openai::OpenAIAgent::new())),
            _ => Err(format!("Unsupported AI agent type: {}", agent_type).into()),
        }
    }

    pub fn list_supported_agents() -> Vec<&'static str> {
        vec!["github_copilot", "amazon_q", "cursor_ide", "openai"]
    }
}