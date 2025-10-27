use crate::{PluginConfig, PluginMetadata, PluginType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plugin configuration manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfigManager {
    pub plugins: HashMap<String, PluginInstanceConfig>,
}

/// Individual plugin instance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInstanceConfig {
    pub instance_id: String,
    pub plugin_type: PluginType,
    pub plugin_name: String, // e.g., "dropbox", "google-drive", "cline"
    pub enabled: bool,
    pub auto_start: bool,
    pub config: PluginConfig,
    pub metadata: Option<PluginMetadata>,
}

impl PluginConfigManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Load configuration from file
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Add a plugin configuration
    pub fn add_plugin(&mut self, config: PluginInstanceConfig) {
        self.plugins.insert(config.instance_id.clone(), config);
    }

    /// Remove a plugin configuration
    pub fn remove_plugin(&mut self, instance_id: &str) -> Option<PluginInstanceConfig> {
        self.plugins.remove(instance_id)
    }

    /// Get plugin configuration
    pub fn get_plugin(&self, instance_id: &str) -> Option<&PluginInstanceConfig> {
        self.plugins.get(instance_id)
    }

    /// Update plugin configuration
    pub fn update_plugin(&mut self, instance_id: &str, config: PluginInstanceConfig) -> Result<(), String> {
        if self.plugins.contains_key(instance_id) {
            self.plugins.insert(instance_id.to_string(), config);
            Ok(())
        } else {
            Err(format!("Plugin instance '{}' not found", instance_id))
        }
    }

    /// List all enabled plugins
    pub fn list_enabled(&self) -> Vec<&PluginInstanceConfig> {
        self.plugins.values().filter(|p| p.enabled).collect()
    }

    /// List plugins by type
    pub fn list_by_type(&self, plugin_type: PluginType) -> Vec<&PluginInstanceConfig> {
        self.plugins.values().filter(|p| p.plugin_type == plugin_type).collect()
    }

    /// Get auto-start plugins
    pub fn get_auto_start(&self) -> Vec<&PluginInstanceConfig> {
        self.plugins.values().filter(|p| p.enabled && p.auto_start).collect()
    }
}

impl Default for PluginConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Default plugin configurations for common sources and agents
impl PluginConfigManager {
    /// Create default configuration with common plugins
    pub fn create_default() -> Self {
        let mut manager = Self::new();

        // Add default source configurations
        manager.add_plugin(PluginInstanceConfig {
            instance_id: "dropbox-main".to_string(),
            plugin_type: PluginType::Source,
            plugin_name: "dropbox".to_string(),
            enabled: false,
            auto_start: false,
            config: PluginConfig {
                enabled: false,
                settings: HashMap::from([
                    ("access_token".to_string(), serde_json::Value::String("".to_string())),
                    ("sync_interval_minutes".to_string(), serde_json::Value::Number(serde_json::Number::from(30))),
                ]),
            },
            metadata: None,
        });

        manager.add_plugin(PluginInstanceConfig {
            instance_id: "google-drive-main".to_string(),
            plugin_type: PluginType::Source,
            plugin_name: "google-drive".to_string(),
            enabled: false,
            auto_start: false,
            config: PluginConfig {
                enabled: false,
                settings: HashMap::from([
                    ("client_id".to_string(), serde_json::Value::String("".to_string())),
                    ("client_secret".to_string(), serde_json::Value::String("".to_string())),
                    ("refresh_token".to_string(), serde_json::Value::String("".to_string())),
                ]),
            },
            metadata: None,
        });

        // Add default agent configurations
        manager.add_plugin(PluginInstanceConfig {
            instance_id: "cline-main".to_string(),
            plugin_type: PluginType::Agent,
            plugin_name: "cline".to_string(),
            enabled: false,
            auto_start: false,
            config: PluginConfig {
                enabled: false,
                settings: HashMap::from([
                    ("api_key".to_string(), serde_json::Value::String("".to_string())),
                    ("model".to_string(), serde_json::Value::String("claude-3-sonnet".to_string())),
                ]),
            },
            metadata: None,
        });

        manager.add_plugin(PluginInstanceConfig {
            instance_id: "amazon-q-main".to_string(),
            plugin_type: PluginType::Agent,
            plugin_name: "amazon-q".to_string(),
            enabled: false,
            auto_start: false,
            config: PluginConfig {
                enabled: false,
                settings: HashMap::from([
                    ("region".to_string(), serde_json::Value::String("us-east-1".to_string())),
                    ("access_key_id".to_string(), serde_json::Value::String("".to_string())),
                    ("secret_access_key".to_string(), serde_json::Value::String("".to_string())),
                ]),
            },
            metadata: None,
        });

        manager
    }
}