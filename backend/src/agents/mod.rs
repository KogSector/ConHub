pub mod core;
pub mod githubcopilot;
pub mod amazonq;
pub mod cursoride;
pub mod openai;

// Re-export commonly used types
pub use core::{AIAgentConnector, AIAgent, AgentStatus, AIAgentFactory, AgentQueryRequest, AgentQueryResponse, AgentUsage};