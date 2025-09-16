use reqwest::Client;
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::models::{AgentRecord, AgentContext, AgentInvokeRequest, AgentInvokeResponse, AgentInvokeUsage};

#[derive(Debug, Clone)]
pub struct AgentService {
    client: Client,
}

#[derive(Serialize, Deserialize)]
pub struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

#[derive(Serialize, Deserialize)]
pub struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
pub struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Serialize, Deserialize)]
pub struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Serialize, Deserialize)]
pub struct OpenAIUsage {
    total_tokens: u32,
}

#[derive(Serialize, Deserialize)]
pub struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    temperature: Option<f32>,
}

#[derive(Serialize, Deserialize)]
pub struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
pub struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    usage: AnthropicUsage,
}

#[derive(Serialize, Deserialize)]
pub struct AnthropicContent {
    text: String,
}

#[derive(Serialize, Deserialize)]
pub struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

impl AgentService {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client }
    }

    pub async fn invoke_agent(
        &self,
        agent: &AgentRecord,
        request: &AgentInvokeRequest,
        context: Option<&AgentContext>,
    ) -> Result<AgentInvokeResponse, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        let response = match agent.agent_type.as_str() {
            "openai" => self.invoke_openai(agent, request, context).await?,
            "anthropic" => self.invoke_anthropic(agent, request, context).await?,
            "custom" => self.invoke_custom(agent, request, context).await?,
            _ => return Err("Unsupported agent type".into()),
        };
        
        let response_time = start_time.elapsed().as_millis() as u64;
        
        Ok(AgentInvokeResponse {
            response: response.response,
            usage: AgentInvokeUsage {
                tokens_used: response.usage.tokens_used,
                response_time_ms: response_time,
            },
            context_used: response.context_used,
        })
    }

    async fn invoke_openai(
        &self,
        agent: &AgentRecord,
        request: &AgentInvokeRequest,
        context: Option<&AgentContext>,
    ) -> Result<AgentInvokeResponse, Box<dyn std::error::Error>> {
        let model = agent.config.model.as_ref().unwrap_or(&"gpt-4".to_string()).clone();
        let temperature = agent.config.temperature.unwrap_or(0.7);
        let max_tokens = agent.config.max_tokens.unwrap_or(1000);
        
        let mut messages = vec![];
        
        // Add system message with context if available
        if let Some(ctx) = context {
            let context_summary = self.format_context_for_openai(ctx);
            messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: format!(
                    "You are an AI assistant with access to the following context:\n{}",
                    context_summary
                ),
            });
        }
        
        // Add custom instructions if any
        if let Some(instructions) = &agent.config.custom_instructions {
            messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: instructions.clone(),
            });
        }
        
        // Add user message
        messages.push(OpenAIMessage {
            role: "user".to_string(),
            content: request.message.clone(),
        });
        
        let payload = OpenAIRequest {
            model,
            messages,
            max_tokens: Some(max_tokens),
            temperature: Some(temperature),
        };
        
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", agent.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("OpenAI API error: {}", error_text).into());
        }
        
        let openai_response: OpenAIResponse = response.json().await?;
        
        let response_text = openai_response.choices
            .first()
            .ok_or("No response from OpenAI")?
            .message
            .content
            .clone();
        
        let context_used = context.map(|_| vec!["repositories".to_string(), "documents".to_string(), "urls".to_string()])
            .unwrap_or_default();
        
        Ok(AgentInvokeResponse {
            response: response_text,
            usage: AgentInvokeUsage {
                tokens_used: openai_response.usage.total_tokens,
                response_time_ms: 0, // Will be set by caller
            },
            context_used,
        })
    }

    async fn invoke_anthropic(
        &self,
        agent: &AgentRecord,
        request: &AgentInvokeRequest,
        context: Option<&AgentContext>,
    ) -> Result<AgentInvokeResponse, Box<dyn std::error::Error>> {
        let model = agent.config.model.as_ref().unwrap_or(&"claude-3-sonnet-20240229".to_string()).clone();
        let temperature = agent.config.temperature.unwrap_or(0.7);
        let max_tokens = agent.config.max_tokens.unwrap_or(1000);
        
        let mut content = request.message.clone();
        
        // Add context if available
        if let Some(ctx) = context {
            let context_summary = self.format_context_for_anthropic(ctx);
            content = format!("Context:\n{}\n\nUser request: {}", context_summary, content);
        }
        
        // Add custom instructions if any
        if let Some(instructions) = &agent.config.custom_instructions {
            content = format!("Instructions: {}\n\n{}", instructions, content);
        }
        
        let messages = vec![AnthropicMessage {
            role: "user".to_string(),
            content,
        }];
        
        let payload = AnthropicRequest {
            model,
            messages,
            max_tokens,
            temperature: Some(temperature),
        };
        
        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &agent.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Anthropic API error: {}", error_text).into());
        }
        
        let anthropic_response: AnthropicResponse = response.json().await?;
        
        let response_text = anthropic_response.content
            .first()
            .ok_or("No response from Anthropic")?
            .text
            .clone();
        
        let total_tokens = anthropic_response.usage.input_tokens + anthropic_response.usage.output_tokens;
        let context_used = context.map(|_| vec!["repositories".to_string(), "documents".to_string(), "urls".to_string()])
            .unwrap_or_default();
        
        Ok(AgentInvokeResponse {
            response: response_text,
            usage: AgentInvokeUsage {
                tokens_used: total_tokens,
                response_time_ms: 0, // Will be set by caller
            },
            context_used,
        })
    }

    async fn invoke_custom(
        &self,
        agent: &AgentRecord,
        request: &AgentInvokeRequest,
        context: Option<&AgentContext>,
    ) -> Result<AgentInvokeResponse, Box<dyn std::error::Error>> {
        let endpoint = agent.endpoint.as_ref().ok_or("Custom agent requires endpoint")?;
        
        let mut payload = json!({
            "message": request.message,
            "config": agent.config
        });
        
        if let Some(ctx) = context {
            payload["context"] = json!(ctx);
        }
        
        let mut request_builder = self.client.post(endpoint)
            .header("Content-Type", "application/json")
            .json(&payload);
        
        // Add authentication if API key is provided
        if !agent.api_key.is_empty() {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", agent.api_key));
        }
        
        let response = request_builder.send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Custom agent API error: {}", error_text).into());
        }
        
        let response_json: Value = response.json().await?;
        
        let response_text = response_json.get("response")
            .and_then(|v| v.as_str())
            .unwrap_or("No response from custom agent")
            .to_string();
        
        let tokens_used = response_json.get("tokens_used")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        
        let context_used = context.map(|_| vec!["custom".to_string()])
            .unwrap_or_default();
        
        Ok(AgentInvokeResponse {
            response: response_text,
            usage: AgentInvokeUsage {
                tokens_used,
                response_time_ms: 0, // Will be set by caller
            },
            context_used,
        })
    }

    fn format_context_for_openai(&self, context: &AgentContext) -> String {
        let mut context_str = String::new();
        
        if !context.repositories.is_empty() {
            context_str.push_str("## Repositories:\n");
            for repo in &context.repositories {
                context_str.push_str(&format!(
                    "- {}: {} ({})\n  Recent files: {}\n  Recent commits: {}\n",
                    repo.name,
                    repo.description.as_ref().unwrap_or(&"No description".to_string()),
                    repo.language,
                    repo.recent_files.join(", "),
                    repo.recent_commits.join(", ")
                ));
            }
        }
        
        if !context.documents.is_empty() {
            context_str.push_str("\n## Documents:\n");
            for doc in &context.documents {
                context_str.push_str(&format!(
                    "- {}: {} ({})\n  Summary: {}\n  Tags: {}\n",
                    doc.name,
                    doc.doc_type,
                    doc.id,
                    doc.summary.as_ref().unwrap_or(&"No summary".to_string()),
                    doc.tags.join(", ")
                ));
            }
        }
        
        if !context.urls.is_empty() {
            context_str.push_str("\n## URLs:\n");
            for url in &context.urls {
                context_str.push_str(&format!(
                    "- {}: {}\n  Summary: {}\n  Tags: {}\n",
                    url.title.as_ref().unwrap_or(&"No title".to_string()),
                    url.url,
                    url.summary.as_ref().unwrap_or(&"No summary".to_string()),
                    url.tags.join(", ")
                ));
            }
        }
        
        context_str
    }

    fn format_context_for_anthropic(&self, context: &AgentContext) -> String {
        // Similar to OpenAI but potentially with different formatting preferences
        self.format_context_for_openai(context)
    }

    pub async fn test_agent_connection(&self, agent: &AgentRecord) -> Result<bool, Box<dyn std::error::Error>> {
        match agent.agent_type.as_str() {
            "openai" => self.test_openai_connection(agent).await,
            "anthropic" => self.test_anthropic_connection(agent).await,
            "custom" => self.test_custom_connection(agent).await,
            _ => Err("Unsupported agent type".into()),
        }
    }

    async fn test_openai_connection(&self, agent: &AgentRecord) -> Result<bool, Box<dyn std::error::Error>> {
        let response = self.client
            .get("https://api.openai.com/v1/models")
            .header("Authorization", format!("Bearer {}", agent.api_key))
            .send()
            .await?;
        
        Ok(response.status().is_success())
    }

    async fn test_anthropic_connection(&self, agent: &AgentRecord) -> Result<bool, Box<dyn std::error::Error>> {
        // Anthropic doesn't have a simple health check endpoint, so we'll make a minimal request
        let messages = vec![AnthropicMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];
        
        let payload = AnthropicRequest {
            model: "claude-3-haiku-20240307".to_string(), // Use the cheapest model for testing
            messages,
            max_tokens: 10,
            temperature: Some(0.0),
        };
        
        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &agent.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;
        
        Ok(response.status().is_success())
    }

    async fn test_custom_connection(&self, agent: &AgentRecord) -> Result<bool, Box<dyn std::error::Error>> {
        let endpoint = agent.endpoint.as_ref().ok_or("Custom agent requires endpoint")?;
        
        // Try a simple health check or minimal request
        let health_url = format!("{}/health", endpoint.trim_end_matches('/'));
        let response = self.client
            .get(&health_url)
            .send()
            .await;
        
        match response {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => {
                // If health endpoint doesn't exist, try the main endpoint with a test request
                let test_payload = json!({
                    "message": "test",
                    "test": true
                });
                
                let response = self.client
                    .post(endpoint)
                    .header("Content-Type", "application/json")
                    .json(&test_payload)
                    .send()
                    .await?;
                
                Ok(response.status().is_success())
            }
        }
    }
}

// Original AI service functions for backward compatibility
pub async fn ask_ai_question(
    client: &Client,
    haystack_url: &str,
    request: &crate::models::SearchRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let payload = json!({
        "query": request.query,
        "top_k": request.limit.unwrap_or(5)
    });
    
    let response = client
        .post(&format!("{}/ask", haystack_url))
        .json(&payload)
        .send()
        .await?;
    
    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Err(format!("AI service error: {}", response.status()).into())
    }
}