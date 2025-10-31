use crate::prelude::*;
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};

use crate::llm::{LlmGenerationClient, detect_image_mime_type};
use crate::llm::{LlmGenerateResponse, LlmGenerateRequest, OutputFormat, LlmApiConfig, OpenAiConfig};
use crate::base::json_schema::ToJsonSchemaOptions;

#[derive(Debug, Serialize)]
pub struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIMessage {
    role: String,
    content: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIUsage {
    total_tokens: u32,
}

pub struct Client {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl Client {
    pub fn new(address: Option<String>, api_config: Option<LlmApiConfig>) -> Result<Self> {
        let _config = match api_config {
            Some(LlmApiConfig::OpenAi(config)) => config,
            Some(_) => anyhow::bail!("unexpected config type, expected OpenAiConfig"),
            None => OpenAiConfig::default(),
        };

        // Get API key from environment
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| anyhow::anyhow!("OPENAI_API_KEY environment variable must be set"))?;

        let base_url = address.unwrap_or_else(|| "https://api.openai.com/v1".to_string());

        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
            base_url,
        })
    }
}

fn create_openai_request(
    request: &LlmGenerateRequest,
) -> Result<OpenAIRequest, Box<dyn std::error::Error>> {
    let mut messages = Vec::new();

    // Add system prompt if provided
    if let Some(system) = &request.system_prompt {
        messages.push(OpenAIMessage {
            role: "system".to_string(),
            content: serde_json::Value::String(system.to_string()),
        });
    }

    // Add user message
    let content = match &request.image {
        Some(img_bytes) => {
            let base64_image = general_purpose::STANDARD.encode(img_bytes.as_ref());
            let mime_type = detect_image_mime_type(img_bytes.as_ref())?;
            let image_url = format!("data:{mime_type};base64,{base64_image}");
            
            serde_json::json!([
                {
                    "type": "text",
                    "text": request.user_prompt
                },
                {
                    "type": "image_url",
                    "image_url": {
                        "url": image_url
                    }
                }
            ])
        }
        None => serde_json::Value::String(request.user_prompt.to_string()),
    };

    messages.push(OpenAIMessage {
        role: "user".to_string(),
        content,
    });

    Ok(OpenAIRequest {
        model: request.model.to_string(),
        messages,
        max_tokens: Some(4000),
    })
}

#[async_trait]
impl LlmGenerationClient for Client {
    async fn generate(
        &self,
        request: LlmGenerateRequest<'_>,
    ) -> Result<LlmGenerateResponse, Box<dyn std::error::Error>> {
        let openai_request = create_openai_request(&request)?;
        
        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("OpenAI API error: {}", error_text).into());
        }

        let openai_response: OpenAIResponse = response.json().await?;

        let text = openai_response
            .choices
            .into_iter()
            .next()
            .and_then(|choice| {
                match choice.message.content {
                    serde_json::Value::String(s) => Some(s),
                    _ => None,
                }
            })
            .ok_or_else(|| anyhow::anyhow!("No response from OpenAI"))?;

        Ok(LlmGenerateResponse { text })
    }

    fn json_schema_options(&self) -> ToJsonSchemaOptions {
        ToJsonSchemaOptions {
            fields_always_required: true,
            supports_format: false,
            extract_descriptions: false,
            top_level_must_be_object: true,
        }
    }
}
