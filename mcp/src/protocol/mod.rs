use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// MCP Protocol Version
pub const MCP_VERSION: &str = "1.0.0";

/// MCP Resource - represents data that can be accessed by AI agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub mime_type: String,
    pub metadata: Option<serde_json::Value>,
}

/// Content of a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    pub uri: String,
    pub mime_type: String,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
}

/// MCP Tool - represents a capability that can be executed by AI agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value, // JSON Schema
    pub returns: Option<serde_json::Value>, // JSON Schema
}

/// Tool execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionRequest {
    pub tool_name: String,
    pub arguments: serde_json::Value,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// MCP Prompt - represents a reusable prompt template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub name: String,
    pub description: String,
    pub template: String,
    pub parameters: Vec<PromptParameter>,
}

/// Parameter for a prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptParameter {
    pub name: String,
    pub description: String,
    pub required: bool,
}

/// Registered AI agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub agent_type: String,
    pub capabilities: Vec<String>,
    pub endpoint: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_active: chrono::DateTime<chrono::Utc>,
}

/// Context shared between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedContext {
    pub id: Uuid,
    pub context_type: String,
    pub data: serde_json::Value,
    pub source_agent_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Sync request for context between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    pub source_agent_id: Uuid,
    pub target_agent_ids: Option<Vec<Uuid>>, // None means broadcast to all
    pub context: serde_json::Value,
    pub context_type: String,
}

/// Notification for agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentNotification {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub notification_type: String,
    pub data: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
