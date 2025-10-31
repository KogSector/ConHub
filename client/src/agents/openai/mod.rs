use async_trait::async_trait;
use std::collections::HashMap;
use std::error::Error;
use tokio::sync::Mutex;

use crate::agents::core::{AIAgentConnector, AIAgent, AgentStatus};
use crate::agents::llm::openai::Client;
use crate::llm::{LlmGenerationClient, LlmGenerateRequest};

pub struct OpenAIAgent {
    agent_info: Mutex<AIAgent>,
    client: Mutex<Option<Client>>,
}

impl OpenAIAgent {
    pub fn new() -> Self {
        Self {
            agent_info: Mutex::new(AIAgent {
                id: "openai".to_string(),
                agent_type: "openai".to_string(),
                name: "OpenAI GPT".to_string(),
                description: "OpenAI's GPT models for general AI assistance".to_string(),
                capabilities: vec![
                    "text_generation".to_string(),
                    "code_assistance".to_string(),
                    "question_answering".to_string(),
                    "summarization".to_string(),
                    "translation".to_string(),
                ],
                is_connected: false,
                status: AgentStatus::Disconnected,
            }),
            client: Mutex::new(None),
        }
    }
}

#[async_trait]
impl AIAgentConnector for OpenAIAgent {
    async fn connect(&self, _credentials: &HashMap<String, String>) -> Result<bool, Box<dyn Error>> {
        let client = Client::new(None, None)?;
        *self.client.lock().await = Some(client);
        let mut agent_info = self.agent_info.lock().await;
        agent_info.is_connected = true;
        agent_info.status = AgentStatus::Connected;
        Ok(true)
    }

    async fn disconnect(&self) -> Result<bool, Box<dyn Error>> {
        *self.client.lock().await = None;
        let mut agent_info = self.agent_info.lock().await;
        agent_info.is_connected = false;
        agent_info.status = AgentStatus::Disconnected;
        Ok(true)
    }

    async fn query(&self, prompt: &str, context: Option<&str>) -> Result<String, Box<dyn Error>> {
        let client = self.client.lock().await;
        if let Some(client) = &*client {
            let llm_request = LlmGenerateRequest {
                model: "gpt-4",
                user_prompt: std::borrow::Cow::Borrowed(prompt),
                system_prompt: context.map(std::borrow::Cow::Borrowed),
                image: None,
                output_format: None,
            };
            let response = client.generate(llm_request).await?;
            Ok(response.text)
        } else {
            Err("Not connected to OpenAI".into())
        }
    }

    fn get_agent(&self) -> AIAgent {
        // This should be synchronous according to the trait
        // We'll need to handle this differently or change the trait
        AIAgent {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            agent_type: "openai".to_string(),
            description: "OpenAI's GPT models for general AI assistance".to_string(),
            status: AgentStatus::Disconnected,
            is_connected: false,
            capabilities: vec!["text_generation".to_string()],
        }
    }

    async fn test_connection(&self) -> Result<bool, Box<dyn Error>> {
        Ok(self.client.lock().await.is_some())
    }
}

impl Default for OpenAIAgent {
    fn default() -> Self {
        Self::new()
    }
}
