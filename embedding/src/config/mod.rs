pub mod model_config;
pub mod fusion_config;

pub use model_config::{
    ModelConfig, ModelType, ArchitectureConfig, ModelSettings,
    PoolingStrategy, ModelPrecision, ModelRegistry
};
pub use fusion_config::{FusionConfig, ModelConfig as FusionModelConfig, RoutingRule};