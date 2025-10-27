use crate::{
    Plugin, PluginConfig, PluginFactory, PluginMetadata, PluginStatus, PluginType, PluginResult,
    agents::{AgentPlugin, AgentPluginFactory},
    sources::{SourcePlugin, SourcePluginFactory, SyncResult, Document},
    error::PluginError,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::RwLock as AsyncRwLock;

/// Plugin registry for managing all plugins
pub struct PluginRegistry {
    source_factories: HashMap<String, Box<dyn SourcePluginFactory>>,
    agent_factories: HashMap<String, Box<dyn AgentPluginFactory>>,
    active_sources: Arc<AsyncRwLock<HashMap<String, Box<dyn SourcePlugin>>>>,
    active_agents: Arc<AsyncRwLock<HashMap<String, Box<dyn AgentPlugin>>>>,
    plugin_configs: Arc<RwLock<HashMap<String, PluginConfig>>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            source_factories: HashMap::new(),
            agent_factories: HashMap::new(),
            active_sources: Arc::new(AsyncRwLock::new(HashMap::new())),
            active_agents: Arc::new(AsyncRwLock::new(HashMap::new())),
            plugin_configs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a source plugin factory
    pub fn register_source_factory(&mut self, factory: Box<dyn SourcePluginFactory>) {
        let source_type = factory.source_type().to_string();
        self.source_factories.insert(source_type, factory);
    }

    /// Register an agent plugin factory
    pub fn register_agent_factory(&mut self, factory: Box<dyn AgentPluginFactory>) {
        let agent_type = factory.agent_type().to_string();
        self.agent_factories.insert(agent_type, factory);
    }

    /// Load and activate a source plugin
    pub async fn load_source(&self, source_type: &str, instance_id: &str, config: PluginConfig) -> Result<(), PluginError> {
        let factory = self.source_factories.get(source_type)
            .ok_or_else(|| PluginError::NotFound(format!("Source type '{}' not found", source_type)))?;

        let mut plugin = factory.create();
        plugin.initialize(config.clone()).await?;
        plugin.start().await?;

        // Store config
        {
            let mut configs = self.plugin_configs.write().unwrap();
            configs.insert(instance_id.to_string(), config);
        }

        // Store active plugin
        {
            let mut active_sources = self.active_sources.write().await;
            active_sources.insert(instance_id.to_string(), plugin);
        }

        Ok(())
    }

    /// Load and activate an agent plugin
    pub async fn load_agent(&self, agent_type: &str, instance_id: &str, config: PluginConfig) -> Result<(), PluginError> {
        let factory = self.agent_factories.get(agent_type)
            .ok_or_else(|| PluginError::NotFound(format!("Agent type '{}' not found", agent_type)))?;

        let mut plugin = factory.create();
        plugin.initialize(config.clone()).await?;
        plugin.start().await?;

        // Store config
        {
            let mut configs = self.plugin_configs.write().unwrap();
            configs.insert(instance_id.to_string(), config);
        }

        // Store active plugin
        {
            let mut active_agents = self.active_agents.write().await;
            active_agents.insert(instance_id.to_string(), plugin);
        }

        Ok(())
    }

    /// Unload a source plugin
    pub async fn unload_source(&self, instance_id: &str) -> Result<(), PluginError> {
        let mut active_sources = self.active_sources.write().await;
        if let Some(mut plugin) = active_sources.remove(instance_id) {
            plugin.stop().await?;
        }

        // Remove config
        {
            let mut configs = self.plugin_configs.write().unwrap();
            configs.remove(instance_id);
        }

        Ok(())
    }

    /// Unload an agent plugin
    pub async fn unload_agent(&self, instance_id: &str) -> Result<(), PluginError> {
        let mut active_agents = self.active_agents.write().await;
        if let Some(mut plugin) = active_agents.remove(instance_id) {
            plugin.stop().await?;
        }

        // Remove config
        {
            let mut configs = self.plugin_configs.write().unwrap();
            configs.remove(instance_id);
        }

        Ok(())
    }

    /// Get a source plugin instance
    pub async fn get_source(&self, instance_id: &str) -> Option<Box<dyn SourcePlugin>> {
        let active_sources = self.active_sources.read().await;
        // Note: This is a simplified version. In practice, you'd want to return a reference or use Arc
        None // Placeholder - would need proper implementation with Arc<dyn SourcePlugin>
    }

    /// Get an agent plugin instance
    pub async fn get_agent(&self, instance_id: &str) -> Option<Box<dyn AgentPlugin>> {
        let active_agents = self.active_agents.read().await;
        // Note: This is a simplified version. In practice, you'd want to return a reference or use Arc
        None // Placeholder - would need proper implementation with Arc<dyn AgentPlugin>
    }

    /// List all available source types
    pub fn list_source_types(&self) -> Vec<String> {
        self.source_factories.keys().cloned().collect()
    }

    /// List all available agent types
    pub fn list_agent_types(&self) -> Vec<String> {
        self.agent_factories.keys().cloned().collect()
    }

    /// List all active source instances
    pub async fn list_active_sources(&self) -> Vec<String> {
        let active_sources = self.active_sources.read().await;
        active_sources.keys().cloned().collect()
    }

    /// List all active agent instances
    pub async fn list_active_agents(&self) -> Vec<String> {
        let active_agents = self.active_agents.read().await;
        active_agents.keys().cloned().collect()
    }

    /// Get plugin status
    pub async fn get_plugin_status(&self, instance_id: &str) -> Option<PluginStatus> {
        // Check sources first
        {
            let active_sources = self.active_sources.read().await;
            if let Some(plugin) = active_sources.get(instance_id) {
                return Some(plugin.status());
            }
        }

        // Check agents
        {
            let active_agents = self.active_agents.read().await;
            if let Some(plugin) = active_agents.get(instance_id) {
                return Some(plugin.status());
            }
        }

        None
    }

    /// Health check all plugins
    pub async fn health_check_all(&self) -> HashMap<String, bool> {
        let mut results = HashMap::new();

        // Check sources
        {
            let active_sources = self.active_sources.read().await;
            for (id, plugin) in active_sources.iter() {
                let healthy = plugin.health_check().await.unwrap_or(false);
                results.insert(id.clone(), healthy);
            }
        }

        // Check agents
        {
            let active_agents = self.active_agents.read().await;
            for (id, plugin) in active_agents.iter() {
                let healthy = plugin.health_check().await.unwrap_or(false);
                results.insert(id.clone(), healthy);
            }
        }

        results
    }

    /// Health check a specific source plugin
    pub async fn health_check_source(&self, instance_id: &str) -> PluginResult<bool> {
        let active_sources = self.active_sources.read().await;
        if let Some(plugin) = active_sources.get(instance_id) {
            plugin.health_check().await
        } else {
            Err(PluginError::NotFound(format!("Source plugin '{}' not found", instance_id)))
        }
    }

    /// Health check a specific agent plugin
    pub async fn health_check_agent(&self, instance_id: &str) -> PluginResult<bool> {
        let active_agents = self.active_agents.read().await;
        if let Some(plugin) = active_agents.get(instance_id) {
            plugin.health_check().await
        } else {
            Err(PluginError::NotFound(format!("Agent plugin '{}' not found", instance_id)))
        }
    }

    /// Sync documents from a source plugin
    pub async fn sync_source_documents(&self, instance_id: &str) -> Result<SyncResult, PluginError> {
        let active_sources = self.active_sources.read().await;
        if let Some(plugin) = active_sources.get(instance_id) {
            plugin.sync().await.map_err(|e| PluginError::RuntimeError(e.to_string()))
        } else {
            Err(PluginError::NotFound(format!("Source plugin '{}' not found", instance_id)))
        }
    }

    /// Search documents in a source plugin
    pub async fn search_source_documents(&self, instance_id: &str, query: &str, limit: Option<usize>) -> Result<Vec<Document>, PluginError> {
        let active_sources = self.active_sources.read().await;
        if let Some(plugin) = active_sources.get(instance_id) {
            let mut documents = plugin.search_documents(query).await.map_err(|e| PluginError::RuntimeError(e.to_string()))?;
            if let Some(limit) = limit {
                documents.truncate(limit);
            }
            Ok(documents)
        } else {
            Err(PluginError::NotFound(format!("Source plugin '{}' not found", instance_id)))
        }
    }
}