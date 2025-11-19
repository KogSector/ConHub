use serde::{Deserialize, Serialize};
use reqwest::Client;
use async_trait::async_trait;
use crate::llm::{LlmEmbeddingClient, LlmEmbeddingRequest, LlmEmbeddingResponse};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoyageEmbeddingRequest {
    pub model: String,
    pub input: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoyageEmbeddingResponse {
    pub object: String,
    pub data: Vec<VoyageEmbeddingData>,
    pub model: String,
    pub usage: VoyageUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoyageEmbeddingData {
    pub object: String,
    pub embedding: Vec<f32>,
    pub index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoyageUsage {
    pub total_tokens: usize,
}

pub struct VoyageEmbeddingClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl VoyageEmbeddingClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.voyageai.com/v1".to_string(),
        }
    }

    pub async fn generate_embeddings(&self, model: &str, texts: Vec<String>, input_type: Option<&str>) -> Result<Vec<Vec<f32>>> {
        let request = VoyageEmbeddingRequest {
            model: model.to_string(),
            input: texts,
            input_type: input_type.map(|s| s.to_string()),
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
            return Err(anyhow::anyhow!("Voyage AI API error: {}", error_text));
        }

        let embedding_response: VoyageEmbeddingResponse = response.json().await?;
        
        Ok(embedding_response.data
            .into_iter()
            .map(|data| data.embedding)
            .collect())
    }
}

#[async_trait]
impl LlmEmbeddingClient for VoyageEmbeddingClient {
    async fn embed_text<'req>(&self, request: LlmEmbeddingRequest<'req>) -> Result<LlmEmbeddingResponse> {
        let input_type = request.task_type.as_ref().map(|t| t.as_ref());
            
        let embeddings = self.generate_embeddings(
            request.model,
            vec![request.text.to_string()],
            input_type
        ).await?;
        
        if let Some(embedding) = embeddings.into_iter().next() {
            Ok(LlmEmbeddingResponse { embedding })
        } else {
            Err(anyhow::anyhow!("No embedding returned from Voyage AI API"))
        }
    }

    fn get_default_embedding_dimension(&self, model: &str) -> Option<u32> {
        match model {
            "voyage-large-2" | "voyage-code-2" | "voyage-2" => Some(1536),
            "voyage-large-2-instruct" => Some(1024),
            "voyage-law-2" | "voyage-finance-2" => Some(1024),
            _ => Some(1536),
        }
    }
}
