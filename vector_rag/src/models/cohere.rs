use serde::{Deserialize, Serialize};
use reqwest::Client;
use async_trait::async_trait;
use crate::llm::{LlmEmbeddingClient, LlmEmbeddingRequest, LlmEmbeddingResponse};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohereEmbeddingRequest {
    pub model: String,
    pub texts: Vec<String>,
    pub input_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncate: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohereEmbeddingResponse {
    pub id: String,
    pub embeddings: Vec<Vec<f32>>,
    pub texts: Vec<String>,
    pub meta: CohereMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohereMeta {
    pub api_version: CohereApiVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billed_units: Option<CohereBilledUnits>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohereApiVersion {
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohereBilledUnits {
    pub input_tokens: usize,
}

pub struct CohereEmbeddingClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl CohereEmbeddingClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.cohere.ai/v1".to_string(),
        }
    }

    pub async fn generate_embeddings(&self, model: &str, texts: Vec<String>, input_type: &str) -> Result<Vec<Vec<f32>>> {
        let request = CohereEmbeddingRequest {
            model: model.to_string(),
            texts,
            input_type: input_type.to_string(),
            truncate: Some("END".to_string()),
        };

        let response = self.client
            .post(&format!("{}/embed", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Cohere API error: {}", error_text));
        }

        let embedding_response: CohereEmbeddingResponse = response.json().await?;
        
        Ok(embedding_response.embeddings)
    }
}

#[async_trait]
impl LlmEmbeddingClient for CohereEmbeddingClient {
    async fn embed_text<'req>(&self, request: LlmEmbeddingRequest<'req>) -> Result<LlmEmbeddingResponse> {
        let input_type = request.task_type
            .as_ref()
            .map(|t| t.as_ref())
            .unwrap_or("search_document");
            
        let embeddings = self.generate_embeddings(
            request.model,
            vec![request.text.to_string()],
            input_type
        ).await?;
        
        if let Some(embedding) = embeddings.into_iter().next() {
            Ok(LlmEmbeddingResponse { embedding })
        } else {
            Err(anyhow::anyhow!("No embedding returned from Cohere API"))
        }
    }

    fn get_default_embedding_dimension(&self, model: &str) -> Option<u32> {
        match model {
            "embed-english-v3.0" => Some(1024),
            "embed-multilingual-v3.0" => Some(1024),
            "embed-english-light-v3.0" => Some(384),
            "embed-multilingual-light-v3.0" => Some(384),
            _ => Some(1024),
        }
    }
}
