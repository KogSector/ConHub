use serde::{Deserialize, Serialize};
use reqwest::Client;
use async_trait::async_trait;
use crate::llm::{LlmEmbeddingClient, LlmEmbeddingRequest, LlmEmbeddingResponse};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JinaEmbeddingRequest {
    pub model: String,
    pub input: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JinaEmbeddingResponse {
    pub model: String,
    pub object: String,
    pub usage: JinaUsage,
    pub data: Vec<JinaEmbeddingData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JinaEmbeddingData {
    pub object: String,
    pub index: usize,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JinaUsage {
    pub total_tokens: usize,
    pub prompt_tokens: usize,
}

pub struct JinaEmbeddingClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl JinaEmbeddingClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.jina.ai/v1".to_string(),
        }
    }

    pub async fn generate_embeddings(&self, model: &str, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let request = JinaEmbeddingRequest {
            model: model.to_string(),
            input: texts,
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
            return Err(anyhow::anyhow!("Jina AI API error: {}", error_text));
        }

        let embedding_response: JinaEmbeddingResponse = response.json().await?;
        
        Ok(embedding_response.data
            .into_iter()
            .map(|data| data.embedding)
            .collect())
    }
}

#[async_trait]
impl LlmEmbeddingClient for JinaEmbeddingClient {
    async fn embed_text<'req>(&self, request: LlmEmbeddingRequest<'req>) -> Result<LlmEmbeddingResponse> {
        let embeddings = self.generate_embeddings(
            request.model,
            vec![request.text.to_string()]
        ).await?;
        
        if let Some(embedding) = embeddings.into_iter().next() {
            Ok(LlmEmbeddingResponse { embedding })
        } else {
            Err(anyhow::anyhow!("No embedding returned from Jina AI API"))
        }
    }

    fn get_default_embedding_dimension(&self, model: &str) -> Option<u32> {
        match model {
            "jina-embeddings-v2-base-en" => Some(768),
            "jina-embeddings-v2-small-en" => Some(512),
            _ => Some(768),
        }
    }
}
