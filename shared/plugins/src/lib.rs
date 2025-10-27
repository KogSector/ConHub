pub mod registry;
pub mod sources;
pub mod agents;
pub mod config;
pub mod error;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub plugin_type: PluginType,
    pub capabilities: Vec<String>,
    pub config_schema: Option<serde_json::Value>,
}

/// Plugin types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginType {
    Source,
    Agent,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub settings: HashMap<String, serde_json::Value>,
}

/// Plugin status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginStatus {
    Inactive,
    Loading,
    Active,
    Error(String),
}

/// Base plugin trait that all plugins must implement
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;
    
    /// Initialize the plugin with configuration
    async fn initialize(&mut self, config: PluginConfig) -> Result<(), error::PluginError>;
    
    /// Start the plugin
    async fn start(&mut self) -> Result<(), error::PluginError>;
    
    /// Stop the plugin
    async fn stop(&mut self) -> Result<(), error::PluginError>;
    
    /// Get current plugin status
    fn status(&self) -> PluginStatus;
    
    /// Health check
    async fn health_check(&self) -> Result<bool, error::PluginError>;
    
    /// Validate configuration
    fn validate_config(&self, config: &PluginConfig) -> Result<(), error::PluginError>;
}

/// Plugin factory trait for creating plugin instances
pub trait PluginFactory: Send + Sync {
    fn create(&self) -> Box<dyn Plugin>;
    fn metadata(&self) -> PluginMetadata;
}

/// Plugin result type
pub type PluginResult<T> = Result<T, error::PluginError>;