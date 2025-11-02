use anyhow::{anyhow, Result};
use crate::services::llm::openai;
use crate::llm::{LlmEmbeddingClient, LlmEmbeddingRequest};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::Semaphore;
use rayon::prelude::*;
use sha2::{Sha256, Digest};

// Advanced caching structure for embeddings
#[derive(Clone)]
pub struct EmbeddingCache {
    cache: Arc<RwLock<HashMap<String, Vec<f32>>>>,
    max_size: usize,
    access_count: Arc<RwLock<HashMap<String, u64>>>,
}

impl EmbeddingCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            access_count: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get(&self, key: &str) -> Option<Vec<f32>> {
        let cache = self.cache.read().unwrap();
        if let Some(embedding) = cache.get(key) {
            // Update access count
            let mut access = self.access_count.write().unwrap();
            *access.entry(key.to_string()).or_insert(0) += 1;
            Some(embedding.clone())
        } else {
            None
        }
    }

    pub fn insert(&self, key: String, value: Vec<f32>) {
        let mut cache = self.cache.write().unwrap();
        
        // Evict least frequently used items if cache is full
        if cache.len() >= self.max_size {
            self.evict_lfu(&mut cache);
        }
        
        cache.insert(key.clone(), value);
        let mut access = self.access_count.write().unwrap();
        access.insert(key, 1);
    }

    fn evict_lfu(&self, cache: &mut HashMap<String, Vec<f32>>) {
        let access = self.access_count.read().unwrap();
        if let Some((lfu_key, _)) = access.iter().min_by_key(|(_, &count)| count) {
            let lfu_key = lfu_key.clone();
            drop(access);
            cache.remove(&lfu_key);
            let mut access = self.access_count.write().unwrap();
            access.remove(&lfu_key);
        }
    }
}

// Optimized vector operations
pub struct VectorOps;

impl VectorOps {
    /// Compute cosine similarity between two vectors using SIMD-optimized operations
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.par_iter().zip(b.par_iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.par_iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.par_iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }

    /// Normalize vector in-place for better performance
    pub fn normalize_inplace(vector: &mut [f32]) {
        let norm: f32 = vector.par_iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            vector.par_iter_mut().for_each(|x| *x /= norm);
        }
    }

    /// Batch normalize vectors for better throughput
    pub fn batch_normalize(vectors: &mut [Vec<f32>]) {
        vectors.par_iter_mut().for_each(|vector| {
            Self::normalize_inplace(vector);
        });
    }
}

/// A service that uses a selected LLM client to generate embeddings with advanced optimizations.
pub struct LlmEmbeddingService {
    client: Box<dyn LlmEmbeddingClient>,
    model: String,
    cache: EmbeddingCache,
    semaphore: Arc<Semaphore>,
    batch_size: usize,
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
            cache: EmbeddingCache::new(10000), // Cache up to 10k embeddings
            semaphore: Arc::new(Semaphore::new(10)), // Limit concurrent requests
            batch_size: 100, // Process in batches of 100
        })
    }

    /// Generate hash key for caching
    fn generate_cache_key(&self, text: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}:{}", self.model, text));
        format!("{:x}", hasher.finalize())
    }

    /// Generates embeddings for a batch of texts using the selected LLM client with optimizations.
    pub async fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Check cache first
        let mut embeddings = Vec::with_capacity(texts.len());
        let mut uncached_indices = Vec::new();
        let mut uncached_texts = Vec::new();

        for (i, text) in texts.iter().enumerate() {
            if text.is_empty() {
                return Err(anyhow!("Text cannot be empty"));
            }

            let cache_key = self.generate_cache_key(text);
            if let Some(cached_embedding) = self.cache.get(&cache_key) {
                embeddings.push((i, cached_embedding));
            } else {
                uncached_indices.push(i);
                uncached_texts.push(text.clone());
            }
        }

        // Process uncached texts in batches
        let mut uncached_embeddings = Vec::new();
        for chunk in uncached_texts.chunks(self.batch_size) {
            let chunk_embeddings = self.process_batch(chunk).await?;
            uncached_embeddings.extend(chunk_embeddings);
        }

        // Cache new embeddings
        for (text, embedding) in uncached_texts.iter().zip(uncached_embeddings.iter()) {
            let cache_key = self.generate_cache_key(text);
            self.cache.insert(cache_key, embedding.clone());
        }

        // Combine cached and new embeddings in correct order
        let mut result = vec![Vec::new(); texts.len()];
        
        // Place cached embeddings
        for (index, embedding) in embeddings {
            result[index] = embedding;
        }
        
        // Place new embeddings
        for (i, embedding) in uncached_embeddings.into_iter().enumerate() {
            if let Some(&original_index) = uncached_indices.get(i) {
                result[original_index] = embedding;
            }
        }

        Ok(result)
    }

    /// Process a batch of texts with rate limiting
    async fn process_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let _permit = self.semaphore.acquire().await.unwrap();
        
        let mut embeddings = Vec::new();
        for text in texts {
            let request = LlmEmbeddingRequest {
                model: &self.model,
                text: std::borrow::Cow::Borrowed(text),
                task_type: None,
                output_dimension: None,
            };

            let response = self.client.embed_text(request).await?;
            embeddings.push(response.embedding);
        }

        // Normalize embeddings for better similarity computations
        let mut embeddings = embeddings;
        VectorOps::batch_normalize(&mut embeddings);

        Ok(embeddings)
    }

    /// Generates embeddings with similarity search optimization
    pub async fn generate_embeddings_with_similarity(&self, texts: &[String], query_embedding: Option<&[f32]>) -> Result<Vec<(Vec<f32>, f32)>> {
        let embeddings = self.generate_embeddings(texts).await?;
        
        if let Some(query) = query_embedding {
            let similarities: Vec<f32> = embeddings
                .par_iter()
                .map(|emb| VectorOps::cosine_similarity(emb, query))
                .collect();
            
            Ok(embeddings.into_iter().zip(similarities).collect())
        } else {
            Ok(embeddings.into_iter().map(|emb| (emb, 0.0)).collect())
        }
    }

    /// Returns the default embedding dimension for the current model.
    pub fn get_dimension(&self) -> Option<u32> {
        self.client.get_default_embedding_dimension(&self.model)
    }

    /// Get cache statistics for monitoring
    pub fn get_cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.cache.read().unwrap();
        (cache.len(), self.cache.max_size)
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.cache.write().unwrap();
        let mut access = self.cache.access_count.write().unwrap();
        cache.clear();
        access.clear();
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
        let texts = vec!["test text".to_string()];
        let embeddings = service
            .generate_embeddings(&texts)
            .await
            .unwrap();

        assert_eq!(embeddings.len(), 1);
        assert_eq!(embeddings[0].len(), 1536);
    }

}
