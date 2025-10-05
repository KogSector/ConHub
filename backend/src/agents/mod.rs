pub mod core;
pub mod githubcopilot;
pub mod amazonq;
pub mod cursoride;
pub mod openai;
pub mod cline;

// Re-export commonly used types
#[allow(unused_imports)]
pub use core::{AIAgentConnector, AIAgent, AgentStatus, AIAgentFactory, AgentQueryRequest, AgentQueryResponse, AgentUsage};
