use serde::{Deserialize, Serialize};
use reqwest::Client;
use async_trait::async_trait;
use crate::llm::{LlmEmbeddingClient, LlmEmbeddingRequest, LlmEmbeddingResponse};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIEmbeddingRequest {
    pub model: String,
    pub input: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIEmbeddingResponse {
    pub object: String,
    pub data: Vec<OpenAIEmbeddingData>,
    pub model: String,
    pub usage: OpenAIUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIEmbeddingData {
    pub object: String,
    pub embedding: Vec<f32>,
    pub index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIUsage {
    pub prompt_tokens: usize,
    pub total_tokens: usize,
}

pub struct OpenAIEmbeddingClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl OpenAIEmbeddingClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }

    pub async fn generate_embeddings(&self, model: &str, texts: Vec<String>, dimensions: Option<u32>) -> Result<Vec<Vec<f32>>> {
        let request = OpenAIEmbeddingRequest {
            model: model.to_string(),
            input: texts,
            dimensions,
        };

        let response = self.client
            .post(&format!("{}/embeddings", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("OpenAI API error: {}", error_text));
        }

        let embedding_response: OpenAIEmbeddingResponse = response.json().await?;
        
        Ok(embedding_response.data
            .into_iter()
            .map(|data| data.embedding)
            .collect())
    }
}

#[async_trait]
impl LlmEmbeddingClient for OpenAIEmbeddingClient {
    async fn embed_text<'req>(&self, request: LlmEmbeddingRequest<'req>) -> Result<LlmEmbeddingResponse> {
        let embeddings = self.generate_embeddings(
            request.model,
            vec![request.text.to_string()],
            request.output_dimension
        ).await?;
        
        if let Some(embedding) = embeddings.into_iter().next() {
            Ok(LlmEmbeddingResponse { embedding })
        } else {
            Err(anyhow::anyhow!("No embedding returned from OpenAI API"))
        }
    }

    fn get_default_embedding_dimension(&self, model: &str) -> Option<u32> {
        match model {
            "text-embedding-3-large" => Some(3072),
            "text-embedding-3-small" => Some(1536),
            "text-embedding-ada-002" => Some(1536),
            _ => Some(1536),
        }
    }
}
