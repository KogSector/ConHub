use anyhow::{anyhow, Result};
use crate::services::llm::openai;
use crate::llm::{LlmEmbeddingClient, LlmEmbeddingRequest};

/// A service that uses a selected LLM client to generate embeddings.
pub struct LlmEmbeddingService {
    client: Box<dyn LlmEmbeddingClient>,
    model: String,
}

impl LlmEmbeddingService {
    /// Creates a new embedding service with a client for the specified provider.
    pub fn new(provider: &str, model: &str) -> Result<Self> {
        let client: Box<dyn LlmEmbeddingClient> = match provider {
            "openai" => Box::new(openai::Client::new(None, None)?),
            _ => return Err(anyhow!("Unsupported LLM provider: {}", provider)),
        };

        Ok(Self {
            client,
            model: model.to_string(),
        })
    }

    /// Generates embeddings for a batch of texts using the selected LLM client.
    pub async fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();

        for text in texts.iter() {
            if text.is_empty() {
                return Err(anyhow!("Text cannot be empty"));
            }

            let request = LlmEmbeddingRequest {
                model: &self.model,
                text: &text,
                task_type: None,
                output_dimension: None,
            };

            let response = self.client.embed_text(request).await?;
            embeddings.push(response.embedding);
        }

        Ok(embeddings)
    }

    /// Returns the default embedding dimension for the current model.
    pub fn get_dimension(&self) -> Option<u32> {
        self.client.get_default_embedding_dimension(&self.model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require valid API keys to be set in the environment.
    // e.g., OPENAI_API_KEY, GEMINI_API_KEY, VOYAGE_API_KEY

    #[tokio::test]
    #[ignore]
    async fn test_openai_embedding() {
        let service = LlmEmbeddingService::new("openai", "text-embedding-3-small").unwrap();
        let embeddings = service
            .generate_embeddings(vec!["test text".to_string()])
            .await
            .unwrap();

        assert_eq!(embeddings.len(), 1);
        assert_eq!(embeddings[0].len(), 1536);
    }

}
