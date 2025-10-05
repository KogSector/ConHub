use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::time::Instant;

use crate::agents::core::{AIAgentConnector, AIAgent, AgentStatus, AgentQueryRequest, AgentQueryResponse, AgentUsage};

pub struct ClineAgent {
    client: Client,
    token: Option<String>,
    agent_info: AIAgent,
}

impl ClineAgent {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            token: None,
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

    async fn validate_token(&self, token: &str) -> Result<bool, Box<dyn Error>> {
        // In a real implementation, you would validate the Cline API token
        // For now, we'll simulate a successful validation
        Ok(!token.is_empty())
    }

    async fn send_cline_request(&self, prompt: &str, context: Option<&str>) -> Result<String, Box<dyn Error>> {
        let full_prompt = if let Some(ctx) = context {
            format!("Context:\n{}\n\nPrompt: {}", ctx, prompt)
        } else {
            prompt.to_string()
        };

        // Simulate Cline response
        let response = format!(
            "Cline suggests:\n\n```\n// Based on your request: {}\n// This is a simulated response from Cline\n\n{}\n```",
            prompt,
            self.generate_mock_response(&full_prompt)
        );

        Ok(response)
    }

    fn generate_mock_response(&self, prompt: &str) -> String {
        if prompt.to_lowercase().contains("refactor") {
            "// Refactored code suggestion:\n// 1. Extracted logic into a separate function\n// 2. Improved variable names for clarity\n// 3. Added error handling".to_string()
        } else if prompt.to_lowercase().contains("class") {
            "class ExampleClass {\n    constructor() {\n        // Initialized class\n    }\n}".to_string()
        } else if prompt.to_lowercase().contains("debug") {
            "// Debugging suggestion:\n// 1. Check for off-by-one errors in loops\n// 2. Verify API endpoint and payload\n// 3. Ensure type consistency".to_string()
        } else {
            "// Cline suggestion based on your context".to_string()
        }
    }
}

#[async_trait]
impl AIAgentConnector for ClineAgent {
    async fn connect(&self, credentials: &HashMap<String, String>) -> Result<bool, Box<dyn Error>> {
        let token = credentials.get("cline_token")
            .ok_or("Cline token is required for connection")?;

        if self.validate_token(token).await? {
            println!("Cline connected successfully");
            Ok(true)
        } else {
            Err("Invalid Cline token".into())
        }
    }

    async fn disconnect(&self) -> Result<bool, Box<dyn Error>> {
        println!("Cline disconnected");
        Ok(true)
    }

    async fn query(&self, prompt: &str, context: Option<&str>) -> Result<String, Box<dyn Error>> {
        if self.token.is_none() {
            return Err("Cline not connected. Please connect first.".into());
        }

        self.send_cline_request(prompt, context).await
    }

    fn get_agent(&self) -> AIAgent {
        self.agent_info.clone()
    }

    async fn test_connection(&self) -> Result<bool, Box<dyn Error>> {
        if let Some(token) = &self.token {
            self.validate_token(token).await
        } else {
            Ok(false)
        }
    }
}

impl Default for ClineAgent {
    fn default() -> Self {
        Self::new()
    }
}
