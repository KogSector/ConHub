use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::collections::HashMap;
use async_trait::async_trait;
use crate::llm::{LlmEmbeddingClient, LlmEmbeddingRequest, LlmEmbeddingResponse};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenEmbeddingRequest {
    pub model: String,
    pub input: Vec<String>,
    pub encoding_format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenEmbeddingResponse {
    pub object: String,
    pub data: Vec<QwenEmbeddingData>,
    pub model: String,
    pub usage: QwenUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenEmbeddingData {
    pub object: String,
    pub embedding: Vec<f32>,
    pub index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenUsage {
    pub prompt_tokens: usize,
    pub total_tokens: usize,
}

pub struct QwenEmbeddingClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl QwenEmbeddingClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://dashscope.aliyuncs.com/api/v1".to_string(),
        }
    }

    pub async fn generate_embeddings(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
        let request = QwenEmbeddingRequest {
            model: "text-embedding-v3".to_string(),
            input: texts,
            encoding_format: Some("float".to_string()),
        };

        let response = self.client
            .post(&format!("{}/services/embeddings/text-embedding/text-embedding", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Qwen API error: {}", error_text).into());
        }

        let embedding_response: QwenEmbeddingResponse = response.json().await?;
        
        Ok(embedding_response.data
            .into_iter()
            .map(|data| data.embedding)
            .collect())
    }
}

#[async_trait]
impl LlmEmbeddingClient for QwenEmbeddingClient {
    async fn embed_text(&self, request: LlmEmbeddingRequest<'_>) -> Result<LlmEmbeddingResponse> {
        let embeddings = self.generate_embeddings(vec![request.text.to_string()]).await?;
        
        if let Some(embedding) = embeddings.into_iter().next() {
            Ok(LlmEmbeddingResponse {
                embedding,
                model: request.model.to_string(),
                usage: None,
            })
        } else {
            Err(anyhow::anyhow!("No embedding returned from Qwen API"))
        }
    }

    fn get_default_embedding_dimension(&self, _model: &str) -> Option<u32> {
        Some(1536) // Qwen text-embedding-v3 dimension
    }
}