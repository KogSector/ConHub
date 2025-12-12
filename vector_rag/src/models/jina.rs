use serde::{Deserialize, Serialize};
use reqwest::Client;
use async_trait::async_trait;
use crate::llm::{LlmEmbeddingClient, LlmEmbeddingRequest, LlmEmbeddingResponse};
use anyhow::Result;
use std::time::Duration;

/// Configuration for external embedding API (provider-agnostic)
#[derive(Debug, Clone)]
pub struct ExternalEmbeddingConfig {
    pub api_url: String,
    pub model: String,
    pub api_key: String,
    pub timeout_ms: u64,
}

impl ExternalEmbeddingConfig {
    pub fn from_env() -> Result<Self> {
        let api_url = std::env::var("EMBEDDINGS_API_URL")
            .unwrap_or_else(|_| "https://api.jina.ai/v1/embeddings".to_string());
        let model = std::env::var("EMBEDDINGS_MODEL")
            .unwrap_or_else(|_| "jina-embeddings-v3".to_string());
        let api_key = std::env::var("EXTERNAL_SEARCH_API_KEY")
            .map_err(|_| anyhow::anyhow!("EXTERNAL_SEARCH_API_KEY environment variable not set"))?;
        let timeout_ms = std::env::var("EXTERNAL_SEARCH_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30000);
        
        Ok(Self {
            api_url,
            model,
            api_key,
            timeout_ms,
        })
    }
}

/// Configuration for external rerank API (provider-agnostic)
#[derive(Debug, Clone)]
pub struct ExternalRerankConfig {
    pub api_url: String,
    pub model: String,
    pub api_key: String,
    pub timeout_ms: u64,
}

impl ExternalRerankConfig {
    pub fn from_env() -> Result<Self> {
        let api_url = std::env::var("RERANK_API_URL")
            .unwrap_or_else(|_| "https://api.jina.ai/v1/rerank".to_string());
        let model = std::env::var("RERANK_MODEL")
            .unwrap_or_else(|_| "jina-reranker-v2-base-multilingual".to_string());
        let api_key = std::env::var("EXTERNAL_SEARCH_API_KEY")
            .map_err(|_| anyhow::anyhow!("EXTERNAL_SEARCH_API_KEY environment variable not set"))?;
        let timeout_ms = std::env::var("EXTERNAL_SEARCH_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30000);
        
        Ok(Self {
            api_url,
            model,
            api_key,
            timeout_ms,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingApiRequest {
    pub model: String,
    pub input: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingApiResponse {
    pub model: String,
    pub object: String,
    pub usage: EmbeddingUsage,
    pub data: Vec<EmbeddingData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingData {
    pub object: String,
    pub index: usize,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingUsage {
    pub total_tokens: usize,
    #[serde(default)]
    pub prompt_tokens: usize,
}

/// Rerank API request (provider-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankApiRequest {
    pub model: String,
    pub query: String,
    pub documents: Vec<String>,
    pub top_n: usize,
    #[serde(default)]
    pub return_documents: bool,
}

/// Rerank API response (provider-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankApiResponse {
    pub model: String,
    pub results: Vec<RerankResult>,
    #[serde(default)]
    pub usage: Option<RerankUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResult {
    pub index: usize,
    pub relevance_score: f32,
    #[serde(default)]
    pub document: Option<RerankDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankDocument {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankUsage {
    #[serde(default)]
    pub total_tokens: usize,
}

/// External embedding client using HTTP API (provider-agnostic)
pub struct HttpEmbeddingClient {
    client: Client,
    config: ExternalEmbeddingConfig,
}

impl HttpEmbeddingClient {
    pub fn new(config: ExternalEmbeddingConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .unwrap_or_else(|_| Client::new());
        
        Self { client, config }
    }
    
    pub fn from_env() -> Result<Self> {
        let config = ExternalEmbeddingConfig::from_env()?;
        Ok(Self::new(config))
    }

    pub async fn generate_embeddings(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        self.generate_embeddings_with_model(&self.config.model, texts).await
    }

    pub async fn generate_embeddings_with_model(&self, model: &str, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let request = EmbeddingApiRequest {
            model: model.to_string(),
            input: texts,
        };

        let response = self.client
            .post(&self.config.api_url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Embedding API error ({}): {}", status, error_text));
        }

        let embedding_response: EmbeddingApiResponse = response.json().await?;
        
        // Sort by index to ensure correct order
        let mut data = embedding_response.data;
        data.sort_by_key(|d| d.index);
        
        Ok(data.into_iter().map(|d| d.embedding).collect())
    }
    
    pub fn get_model(&self) -> &str {
        &self.config.model
    }
}

/// External rerank client using HTTP API (provider-agnostic)
pub struct HttpRerankClient {
    client: Client,
    config: ExternalRerankConfig,
}

impl HttpRerankClient {
    pub fn new(config: ExternalRerankConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .unwrap_or_else(|_| Client::new());
        
        Self { client, config }
    }
    
    pub fn from_env() -> Result<Self> {
        let config = ExternalRerankConfig::from_env()?;
        Ok(Self::new(config))
    }

    /// Rerank documents by relevance to query
    pub async fn rerank(&self, query: &str, documents: Vec<String>, top_n: usize) -> Result<Vec<RerankResult>> {
        if documents.is_empty() {
            return Ok(Vec::new());
        }

        let request = RerankApiRequest {
            model: self.config.model.clone(),
            query: query.to_string(),
            documents,
            top_n,
            return_documents: false,
        };

        let response = self.client
            .post(&self.config.api_url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Rerank API error ({}): {}", status, error_text));
        }

        let rerank_response: RerankApiResponse = response.json().await?;
        Ok(rerank_response.results)
    }
    
    pub fn get_model(&self) -> &str {
        &self.config.model
    }
}

// Legacy alias for backward compatibility
pub type JinaEmbeddingClient = HttpEmbeddingClient;
pub type JinaEmbeddingRequest = EmbeddingApiRequest;
pub type JinaEmbeddingResponse = EmbeddingApiResponse;
pub type JinaEmbeddingData = EmbeddingData;
pub type JinaUsage = EmbeddingUsage;

#[async_trait]
impl LlmEmbeddingClient for HttpEmbeddingClient {
    async fn embed_text<'req>(&self, request: LlmEmbeddingRequest<'req>) -> Result<LlmEmbeddingResponse> {
        let embeddings = self.generate_embeddings_with_model(
            request.model,
            vec![request.text.to_string()]
        ).await?;
        
        if let Some(embedding) = embeddings.into_iter().next() {
            Ok(LlmEmbeddingResponse { embedding })
        } else {
            Err(anyhow::anyhow!("No embedding returned from external API"))
        }
    }

    fn get_default_embedding_dimension(&self, model: &str) -> Option<u32> {
        // Common embedding dimensions for various models
        match model {
            // Jina models
            "jina-embeddings-v3" => Some(1024),
            "jina-embeddings-v2-base-en" => Some(768),
            "jina-embeddings-v2-small-en" => Some(512),
            "jina-clip-v1" => Some(768),
            // OpenAI models
            "text-embedding-3-small" => Some(1536),
            "text-embedding-3-large" => Some(3072),
            "text-embedding-ada-002" => Some(1536),
            // Cohere models
            "embed-english-v3.0" => Some(1024),
            "embed-multilingual-v3.0" => Some(1024),
            // Default
            _ => Some(1024),
        }
    }
}
