use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;

use super::LlmEmbeddingClient;
use async_openai::{
    config::OpenAIConfig,
    Client as OpenAIClient,
    error::OpenAIError,
    types::{ CreateEmbeddingRequest, EmbeddingInput },
};
use phf::phf_map;

static DEFAULT_EMBEDDING_DIMENSIONS: phf::Map<&str, u32> = phf_map! {
    "text-embedding-3-small" => 1536,
    "text-embedding-3-large" => 3072,
    "text-embedding-ada-002" => 1536,
};

pub struct Client {
    client: async_openai::Client<OpenAIConfig>,
}

impl Client {
    pub fn new(address: Option<String>, api_config: Option<super::LlmApiConfig>) -> Result<Self> {
        let config = match api_config {
            Some(super::LlmApiConfig::OpenAi(config)) => config,
            Some(_) => bail!("unexpected config type, expected OpenAiConfig"),
            None => super::OpenAiConfig::default(),
        };

        let mut openai_config = OpenAIConfig::new();
        if let Some(address) = address {
            openai_config = openai_config.with_api_base(address);
        }
        if let Some(org_id) = config.org_id {
            openai_config = openai_config.with_org_id(org_id);
        }
        if let Some(project_id) = config.project_id {
            openai_config = openai_config.with_org_id(project_id);
        }

        // Verify API key is set
        if std::env::var("OPENAI_API_KEY").is_err() {
            bail!("OPENAI_API_KEY environment variable must be set");
        }
        Ok(Self {
            // OpenAI client will use OPENAI_API_KEY and OPENAI_API_BASE env variables by default
            client: OpenAIClient::with_config(openai_config),
        })
    }
}

// Retryable implementation removed for simplicity

#[async_trait]
impl LlmEmbeddingClient for Client {
    async fn embed_text<'req>(
        &self,
        request: super::LlmEmbeddingRequest<'req>,
    ) -> Result<super::LlmEmbeddingResponse> {
        let response = self.client
            .embeddings()
            .create(CreateEmbeddingRequest {
                model: request.model.to_string(),
                input: EmbeddingInput::String(request.text.to_string()),
                ..Default::default()
            })
            .await
            .map_err(|e| anyhow!("Failed to create embedding: {}", e))?;
        Ok(super::LlmEmbeddingResponse {
            embedding: response
                .data
                .into_iter()
                .next()
                .ok_or_else(|| anyhow::anyhow!("No embedding returned from OpenAI"))?
                .embedding,
        })
    }

    fn get_default_embedding_dimension(&self, model: &str) -> Option<u32> {
        DEFAULT_EMBEDDING_DIMENSIONS.get(model).copied()
    }
}
