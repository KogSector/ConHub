use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

use crate::llm::{LlmEmbeddingClient, LlmEmbeddingRequest, LlmEmbeddingResponse};
use crate::models::qwen::QwenEmbeddingClient;
use crate::models::openai::OpenAIEmbeddingClient;
use crate::models::cohere::CohereEmbeddingClient;
use crate::models::voyage::VoyageEmbeddingClient;
use crate::models::jina::JinaEmbeddingClient;
use crate::models::huggingface::HuggingFaceEmbeddingClient;
use crate::config::fusion_config::{FusionConfig, RoutingRule};
use crate::config::profile_config::ProfileConfig;
use crate::services::embedding::EmbeddingCache;

/// Factory for creating embedding clients
pub struct EmbeddingClientFactory;

impl EmbeddingClientFactory {
    pub fn create(client_type: &str) -> Result<Box<dyn LlmEmbeddingClient>> {
        match client_type {
            "qwen" => {
                let api_key = std::env::var("QWEN_API_KEY")
                    .map_err(|_| anyhow!("QWEN_API_KEY environment variable not set"))?;
                Ok(Box::new(QwenEmbeddingClient::new(api_key)))
            }
            "openai" => {
                let api_key = std::env::var("OPENAI_API_KEY")
                    .map_err(|_| anyhow!("OPENAI_API_KEY environment variable not set"))?;
                Ok(Box::new(OpenAIEmbeddingClient::new(api_key)))
            }
            "cohere" => {
                let api_key = std::env::var("COHERE_API_KEY")
                    .map_err(|_| anyhow!("COHERE_API_KEY environment variable not set"))?;
                Ok(Box::new(CohereEmbeddingClient::new(api_key)))
            }
            "voyage" => {
                let api_key = std::env::var("VOYAGE_API_KEY")
                    .map_err(|_| anyhow!("VOYAGE_API_KEY environment variable not set"))?;
                Ok(Box::new(VoyageEmbeddingClient::new(api_key)))
            }
            "jina" => {
                let api_key = std::env::var("JINA_API_KEY")
                    .map_err(|_| anyhow!("JINA_API_KEY environment variable not set"))?;
                Ok(Box::new(JinaEmbeddingClient::new(api_key)))
            }
            "huggingface" => {
                let api_token = std::env::var("HUGGINGFACE_API_TOKEN")
                    .map_err(|_| anyhow!("HUGGINGFACE_API_TOKEN environment variable not set"))?;
                
                // Check for dedicated endpoint URL (optional)
                let dedicated_endpoint = std::env::var("HUGGINGFACE_ENDPOINT_URL").ok();
                let base_url = std::env::var("HUGGINGFACE_API_BASE_URL").ok();
                
                let client = if let Some(endpoint) = dedicated_endpoint {
                    // Use dedicated Inference Endpoint
                    HuggingFaceEmbeddingClient::with_endpoint(api_token, endpoint)
                } else if let Some(base) = base_url {
                    // Use custom base URL for Inference API
                    HuggingFaceEmbeddingClient::with_base_url(api_token, base)
                } else {
                    // Use default HuggingFace Inference API
                    HuggingFaceEmbeddingClient::new(api_token)
                };
                
                Ok(Box::new(client))
            }
            _ => Err(anyhow!("Unknown client type: {}", client_type)),
        }
    }
}

/// Fusion embedding service that orchestrates multiple embedding models
pub struct FusionEmbeddingService {
    config: FusionConfig,
    profile_config: ProfileConfig,
    clients: Arc<RwLock<HashMap<String, Arc<Box<dyn LlmEmbeddingClient>>>>>,
    cache: EmbeddingCache,
}

impl FusionEmbeddingService {
    pub fn new(config_path: &str) -> Result<Self> {
        let config = FusionConfig::from_file(config_path)?;
        
        // Load profile config
        let profile_config_path = config_path.replace("fusion_config.json", "profiles.json");
        let profile_config = ProfileConfig::from_file(&profile_config_path)
            .unwrap_or_else(|e| {
                warn!("Failed to load profile config: {}. Using fusion config only.", e);
                // Create a minimal fallback profile config
                ProfileConfig {
                    profiles: vec![],
                    fallback_profile: crate::config::profile_config::FallbackProfile {
                        id: "default_fallback".to_string(),
                        models: vec!["general_text".to_string()],
                        weights: vec![1.0],
                        fusion_strategy: "weighted_average".to_string(),
                        chunker: "generic_paragraphs".to_string(),
                        description: "Default fallback".to_string(),
                    },
                    content_type_detection: crate::config::profile_config::ContentTypeDetection {
                        code_extensions: vec![],
                        text_extensions: vec![],
                        language_mapping: HashMap::new(),
                    },
                }
            });
        
        // Initialize clients
        let mut clients = HashMap::new();
        for model_config in &config.models {
            match EmbeddingClientFactory::create(&model_config.client) {
                Ok(client) => {
                    info!("✓ Initialized {} client for model {}", model_config.client, model_config.name);
                    clients.insert(model_config.name.clone(), Arc::new(client));
                }
                Err(e) => {
                    warn!("✗ Failed to initialize {} client: {}. Skipping.", model_config.client, e);
                }
            }
        }
        
        if clients.is_empty() {
            return Err(anyhow!("No embedding clients could be initialized"));
        }
        
        info!("✓ Fusion embedding service initialized with {} models", clients.len());
        
        Ok(Self {
            config,
            profile_config,
            clients: Arc::new(RwLock::new(clients)),
            cache: EmbeddingCache::new(10000),
        })
    }
    
    /// Route request to appropriate models based on source type
    pub async fn generate_embeddings(
        &self,
        texts: &[String],
        source_type: &str,
    ) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        
        // Get routing rule for source type
        let routing = self.config.get_models_for_source(source_type);
        
        if routing.is_none() {
            warn!("No routing rule found for source type: {}, using fallback", source_type);
            return self.generate_with_fallback(texts).await;
        }
        
        let routing = routing.unwrap();
        
        // Generate embeddings with multiple models
        match routing.fusion_strategy.as_str() {
            "weighted_average" => {
                self.fuse_with_weighted_average(texts, routing).await
            }
            "concatenate" => {
                self.fuse_with_concatenation(texts, routing).await
            }
            "max_pooling" => {
                self.fuse_with_max_pooling(texts, routing).await
            }
            _ => {
                warn!("Unknown fusion strategy: {}, using weighted_average", routing.fusion_strategy);
                self.fuse_with_weighted_average(texts, routing).await
            }
        }
    }
    
    /// Generate embeddings with fine-grained profile routing
    pub async fn generate_embeddings_with_profile(
        &self,
        texts: &[String],
        connector_type: &str,
        block_type: Option<&str>,
        language: Option<&str>,
        content_type: Option<&str>,
    ) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        
        // Find matching profile
        let profile = self.profile_config.find_profile(
            connector_type,
            block_type,
            language,
            content_type,
        );
        
        info!(
            "🎯 Using profile '{}' for connector={}, block_type={:?}, language={:?}",
            profile.id, connector_type, block_type, language
        );
        
        // Create routing rule from profile
        let routing = RoutingRule {
            source: connector_type.to_string(),
            models: profile.models.clone(),
            weights: profile.weights.clone(),
            fusion_strategy: profile.fusion_strategy.clone(),
        };
        
        // Generate embeddings with the profile's strategy
        match routing.fusion_strategy.as_str() {
            "weighted_average" => {
                self.fuse_with_weighted_average(texts, &routing).await
            }
            "concatenate" => {
                self.fuse_with_concatenation(texts, &routing).await
            }
            "max_pooling" => {
                self.fuse_with_max_pooling(texts, &routing).await
            }
            _ => {
                warn!("Unknown fusion strategy: {}, using weighted_average", routing.fusion_strategy);
                self.fuse_with_weighted_average(texts, &routing).await
            }
        }
    }
    
    /// Fuse embeddings using weighted average
    async fn fuse_with_weighted_average(
        &self,
        texts: &[String],
        routing: &RoutingRule,
    ) -> Result<Vec<Vec<f32>>> {
        let mut all_embeddings: Vec<Vec<Vec<f32>>> = Vec::new();
        let clients = self.clients.read().await;
        
        // Generate embeddings with each model
        for (model_name, &weight) in routing.models.iter().zip(routing.weights.iter()) {
            if weight <= 0.0 {
                continue;
            }
            
            let client = clients.get(model_name);
            if client.is_none() {
                warn!("Client {} not available, skipping", model_name);
                continue;
            }
            
            let client = client.unwrap();
            let model_config = self.config.get_model_config(model_name)
                .ok_or_else(|| anyhow!("Model config not found for {}", model_name))?;
            
            info!("Generating embeddings with model: {}", model_name);
            
            let mut model_embeddings = Vec::new();
            for text in texts {
                let request = LlmEmbeddingRequest {
                    model: &model_config.model,
                    text: std::borrow::Cow::Borrowed(text),
                    task_type: None,
                    output_dimension: Some(model_config.dimension),
                };
                
                match client.embed_text(request).await {
                    Ok(response) => {
                        model_embeddings.push(response.embedding);
                    }
                    Err(e) => {
                        error!("Failed to generate embedding with {}: {}", model_name, e);
                        // Continue with other models
                    }
                }
            }
            
            if !model_embeddings.is_empty() {
                all_embeddings.push(model_embeddings);
            }
        }
        
        if all_embeddings.is_empty() {
            return Err(anyhow!("All models failed to generate embeddings"));
        }
        
        // Fuse embeddings using weighted average
        let num_texts = texts.len();
        let mut fused_embeddings = Vec::new();
        
        for text_idx in 0..num_texts {
            // Get dimension from first available embedding
            let first_embedding = &all_embeddings[0][text_idx];
            let dimension = first_embedding.len();
            
            let mut fused = vec![0.0; dimension];
            let mut total_weight = 0.0;
            
            for (model_idx, model_embeddings) in all_embeddings.iter().enumerate() {
                if let Some(embedding) = model_embeddings.get(text_idx) {
                    let weight = routing.weights.get(model_idx).copied().unwrap_or(1.0);
                    
                    // Normalize embedding dimension if needed
                    let normalized = if embedding.len() == dimension {
                        embedding.clone()
                    } else {
                        // Simple interpolation for different dimensions
                        self.normalize_dimension(embedding, dimension)
                    };
                    
                    for (i, &val) in normalized.iter().enumerate() {
                        fused[i] += val * weight;
                    }
                    total_weight += weight;
                }
            }
            
            // Normalize by total weight
            if total_weight > 0.0 {
                for val in &mut fused {
                    *val /= total_weight;
                }
            }
            
            fused_embeddings.push(fused);
        }
        
        Ok(fused_embeddings)
    }
    
    /// Fuse embeddings using concatenation
    async fn fuse_with_concatenation(
        &self,
        texts: &[String],
        routing: &RoutingRule,
    ) -> Result<Vec<Vec<f32>>> {
        let mut all_embeddings: Vec<Vec<Vec<f32>>> = Vec::new();
        let clients = self.clients.read().await;
        
        for model_name in &routing.models {
            let client = clients.get(model_name);
            if client.is_none() {
                continue;
            }
            
            let client = client.unwrap();
            let model_config = self.config.get_model_config(model_name)
                .ok_or_else(|| anyhow!("Model config not found for {}", model_name))?;
            
            let mut model_embeddings = Vec::new();
            for text in texts {
                let request = LlmEmbeddingRequest {
                    model: &model_config.model,
                    text: std::borrow::Cow::Borrowed(text),
                    task_type: None,
                    output_dimension: Some(model_config.dimension),
                };
                
                if let Ok(response) = client.embed_text(request).await {
                    model_embeddings.push(response.embedding);
                }
            }
            
            if !model_embeddings.is_empty() {
                all_embeddings.push(model_embeddings);
            }
        }
        
        if all_embeddings.is_empty() {
            return Err(anyhow!("All models failed to generate embeddings"));
        }
        
        // Concatenate embeddings
        let num_texts = texts.len();
        let mut fused_embeddings = Vec::new();
        
        for text_idx in 0..num_texts {
            let mut concatenated = Vec::new();
            
            for model_embeddings in &all_embeddings {
                if let Some(embedding) = model_embeddings.get(text_idx) {
                    concatenated.extend_from_slice(embedding);
                }
            }
            
            fused_embeddings.push(concatenated);
        }
        
        Ok(fused_embeddings)
    }
    
    /// Fuse embeddings using max pooling
    async fn fuse_with_max_pooling(
        &self,
        texts: &[String],
        routing: &RoutingRule,
    ) -> Result<Vec<Vec<f32>>> {
        let mut all_embeddings: Vec<Vec<Vec<f32>>> = Vec::new();
        let clients = self.clients.read().await;
        
        for model_name in &routing.models {
            let client = clients.get(model_name);
            if client.is_none() {
                continue;
            }
            
            let client = client.unwrap();
            let model_config = self.config.get_model_config(model_name)
                .ok_or_else(|| anyhow!("Model config not found for {}", model_name))?;
            
            let mut model_embeddings = Vec::new();
            for text in texts {
                let request = LlmEmbeddingRequest {
                    model: &model_config.model,
                    text: std::borrow::Cow::Borrowed(text),
                    task_type: None,
                    output_dimension: Some(model_config.dimension),
                };
                
                if let Ok(response) = client.embed_text(request).await {
                    model_embeddings.push(response.embedding);
                }
            }
            
            if !model_embeddings.is_empty() {
                all_embeddings.push(model_embeddings);
            }
        }
        
        if all_embeddings.is_empty() {
            return Err(anyhow!("All models failed to generate embeddings"));
        }
        
        // Max pooling
        let num_texts = texts.len();
        let mut fused_embeddings = Vec::new();
        
        for text_idx in 0..num_texts {
            let first_embedding = &all_embeddings[0][text_idx];
            let dimension = first_embedding.len();
            
            let mut max_pooled = vec![f32::MIN; dimension];
            
            for model_embeddings in &all_embeddings {
                if let Some(embedding) = model_embeddings.get(text_idx) {
                    let normalized = if embedding.len() == dimension {
                        embedding.clone()
                    } else {
                        self.normalize_dimension(embedding, dimension)
                    };
                    
                    for (i, &val) in normalized.iter().enumerate() {
                        max_pooled[i] = max_pooled[i].max(val);
                    }
                }
            }
            
            fused_embeddings.push(max_pooled);
        }
        
        Ok(fused_embeddings)
    }
    
    /// Generate embeddings with fallback model
    async fn generate_with_fallback(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let clients = self.clients.read().await;
        let fallback_client = clients.get(&self.config.fallback_model)
            .ok_or_else(|| anyhow!("Fallback model {} not available", self.config.fallback_model))?;
        
        let model_config = self.config.get_model_config(&self.config.fallback_model)
            .ok_or_else(|| anyhow!("Fallback model config not found"))?;
        
        let mut embeddings = Vec::new();
        for text in texts {
            let request = LlmEmbeddingRequest {
                model: &model_config.model,
                text: std::borrow::Cow::Borrowed(text),
                task_type: None,
                output_dimension: Some(model_config.dimension),
            };
            
            let response = fallback_client.embed_text(request).await?;
            embeddings.push(response.embedding);
        }
        
        Ok(embeddings)
    }
    
    /// Normalize embedding dimension using interpolation
    fn normalize_dimension(&self, embedding: &[f32], target_dim: usize) -> Vec<f32> {
        let source_dim = embedding.len();
        if source_dim == target_dim {
            return embedding.to_vec();
        }
        
        let mut normalized = vec![0.0; target_dim];
        let ratio = source_dim as f32 / target_dim as f32;
        
        for i in 0..target_dim {
            let source_idx = (i as f32 * ratio) as usize;
            normalized[i] = embedding.get(source_idx).copied().unwrap_or(0.0);
        }
        
        normalized
    }
    
    /// Get available models
    pub fn get_available_models(&self) -> Vec<String> {
        self.config.models.iter().map(|m| m.name.clone()).collect()
    }
    
    /// Get fusion config
    pub fn get_config(&self) -> &FusionConfig {
        &self.config
    }
}
