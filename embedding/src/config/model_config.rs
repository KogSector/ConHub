use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Model configuration for production deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model name identifier
    pub name: String,
    /// Model type (text, image, audio, etc.)
    pub model_type: ModelType,
    /// Path to model weights file
    pub weights_path: PathBuf,
    /// Path to tokenizer files
    pub tokenizer_path: Option<PathBuf>,
    /// Model architecture parameters
    pub architecture: ArchitectureConfig,
    /// Model-specific settings
    pub settings: ModelSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    Text,
    Image,
    Audio,
    Video,
    Code,
    Multimodal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureConfig {
    /// Hidden dimension size
    pub hidden_size: usize,
    /// Number of transformer layers
    pub num_layers: usize,
    /// Number of attention heads
    pub num_heads: usize,
    /// Vocabulary size
    pub vocab_size: usize,
    /// Maximum sequence length
    pub max_seq_length: usize,
    /// Intermediate size for feed-forward layers
    pub intermediate_size: usize,
    /// Dropout probability
    pub dropout: f32,
    /// Layer normalization epsilon
    pub layer_norm_eps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSettings {
    /// Whether to normalize embeddings
    pub normalize_embeddings: bool,
    /// Pooling strategy
    pub pooling_strategy: PoolingStrategy,
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Model precision (fp16, fp32, int8)
    pub precision: ModelPrecision,
    /// Device placement (cpu, cuda, mps)
    pub device: String,
    /// Additional model-specific parameters
    pub extra_params: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoolingStrategy {
    Mean,
    Max,
    Cls,
    LastToken,
    WeightedMean,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelPrecision {
    FP32,
    FP16,
    INT8,
    INT4,
}

/// Production model configurations
impl ModelConfig {
    /// BGE-base-en-v1.5 configuration
    pub fn bge_base_en_v1_5() -> Self {
        Self {
            name: "bge-base-en-v1.5".to_string(),
            model_type: ModelType::Text,
            weights_path: PathBuf::from("models/bge-base-en-v1.5/pytorch_model.bin"),
            tokenizer_path: Some(PathBuf::from("models/bge-base-en-v1.5/tokenizer.json")),
            architecture: ArchitectureConfig {
                hidden_size: 768,
                num_layers: 12,
                num_heads: 12,
                vocab_size: 30522,
                max_seq_length: 512,
                intermediate_size: 3072,
                dropout: 0.1,
                layer_norm_eps: 1e-12,
            },
            settings: ModelSettings {
                normalize_embeddings: true,
                pooling_strategy: PoolingStrategy::Cls,
                max_batch_size: 32,
                precision: ModelPrecision::FP32,
                device: "cpu".to_string(),
                extra_params: HashMap::new(),
            },
        }
    }

    /// BGE-large-en-v1.5 configuration
    pub fn bge_large_en_v1_5() -> Self {
        Self {
            name: "bge-large-en-v1.5".to_string(),
            model_type: ModelType::Text,
            weights_path: PathBuf::from("models/bge-large-en-v1.5/pytorch_model.bin"),
            tokenizer_path: Some(PathBuf::from("models/bge-large-en-v1.5/tokenizer.json")),
            architecture: ArchitectureConfig {
                hidden_size: 1024,
                num_layers: 24,
                num_heads: 16,
                vocab_size: 30522,
                max_seq_length: 512,
                intermediate_size: 4096,
                dropout: 0.1,
                layer_norm_eps: 1e-12,
            },
            settings: ModelSettings {
                normalize_embeddings: true,
                pooling_strategy: PoolingStrategy::Cls,
                max_batch_size: 16,
                precision: ModelPrecision::FP32,
                device: "cpu".to_string(),
                extra_params: HashMap::new(),
            },
        }
    }

    /// E5-base-v2 configuration
    pub fn e5_base_v2() -> Self {
        Self {
            name: "e5-base-v2".to_string(),
            model_type: ModelType::Text,
            weights_path: PathBuf::from("models/e5-base-v2/pytorch_model.bin"),
            tokenizer_path: Some(PathBuf::from("models/e5-base-v2/tokenizer.json")),
            architecture: ArchitectureConfig {
                hidden_size: 768,
                num_layers: 12,
                num_heads: 12,
                vocab_size: 30522,
                max_seq_length: 512,
                intermediate_size: 3072,
                dropout: 0.1,
                layer_norm_eps: 1e-12,
            },
            settings: ModelSettings {
                normalize_embeddings: true,
                pooling_strategy: PoolingStrategy::Mean,
                max_batch_size: 32,
                precision: ModelPrecision::FP32,
                device: "cpu".to_string(),
                extra_params: HashMap::new(),
            },
        }
    }

    /// Qwen2-7B-Instruct configuration for code embeddings
    pub fn qwen2_7b_instruct() -> Self {
        Self {
            name: "qwen2-7b-instruct".to_string(),
            model_type: ModelType::Code,
            weights_path: PathBuf::from("models/qwen2-7b-instruct/pytorch_model.bin"),
            tokenizer_path: Some(PathBuf::from("models/qwen2-7b-instruct/tokenizer.json")),
            architecture: ArchitectureConfig {
                hidden_size: 4096,
                num_layers: 32,
                num_heads: 32,
                vocab_size: 152064,
                max_seq_length: 2048,
                intermediate_size: 11008,
                dropout: 0.0,
                layer_norm_eps: 1e-6,
            },
            settings: ModelSettings {
                normalize_embeddings: true,
                pooling_strategy: PoolingStrategy::LastToken,
                max_batch_size: 8,
                precision: ModelPrecision::FP16,
                device: "cuda".to_string(),
                extra_params: HashMap::new(),
            },
        }
    }

    /// CLIP-ViT-B/32 configuration for multimodal embeddings
    pub fn clip_vit_b32() -> Self {
        Self {
            name: "clip-vit-b32".to_string(),
            model_type: ModelType::Multimodal,
            weights_path: PathBuf::from("models/clip-vit-b32/pytorch_model.bin"),
            tokenizer_path: Some(PathBuf::from("models/clip-vit-b32/tokenizer.json")),
            architecture: ArchitectureConfig {
                hidden_size: 512,
                num_layers: 12,
                num_heads: 8,
                vocab_size: 49408,
                max_seq_length: 77,
                intermediate_size: 2048,
                dropout: 0.0,
                layer_norm_eps: 1e-5,
            },
            settings: ModelSettings {
                normalize_embeddings: true,
                pooling_strategy: PoolingStrategy::Cls,
                max_batch_size: 16,
                precision: ModelPrecision::FP32,
                device: "cuda".to_string(),
                extra_params: {
                    let mut params = HashMap::new();
                    params.insert("image_size".to_string(), serde_json::Value::Number(224.into()));
                    params.insert("patch_size".to_string(), serde_json::Value::Number(32.into()));
                    params
                },
            },
        }
    }

    /// Load model configuration from file
    pub fn from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: ModelConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save model configuration to file
    pub fn to_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validate model configuration
    pub fn validate(&self) -> Result<(), String> {
        // Check if weights file exists
        if !self.weights_path.exists() {
            return Err(format!("Model weights file not found: {:?}", self.weights_path));
        }

        // Check if tokenizer file exists (if specified)
        if let Some(tokenizer_path) = &self.tokenizer_path {
            if !tokenizer_path.exists() {
                return Err(format!("Tokenizer file not found: {:?}", tokenizer_path));
            }
        }

        // Validate architecture parameters
        if self.architecture.hidden_size == 0 {
            return Err("Hidden size must be greater than 0".to_string());
        }

        if self.architecture.num_layers == 0 {
            return Err("Number of layers must be greater than 0".to_string());
        }

        if self.architecture.num_heads == 0 {
            return Err("Number of heads must be greater than 0".to_string());
        }

        if self.architecture.hidden_size % self.architecture.num_heads != 0 {
            return Err("Hidden size must be divisible by number of heads".to_string());
        }

        // Validate settings
        if self.settings.max_batch_size == 0 {
            return Err("Max batch size must be greater than 0".to_string());
        }

        Ok(())
    }

    /// Get embedding dimension
    pub fn embedding_dim(&self) -> usize {
        self.architecture.hidden_size
    }

    /// Check if model supports the given modality
    pub fn supports_modality(&self, modality: &ModelType) -> bool {
        match (&self.model_type, modality) {
            (ModelType::Multimodal, _) => true,
            (a, b) => a == b,
        }
    }
}

/// Model registry for managing multiple models
#[derive(Debug, Clone)]
pub struct ModelRegistry {
    models: HashMap<String, ModelConfig>,
    default_model: Option<String>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            default_model: None,
        }
    }

    /// Register a model configuration
    pub fn register(&mut self, config: ModelConfig) {
        let name = config.name.clone();
        self.models.insert(name.clone(), config);
        
        // Set as default if it's the first model
        if self.default_model.is_none() {
            self.default_model = Some(name);
        }
    }

    /// Get model configuration by name
    pub fn get(&self, name: &str) -> Option<&ModelConfig> {
        self.models.get(name)
    }

    /// Get default model configuration
    pub fn get_default(&self) -> Option<&ModelConfig> {
        self.default_model.as_ref().and_then(|name| self.models.get(name))
    }

    /// Set default model
    pub fn set_default(&mut self, name: &str) -> Result<(), String> {
        if self.models.contains_key(name) {
            self.default_model = Some(name.to_string());
            Ok(())
        } else {
            Err(format!("Model '{}' not found in registry", name))
        }
    }

    /// List all registered models
    pub fn list_models(&self) -> Vec<&str> {
        self.models.keys().map(|s| s.as_str()).collect()
    }

    /// Load models from directory
    pub fn load_from_directory(&mut self, dir: &PathBuf) -> Result<usize, Box<dyn std::error::Error>> {
        let mut loaded = 0;
        
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    match ModelConfig::from_file(&path) {
                        Ok(config) => {
                            self.register(config);
                            loaded += 1;
                        }
                        Err(e) => {
                            eprintln!("Failed to load model config from {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
        
        Ok(loaded)
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        
        // Register default models
        registry.register(ModelConfig::bge_base_en_v1_5());
        registry.register(ModelConfig::e5_base_v2());
        registry.register(ModelConfig::qwen2_7b_instruct());
        registry.register(ModelConfig::clip_vit_b32());
        
        registry.set_default("bge-base-en-v1.5").unwrap();
        
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_config_creation() {
        let config = ModelConfig::bge_base_en_v1_5();
        assert_eq!(config.name, "bge-base-en-v1.5");
        assert_eq!(config.embedding_dim(), 768);
        assert!(config.supports_modality(&ModelType::Text));
        assert!(!config.supports_modality(&ModelType::Image));
    }

    #[test]
    fn test_model_registry() {
        let mut registry = ModelRegistry::new();
        let config = ModelConfig::bge_base_en_v1_5();
        
        registry.register(config);
        
        assert!(registry.get("bge-base-en-v1.5").is_some());
        assert!(registry.get("nonexistent").is_none());
        assert_eq!(registry.list_models().len(), 1);
    }

    #[test]
    fn test_multimodal_support() {
        let clip_config = ModelConfig::clip_vit_b32();
        assert!(clip_config.supports_modality(&ModelType::Text));
        assert!(clip_config.supports_modality(&ModelType::Image));
        assert!(clip_config.supports_modality(&ModelType::Multimodal));
    }
}