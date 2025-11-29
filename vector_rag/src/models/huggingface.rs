use serde::{Deserialize, Serialize};
use reqwest::Client;
use async_trait::async_trait;
use crate::llm::{LlmEmbeddingClient, LlmEmbeddingRequest, LlmEmbeddingResponse};
use anyhow::Result;
use std::time::Duration;

/// Request body for HuggingFace Inference API
#[derive(Debug, Clone, Serialize)]
pub struct HuggingFaceEmbeddingRequest {
    pub inputs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<HuggingFaceOptions>,
}

/// Options for HuggingFace Inference API
#[derive(Debug, Clone, Serialize)]
pub struct HuggingFaceOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_for_model: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_cache: Option<bool>,
}

/// HuggingFace Inference Endpoints can return different response formats
/// This handles both direct array and structured response formats
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum HuggingFaceEmbeddingResponse {
    /// Direct array of embeddings: [[f32, f32, ...], [f32, f32, ...]]
    DirectArray(Vec<Vec<f32>>),
    /// Single embedding (for single input): [f32, f32, ...]
    SingleEmbedding(Vec<f32>),
    /// Structured response with embeddings field
    Structured { embeddings: Vec<Vec<f32>> },
    /// Error response from HuggingFace
    Error { error: String },
}

/// Configuration for a specific HuggingFace model endpoint
#[derive(Debug, Clone)]
pub struct HuggingFaceModelConfig {
    pub model_id: String,
    pub endpoint_url: Option<String>,
    pub dimension: u32,
}

impl Default for HuggingFaceModelConfig {
    fn default() -> Self {
        Self {
            model_id: "BAAI/bge-base-en-v1.5".to_string(),
            endpoint_url: None,
            dimension: 768,
        }
    }
}

/// HuggingFace embedding client that supports both Inference API and Inference Endpoints
pub struct HuggingFaceEmbeddingClient {
    client: Client,
    api_token: String,
    /// Base URL for HuggingFace Inference API
    inference_api_base: String,
    /// Optional dedicated endpoint URL (for Inference Endpoints)
    dedicated_endpoint: Option<String>,
    /// Model configurations for known models
    model_configs: std::collections::HashMap<String, HuggingFaceModelConfig>,
}

impl HuggingFaceEmbeddingClient {
    /// Create a new HuggingFace client with API token
    pub fn new(api_token: String) -> Self {
        Self::with_config(api_token, None, None)
    }

    /// Create a new HuggingFace client with custom base URL
    pub fn with_base_url(api_token: String, base_url: String) -> Self {
        Self::with_config(api_token, Some(base_url), None)
    }

    /// Create a new HuggingFace client for a dedicated Inference Endpoint
    pub fn with_endpoint(api_token: String, endpoint_url: String) -> Self {
        Self::with_config(api_token, None, Some(endpoint_url))
    }

    /// Full configuration constructor
    pub fn with_config(
        api_token: String,
        base_url: Option<String>,
        dedicated_endpoint: Option<String>,
    ) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        let inference_api_base = base_url
            .unwrap_or_else(|| "https://api-inference.huggingface.co".to_string());

        // Pre-populate known model configurations
        let mut model_configs = std::collections::HashMap::new();
        
        // BGE models (BAAI)
        model_configs.insert("BAAI/bge-base-en-v1.5".to_string(), HuggingFaceModelConfig {
            model_id: "BAAI/bge-base-en-v1.5".to_string(),
            endpoint_url: None,
            dimension: 768,
        });
        model_configs.insert("BAAI/bge-large-en-v1.5".to_string(), HuggingFaceModelConfig {
            model_id: "BAAI/bge-large-en-v1.5".to_string(),
            endpoint_url: None,
            dimension: 1024,
        });
        model_configs.insert("BAAI/bge-small-en-v1.5".to_string(), HuggingFaceModelConfig {
            model_id: "BAAI/bge-small-en-v1.5".to_string(),
            endpoint_url: None,
            dimension: 384,
        });
        model_configs.insert("BAAI/bge-m3".to_string(), HuggingFaceModelConfig {
            model_id: "BAAI/bge-m3".to_string(),
            endpoint_url: None,
            dimension: 1024,
        });
        
        // Sentence Transformers
        model_configs.insert("sentence-transformers/all-MiniLM-L6-v2".to_string(), HuggingFaceModelConfig {
            model_id: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            endpoint_url: None,
            dimension: 384,
        });
        model_configs.insert("sentence-transformers/all-mpnet-base-v2".to_string(), HuggingFaceModelConfig {
            model_id: "sentence-transformers/all-mpnet-base-v2".to_string(),
            endpoint_url: None,
            dimension: 768,
        });
        
        // Jina code embedding
        model_configs.insert("jinaai/jina-embeddings-v2-base-code".to_string(), HuggingFaceModelConfig {
            model_id: "jinaai/jina-embeddings-v2-base-code".to_string(),
            endpoint_url: None,
            dimension: 768,
        });
        model_configs.insert("jinaai/jina-embeddings-v2-base-en".to_string(), HuggingFaceModelConfig {
            model_id: "jinaai/jina-embeddings-v2-base-en".to_string(),
            endpoint_url: None,
            dimension: 768,
        });
        
        // E5 models
        model_configs.insert("intfloat/e5-base-v2".to_string(), HuggingFaceModelConfig {
            model_id: "intfloat/e5-base-v2".to_string(),
            endpoint_url: None,
            dimension: 768,
        });
        model_configs.insert("intfloat/e5-large-v2".to_string(), HuggingFaceModelConfig {
            model_id: "intfloat/e5-large-v2".to_string(),
            endpoint_url: None,
            dimension: 1024,
        });
        model_configs.insert("intfloat/multilingual-e5-large".to_string(), HuggingFaceModelConfig {
            model_id: "intfloat/multilingual-e5-large".to_string(),
            endpoint_url: None,
            dimension: 1024,
        });

        Self {
            client,
            api_token,
            inference_api_base,
            dedicated_endpoint,
            model_configs,
        }
    }

    /// Get the URL for a specific model
    fn get_model_url(&self, model_id: &str) -> String {
        // If we have a dedicated endpoint, always use it
        if let Some(ref endpoint) = self.dedicated_endpoint {
            return endpoint.clone();
        }

        // Check if the model has a custom endpoint configured
        if let Some(config) = self.model_configs.get(model_id) {
            if let Some(ref endpoint) = config.endpoint_url {
                return endpoint.clone();
            }
        }

        // Default to Inference API
        format!("{}/models/{}", self.inference_api_base, model_id)
    }

    /// Generate embeddings for multiple texts
    pub async fn generate_embeddings(&self, model_id: &str, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let url = self.get_model_url(model_id);
        
        let request_body = HuggingFaceEmbeddingRequest {
            inputs: texts.clone(),
            options: Some(HuggingFaceOptions {
                wait_for_model: Some(true),
                use_cache: Some(true),
            }),
        };

        log::debug!("HuggingFace embedding request to {}: {} texts", url, texts.len());

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "HuggingFace API error ({}): {}",
                status,
                error_text
            ));
        }

        let response_text = response.text().await?;
        
        // Try to parse the response in different formats
        let embeddings = self.parse_response(&response_text, texts.len())?;
        
        log::debug!("HuggingFace returned {} embeddings", embeddings.len());
        
        Ok(embeddings)
    }

    /// Parse HuggingFace response which can come in different formats
    fn parse_response(&self, response_text: &str, expected_count: usize) -> Result<Vec<Vec<f32>>> {
        // First try to parse as the response enum
        match serde_json::from_str::<HuggingFaceEmbeddingResponse>(response_text) {
            Ok(HuggingFaceEmbeddingResponse::DirectArray(embeddings)) => {
                Ok(embeddings)
            }
            Ok(HuggingFaceEmbeddingResponse::SingleEmbedding(embedding)) => {
                // Single embedding returned, wrap in vec
                Ok(vec![embedding])
            }
            Ok(HuggingFaceEmbeddingResponse::Structured { embeddings }) => {
                Ok(embeddings)
            }
            Ok(HuggingFaceEmbeddingResponse::Error { error }) => {
                Err(anyhow::anyhow!("HuggingFace API error: {}", error))
            }
            Err(parse_error) => {
                // Try parsing as nested array (some models return [[[f32]]])
                if let Ok(nested) = serde_json::from_str::<Vec<Vec<Vec<f32>>>>(response_text) {
                    // Flatten one level if we got nested arrays
                    let flattened: Vec<Vec<f32>> = nested.into_iter().flatten().collect();
                    if !flattened.is_empty() {
                        return Ok(flattened);
                    }
                }
                
                Err(anyhow::anyhow!(
                    "Failed to parse HuggingFace response: {}. Response: {}",
                    parse_error,
                    &response_text[..response_text.len().min(500)]
                ))
            }
        }
    }

    /// Register a custom model configuration
    pub fn register_model(&mut self, config: HuggingFaceModelConfig) {
        self.model_configs.insert(config.model_id.clone(), config);
    }

    /// Get dimension for a known model
    pub fn get_model_dimension(&self, model_id: &str) -> Option<u32> {
        self.model_configs.get(model_id).map(|c| c.dimension)
    }
}

#[async_trait]
impl LlmEmbeddingClient for HuggingFaceEmbeddingClient {
    async fn embed_text<'req>(&self, request: LlmEmbeddingRequest<'req>) -> Result<LlmEmbeddingResponse> {
        let embeddings = self.generate_embeddings(
            request.model,
            vec![request.text.to_string()]
        ).await?;
        
        if let Some(embedding) = embeddings.into_iter().next() {
            Ok(LlmEmbeddingResponse { embedding })
        } else {
            Err(anyhow::anyhow!("No embedding returned from HuggingFace API"))
        }
    }

    fn get_default_embedding_dimension(&self, model: &str) -> Option<u32> {
        // Check our known model configs first
        if let Some(dim) = self.get_model_dimension(model) {
            return Some(dim);
        }

        // Fallback to common patterns
        match model {
            // BGE models
            m if m.contains("bge-large") => Some(1024),
            m if m.contains("bge-base") => Some(768),
            m if m.contains("bge-small") => Some(384),
            m if m.contains("bge-m3") => Some(1024),
            
            // Sentence transformers
            m if m.contains("MiniLM-L6") => Some(384),
            m if m.contains("MiniLM-L12") => Some(384),
            m if m.contains("mpnet-base") => Some(768),
            
            // Jina
            m if m.contains("jina-embeddings-v2") => Some(768),
            
            // E5 models
            m if m.contains("e5-large") => Some(1024),
            m if m.contains("e5-base") => Some(768),
            m if m.contains("e5-small") => Some(384),
            
            // Default
            _ => Some(768),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_dimension_lookup() {
        let client = HuggingFaceEmbeddingClient::new("test_token".to_string());
        
        assert_eq!(client.get_default_embedding_dimension("BAAI/bge-base-en-v1.5"), Some(768));
        assert_eq!(client.get_default_embedding_dimension("BAAI/bge-large-en-v1.5"), Some(1024));
        assert_eq!(client.get_default_embedding_dimension("sentence-transformers/all-MiniLM-L6-v2"), Some(384));
        assert_eq!(client.get_default_embedding_dimension("jinaai/jina-embeddings-v2-base-code"), Some(768));
    }

    #[test]
    fn test_url_construction() {
        let client = HuggingFaceEmbeddingClient::new("test_token".to_string());
        let url = client.get_model_url("BAAI/bge-base-en-v1.5");
        assert_eq!(url, "https://api-inference.huggingface.co/models/BAAI/bge-base-en-v1.5");
    }

    #[test]
    fn test_dedicated_endpoint() {
        let client = HuggingFaceEmbeddingClient::with_endpoint(
            "test_token".to_string(),
            "https://my-endpoint.huggingface.cloud".to_string()
        );
        let url = client.get_model_url("any-model");
        assert_eq!(url, "https://my-endpoint.huggingface.cloud");
    }
}
