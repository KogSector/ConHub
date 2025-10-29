use async_trait::async_trait;
use std::collections::HashMap;
use std::error::Error;

use crate::agents::core::{AIAgentConnector, AIAgent, AgentStatus};

pub struct ClineAgent {
    agent_info: AIAgent,
}

impl ClineAgent {
    pub fn new() -> Self {
        Self {
            agent_info: AIAgent {
                id: "cline".to_string(),
                agent_type: "cline".to_string(),
                name: "Cline".to_string(),
                description: "AI-powered software engineer for complex tasks".to_string(),
                capabilities: vec![
                    "code_generation".to_string(),
                    "code_refactoring".to_string(),
                    "project_scaffolding".to_string(),
                    "debugging".to_string(),
                    "documentation_generation".to_string(),
                ],
                is_connected: false,
                status: AgentStatus::Disconnected,
            },
        }
    }




}

#[async_trait]
impl AIAgentConnector for ClineAgent {
    async fn connect(&self, _credentials: &HashMap<String, String>) -> Result<bool, Box<dyn Error>> {
        println!("Cline connected successfully");
        Ok(true)
    }

    async fn disconnect(&self) -> Result<bool, Box<dyn Error>> {
        println!("Cline disconnected");
        Ok(true)
    }

    async fn query(&self, prompt: &str, context: Option<&str>) -> Result<String, Box<dyn Error>> {
        let response = format!(
            "Cline response for: '{}' with context: '{}'",
            prompt,
            context.unwrap_or("None")
        );
        Ok(response)
    }

    fn get_agent(&self) -> AIAgent {
        self.agent_info.clone()
    }

    async fn test_connection(&self) -> Result<bool, Box<dyn Error>> {
        Ok(true)
    }
}

impl Default for ClineAgent {
    fn default() -> Self {
        Self::new()
    }
}
