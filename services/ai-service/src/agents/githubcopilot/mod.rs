use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::time::Instant;

use crate::agents::core::{AIAgentConnector, AIAgent, AgentStatus, AgentQueryRequest, AgentQueryResponse, AgentUsage};

pub struct GitHubCopilotAgent {
    client: Client,
    token: Option<String>,
    agent_info: AIAgent,
}

impl GitHubCopilotAgent {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            token: None,
            agent_info: AIAgent {
                id: "github_copilot".to_string(),
                agent_type: "github_copilot".to_string(),
                name: "GitHub Copilot".to_string(),
                description: "AI pair programmer for code assistance and completion".to_string(),
                capabilities: vec![
                    "code_completion".to_string(),
                    "code_explanation".to_string(),
                    "bug_fixing".to_string(),
                    "code_review".to_string(),
                    "documentation".to_string(),
                ],
                is_connected: false,
                status: AgentStatus::Disconnected,
            },
        }
    }

    async fn validate_token(&self, token: &str) -> Result<bool, Box<dyn Error>> {
        
        let response = self.client
            .get("https://api.github.com/user")
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "ConHub/1.0")
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    async fn send_copilot_request(&self, prompt: &str, context: Option<&str>) -> Result<String, Box<dyn Error>> {
        
        
        let full_prompt = if let Some(ctx) = context {
            format!("Context:\n{}\n\nPrompt: {}", ctx, prompt)
        } else {
            prompt.to_string()
        };

        
        let response = format!(
            "GitHub Copilot suggests:\n\n```\n
            prompt,
            self.generate_mock_response(&full_prompt)
        );

        Ok(response)
    }

    fn generate_mock_response(&self, prompt: &str) -> String {
        
        if prompt.to_lowercase().contains("function") {
            "function exampleFunction() {\n    
        } else if prompt.to_lowercase().contains("class") {
            "class ExampleClass {\n    constructor() {\n        
        } else if prompt.to_lowercase().contains("bug") || prompt.to_lowercase().contains("fix") {
            "
        } else {
            "
        }
    }
}

#[async_trait]
impl AIAgentConnector for GitHubCopilotAgent {
    async fn connect(&self, credentials: &HashMap<String, String>) -> Result<bool, Box<dyn Error>> {
        let token = credentials.get("github_token")
            .or_else(|| credentials.get("access_token"))
            .ok_or("GitHub token is required for Copilot connection")?;

        if self.validate_token(token).await? {
            
            println!("GitHub Copilot connected successfully");
            Ok(true)
        } else {
            Err("Invalid GitHub token for Copilot access".into())
        }
    }

    async fn disconnect(&self) -> Result<bool, Box<dyn Error>> {
        println!("GitHub Copilot disconnected");
        Ok(true)
    }

    async fn query(&self, prompt: &str, context: Option<&str>) -> Result<String, Box<dyn Error>> {
        if self.token.is_none() {
            return Err("GitHub Copilot not connected. Please connect first.".into());
        }

        self.send_copilot_request(prompt, context).await
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

impl Default for GitHubCopilotAgent {
    fn default() -> Self {
        Self::new()
    }
}


#[allow(dead_code)]
pub async fn query_with_code_context(
    agent: &GitHubCopilotAgent,
    request: AgentQueryRequest,
    file_context: Option<&str>,
    language: Option<&str>,
) -> Result<AgentQueryResponse, Box<dyn Error>> {
    let start_time = Instant::now();
    
    let context = match (request.context.as_deref(), file_context, language) {
        (Some(ctx), Some(file), Some(lang)) => {
            Some(format!("Language: {}\nFile Context:\n{}\nAdditional Context:\n{}", lang, file, ctx))
        }
        (Some(ctx), Some(file), None) => {
            Some(format!("File Context:\n{}\nAdditional Context:\n{}", file, ctx))
        }
        (Some(ctx), None, Some(lang)) => {
            Some(format!("Language: {}\nContext:\n{}", lang, ctx))
        }
        (None, Some(file), Some(lang)) => {
            Some(format!("Language: {}\nFile Context:\n{}", lang, file))
        }
        (Some(ctx), None, None) => Some(ctx.to_string()),
        (None, Some(file), None) => Some(file.to_string()),
        (None, None, Some(lang)) => Some(format!("Language: {}", lang)),
        (None, None, None) => None,
    };

    let response = agent.query(&request.prompt, context.as_deref()).await?;
    let elapsed = start_time.elapsed();

    Ok(AgentQueryResponse {
        response,
        usage: AgentUsage {
            tokens_used: request.prompt.len() as u32, 
            response_time_ms: elapsed.as_millis() as u64,
            model: Some("github-copilot".to_string()),
        },
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("agent_type".to_string(), json!("github_copilot"));
            if let Some(lang) = language {
                meta.insert("language".to_string(), json!(lang));
            }
            meta
        },
    })
}