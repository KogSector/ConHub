use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionConfig {
    pub models: Vec<ModelConfig>,
    pub routing: Vec<RoutingRule>,
    pub fallback_model: String,
    pub cache_embeddings: bool,
    pub normalize_embeddings: bool,
    pub batch_size: usize,
    pub max_retries: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub client: String,
    pub model: String,
    pub dimension: u32,
    pub strengths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    pub source: String,
    pub models: Vec<String>,
    pub weights: Vec<f32>,
    pub fusion_strategy: String,
}

impl FusionConfig {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: FusionConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn get_models_for_source(&self, source: &str) -> Option<&RoutingRule> {
        self.routing.iter().find(|r| r.source == source)
    }

    pub fn get_model_config(&self, name: &str) -> Option<&ModelConfig> {
        self.models.iter().find(|m| m.name == name)
    }
}
