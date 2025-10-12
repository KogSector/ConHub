use crate::config::IndexerConfig;

pub struct EmbeddingService {
    config: IndexerConfig,
}

impl EmbeddingService {
    pub fn new(config: IndexerConfig) -> Self {
        Self { config }
    }

    /// Generate embeddings for text
    /// In a full implementation, this would call OpenAI, Cohere, or a local model
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        // Check if we have an API key
        if self.config.openai_api_key.is_none() {
            // Return a dummy embedding for development
            return Ok(self.generate_dummy_embedding(text));
        }

        // In production, call the actual embedding API
        // For now, return dummy embeddings
        Ok(self.generate_dummy_embedding(text))
    }

    /// Generate batch embeddings for multiple texts
    pub async fn generate_batch_embeddings(
        &self,
        texts: &[String],
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
        let mut embeddings = Vec::with_capacity(texts.len());
        
        for text in texts {
            embeddings.push(self.generate_embedding(text).await?);
        }

        Ok(embeddings)
    }

    /// Generate a deterministic dummy embedding based on text content
    /// This is only for development/testing
    fn generate_dummy_embedding(&self, text: &str) -> Vec<f32> {
        const EMBEDDING_DIM: usize = 384; // Common size for many models
        
        let mut embedding = vec![0.0f32; EMBEDDING_DIM];
        
        // Use text hash to generate pseudo-random but deterministic values
        let hash = self.simple_hash(text);
        
        for (i, val) in embedding.iter_mut().enumerate() {
            let seed = (hash.wrapping_add(i as u64)) as f32;
            *val = (seed.sin() * 0.5).clamp(-1.0, 1.0);
        }
        
        // Normalize the embedding
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }
        
        embedding
    }

    /// Simple hash function for generating deterministic dummy embeddings
    fn simple_hash(&self, text: &str) -> u64 {
        let mut hash = 5381u64;
        for byte in text.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
        }
        hash
    }

    /// Store embedding in vector database
    pub async fn store_embedding(
        &self,
        id: &str,
        embedding: &[f32],
        metadata: std::collections::HashMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // If we have Qdrant configured, store there
        if self.config.qdrant_url.is_some() {
            log::debug!("Storing embedding {} in Qdrant", id);
            // TODO: Implement Qdrant storage
        } else {
            log::debug!("Qdrant not configured, skipping embedding storage for {}", id);
        }

        Ok(())
    }

    /// Search for similar embeddings
    pub async fn search_similar(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<(String, f32)>, Box<dyn std::error::Error>> {
        // If we have Qdrant configured, search there
        if self.config.qdrant_url.is_some() {
            log::debug!("Searching similar embeddings in Qdrant");
            // TODO: Implement Qdrant search
        }

        // Return empty results for now
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_embedding() {
        let config = crate::config::IndexerConfig::from_env();
        let service = EmbeddingService::new(config);
        
        let text = "This is a test text";
        let embedding = service.generate_embedding(text).await.unwrap();
        
        assert_eq!(embedding.len(), 384);
        
        // Check that it's normalized
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_batch_embeddings() {
        let config = crate::config::IndexerConfig::from_env();
        let service = EmbeddingService::new(config);
        
        let texts = vec![
            "First text".to_string(),
            "Second text".to_string(),
            "Third text".to_string(),
        ];
        
        let embeddings = service.generate_batch_embeddings(&texts).await.unwrap();
        
        assert_eq!(embeddings.len(), 3);
        assert_eq!(embeddings[0].len(), 384);
    }
}
