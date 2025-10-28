use anyhow::{anyhow, Result};
use conhub_plugins::{
    registry::PluginRegistry, 
    config::PluginConfigManager, 
    sources::Document, 
    agents::{AgentMessage, AgentResponse, ConversationContext},
    PluginConfig, PluginType
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginManagerConfig {
    pub auto_start_plugins: bool,
    pub health_check_interval: u64, // seconds
    pub max_concurrent_operations: usize,
    pub default_timeout: u64, // seconds
}

impl Default for PluginManagerConfig {
    fn default() -> Self {
        Self {
            auto_start_plugins: true,
            health_check_interval: 300, // 5 minutes
            max_concurrent_operations: 10,
            default_timeout: 30,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginOperationResult {
    pub success: bool,
    pub plugin_id: String,
    pub operation: String,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkOperationResult {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub results: Vec<PluginOperationResult>,
}

pub struct PluginManager {
    registry: Arc<RwLock<PluginRegistry>>,
    config_manager: Arc<RwLock<PluginConfigManager>>,
    config: PluginManagerConfig,
    operation_stats: Arc<RwLock<HashMap<String, OperationStats>>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct OperationStats {
    total_operations: u64,
    successful_operations: u64,
    failed_operations: u64,
    average_duration_ms: f64,
}

impl PluginManager {
    pub fn new(
        registry: Arc<RwLock<PluginRegistry>>,
        config_manager: Arc<RwLock<PluginConfigManager>>,
        config: Option<PluginManagerConfig>,
    ) -> Self {
        Self {
            registry,
            config_manager,
            config: config.unwrap_or_default(),
            operation_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize all configured plugins
    pub async fn initialize_all_plugins(&self) -> Result<BulkOperationResult> {
        info!("Initializing all configured plugins");
        
        let config_manager = self.config_manager.read().await;
        let configs: Vec<_> = config_manager.plugins.values().cloned().collect();
        
        let mut results = Vec::new();
        let mut successful = 0;
        let mut failed = 0;
        
        for config in configs {
            let start_time = std::time::Instant::now();
            let result = {
                let registry = self.registry.read().await;
                let plugin_config = PluginConfig {
                    enabled: config.enabled,
                    settings: config.config.settings.clone(),
                };
                match config.plugin_type {
                    PluginType::Source => {
                        registry.load_source(&config.plugin_name, &config.instance_id, plugin_config).await
                    },
                    PluginType::Agent => {
                        registry.load_agent(&config.plugin_name, &config.instance_id, plugin_config).await
                    },
                }
            };
            
            let duration = start_time.elapsed().as_millis() as u64;
            
            let operation_result = match result {
                Ok(_) => {
                    successful += 1;
                    PluginOperationResult {
                        success: true,
                        plugin_id: config.instance_id.clone(),
                        operation: "initialize".to_string(),
                        result: None,
                        error: None,
                        duration_ms: duration,
                    }
                }
                Err(ref e) => {
                    failed += 1;
                    PluginOperationResult {
                        success: false,
                        plugin_id: config.instance_id.clone(),
                        operation: "initialize".to_string(),
                        result: None,
                        error: Some(e.to_string()),
                        duration_ms: duration,
                    }
                }
            };
            
            results.push(operation_result);
            self.update_operation_stats(&config.instance_id, "initialize", duration, result.is_ok()).await;
        }
        
        info!("Plugin initialization completed: {} successful, {} failed", successful, failed);
        
        Ok(BulkOperationResult {
            total: results.len(),
            successful,
            failed,
            results,
        })
    }

    /// Start all enabled plugins
    pub async fn start_enabled_plugins(&self) -> Result<BulkOperationResult> {
        info!("Starting all enabled plugins");
        
        let config_manager = self.config_manager.read().await;
        let configs: Vec<_> = config_manager.plugins.values().cloned().collect();
        
        let mut results = Vec::new();
        let mut successful = 0;
        let mut failed = 0;
        
        for config in configs {
            if !config.enabled || !config.auto_start {
                continue;
            }
            
            let start_time = std::time::Instant::now();
            let result = {
                let registry = self.registry.read().await;
                match config.plugin_type {
                    conhub_plugins::PluginType::Source => {
                        registry.load_source(&config.plugin_name, &config.instance_id, config.config.clone()).await
                    }
                    conhub_plugins::PluginType::Agent => {
                        registry.load_agent(&config.plugin_name, &config.instance_id, config.config.clone()).await
                    }
                }
            };
            
            let duration = start_time.elapsed().as_millis() as u64;
            
            let operation_result = match result {
                Ok(_) => {
                    successful += 1;
                    PluginOperationResult {
                        success: true,
                        plugin_id: config.instance_id.clone(),
                        operation: "start".to_string(),
                        result: None,
                        error: None,
                        duration_ms: duration,
                    }
                }
                Err(ref e) => {
                    failed += 1;
                    PluginOperationResult {
                        success: false,
                        plugin_id: config.instance_id.clone(),
                        operation: "start".to_string(),
                        result: None,
                        error: Some(e.to_string()),
                        duration_ms: duration,
                    }
                }
            };
            
            results.push(operation_result);
            self.update_operation_stats(&config.instance_id, "start", duration, result.is_ok()).await;
        }
        
        info!("Plugin startup completed: {} successful, {} failed", successful, failed);
        
        Ok(BulkOperationResult {
            total: results.len(),
            successful,
            failed,
            results,
        })
    }

    /// Perform health checks on all running plugins
    pub async fn health_check_all_plugins(&self) -> Result<BulkOperationResult> {
        info!("Performing health checks on all plugins");
        
        let registry = self.registry.read().await;
        let mut active_plugins = registry.list_active_sources().await;
        active_plugins.extend(registry.list_active_agents().await);
        
        let mut results = Vec::new();
        let mut successful = 0;
        let mut failed = 0;
        
        for plugin_id in active_plugins {
            let start_time = std::time::Instant::now();
            
            // Check if it's a source plugin first, then agent plugin
            let result = match registry.health_check_source(&plugin_id).await {
                Ok(healthy) => Ok(healthy),
                Err(_) => {
                    // Try as agent plugin
                    registry.health_check_agent(&plugin_id).await.map_err(|e| anyhow!("Health check failed: {}", e))
                }
            };
            let duration = start_time.elapsed().as_millis() as u64;
            
            let operation_result = match result {
                Ok(_) => {
                    successful += 1;
                    PluginOperationResult {
                        success: true,
                        plugin_id: plugin_id.clone(),
                        operation: "health_check".to_string(),
                        result: None,
                        error: None,
                        duration_ms: duration,
                    }
                }
                Err(ref e) => {
                    failed += 1;
                    warn!("Health check failed for plugin {}: {}", plugin_id, e);
                    PluginOperationResult {
                        success: false,
                        plugin_id: plugin_id.clone(),
                        operation: "health_check".to_string(),
                        result: None,
                        error: Some(e.to_string()),
                        duration_ms: duration,
                    }
                }
            };
            
            results.push(operation_result);
            self.update_operation_stats(&plugin_id, "health_check", duration, result.is_ok()).await;
        }
        
        info!("Health check completed: {} successful, {} failed", successful, failed);
        
        Ok(BulkOperationResult {
            total: results.len(),
            successful,
            failed,
            results,
        })
    }

    /// Sync documents from all source plugins
    pub async fn sync_all_sources(&self) -> Result<BulkOperationResult> {
        info!("Syncing documents from all source plugins");
        
        let registry = self.registry.read().await;
        let source_plugins = registry.list_active_sources().await;
        
        let mut results = Vec::new();
        let mut successful = 0;
        let mut failed = 0;
        
        for plugin_id in source_plugins {
            let start_time = std::time::Instant::now();
            let result = registry.sync_source_documents(&plugin_id).await;
            let duration = start_time.elapsed().as_millis() as u64;
            
            let operation_result = match result {
                Ok(ref sync_result) => {
                    successful += 1;
                    PluginOperationResult {
                        success: true,
                        plugin_id: plugin_id.clone(),
                        operation: "sync".to_string(),
                        result: Some(serde_json::to_value(sync_result)?),
                        error: None,
                        duration_ms: duration,
                    }
                }
                Err(ref e) => {
                    failed += 1;
                    error!("Sync failed for source plugin {}: {}", plugin_id, e);
                    PluginOperationResult {
                        success: false,
                        plugin_id: plugin_id.clone(),
                        operation: "sync".to_string(),
                        result: None,
                        error: Some(e.to_string()),
                        duration_ms: duration,
                    }
                }
            };
            
            results.push(operation_result);
            self.update_operation_stats(&plugin_id, "sync", duration, result.is_ok()).await;
        }
        
        info!("Source sync completed: {} successful, {} failed", successful, failed);
        
        Ok(BulkOperationResult {
            total: results.len(),
            successful,
            failed,
            results,
        })
    }

    /// Search across all source plugins
    pub async fn search_all_sources(&self, query: &str, limit: Option<usize>) -> Result<Vec<Document>> {
        info!("Searching across all source plugins for: {}", query);
        
        let registry = self.registry.read().await;
        let source_plugins = registry.list_active_sources().await;
        
        let mut all_documents = Vec::new();
        let per_plugin_limit = limit.map(|l| l / source_plugins.len().max(1));
        
        for plugin_id in source_plugins {
            match registry.search_source_documents(&plugin_id, query, per_plugin_limit).await {
                Ok(mut documents) => {
                    info!("Found {} documents from plugin {}", documents.len(), plugin_id);
                    all_documents.append(&mut documents);
                }
                Err(e) => {
                    warn!("Search failed for source plugin {}: {}", plugin_id, e);
                }
            }
        }
        
        // Sort by relevance/modified date and apply global limit
        all_documents.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
        
        if let Some(limit) = limit {
            all_documents.truncate(limit);
        }
        
        info!("Search completed: found {} total documents", all_documents.len());
        Ok(all_documents)
    }

    /// Process a message with the best available agent plugin
    pub async fn process_with_best_agent(
        &self,
        message: AgentMessage,
        context: ConversationContext,
        preferred_agent: Option<String>,
    ) -> Result<AgentResponse> {
        let registry = self.registry.read().await;
        let agent_plugins = registry.list_active_agents().await;
        
        if agent_plugins.is_empty() {
            return Err(anyhow!("No active agent plugins available"));
        }
        
        // Use preferred agent if specified and available
        let selected_agent = if let Some(preferred) = preferred_agent {
            if agent_plugins.contains(&preferred) {
                preferred
            } else {
                warn!("Preferred agent {} not available, using first available", preferred);
                agent_plugins[0].clone()
            }
        } else {
            // Use the first available agent (could be enhanced with load balancing)
            agent_plugins[0].clone()
        };
        
        info!("Processing message with agent: {}", selected_agent);
        
        let start_time = std::time::Instant::now();
        let result = if let Some(agent) = registry.get_agent(&selected_agent).await {
            agent.process_message(message, context).await.map_err(|e| anyhow!("Agent error: {}", e))
        } else {
            Err(anyhow!("Agent '{}' not found", selected_agent))
        };
        let duration = start_time.elapsed().as_millis() as u64;
        
        self.update_operation_stats(&selected_agent, "process_message", duration, result.is_ok()).await;
        
        result
    }

    /// Get operation statistics for all plugins
    pub async fn get_operation_stats(&self) -> HashMap<String, OperationStats> {
        self.operation_stats.read().await.clone()
    }

    /// Get plugin status summary
    pub async fn get_status_summary(&self) -> Result<Value> {
        let registry = self.registry.read().await;
        let config_manager = self.config_manager.read().await;
        
        let all_configs: Vec<_> = config_manager.plugins.values().collect();
        let source_plugins = registry.list_active_sources().await;
        let agent_plugins = registry.list_active_agents().await;
        let mut active_plugins = source_plugins.clone();
        active_plugins.extend(agent_plugins.clone());
        
        let stats = self.get_operation_stats().await;
        
        Ok(serde_json::json!({
            "total_configured": all_configs.len(),
            "total_active": active_plugins.len(),
            "active_sources": source_plugins.len(),
            "active_agents": agent_plugins.len(),
            "enabled_plugins": all_configs.iter().filter(|c| c.enabled).count(),
            "auto_start_plugins": all_configs.iter().filter(|c| c.auto_start).count(),
            "operation_stats": stats,
            "last_updated": chrono::Utc::now()
        }))
    }

    /// Restart a specific plugin
    pub async fn restart_plugin(&self, plugin_id: &str) -> Result<PluginOperationResult> {
        info!("Restarting plugin: {}", plugin_id);
        
        let start_time = std::time::Instant::now();
        
        // Get plugin configuration to determine type
        let config = {
            let config_manager = self.config_manager.read().await;
            config_manager.get_plugin(plugin_id)
                .ok_or_else(|| anyhow!("Plugin configuration not found: {}", plugin_id))?
                .clone()
        };
        
        // Stop the plugin first
        let registry = self.registry.read().await;
        let stop_result = match config.plugin_type {
            conhub_plugins::PluginType::Source => {
                registry.unload_source(plugin_id).await
            }
            conhub_plugins::PluginType::Agent => {
                registry.unload_agent(plugin_id).await
            }
        };
        
        if let Err(e) = stop_result {
            warn!("Failed to stop plugin {} during restart: {}", plugin_id, e);
        }
        
        // Start the plugin again
        let result = match config.plugin_type {
            conhub_plugins::PluginType::Source => {
                registry.load_source(&config.plugin_name, plugin_id, config.config.clone()).await
            }
            conhub_plugins::PluginType::Agent => {
                registry.load_agent(&config.plugin_name, plugin_id, config.config.clone()).await
            }
        };
        let duration = start_time.elapsed().as_millis() as u64;
        
        let success = result.is_ok();
        let operation_result = match result {
            Ok(_) => {
                info!("Plugin {} restarted successfully", plugin_id);
                PluginOperationResult {
                    success: true,
                    plugin_id: plugin_id.to_string(),
                    operation: "restart".to_string(),
                    result: None,
                    error: None,
                    duration_ms: duration,
                }
            }
            Err(e) => {
                error!("Failed to restart plugin {}: {}", plugin_id, e);
                PluginOperationResult {
                    success: false,
                    plugin_id: plugin_id.to_string(),
                    operation: "restart".to_string(),
                    result: None,
                    error: Some(e.to_string()),
                    duration_ms: duration,
                }
            }
        };
        
        self.update_operation_stats(plugin_id, "restart", duration, success).await;
        Ok(operation_result)
    }

    /// Update operation statistics
    async fn update_operation_stats(&self, plugin_id: &str, operation: &str, duration_ms: u64, success: bool) {
        let mut stats = self.operation_stats.write().await;
        let plugin_stats = stats.entry(plugin_id.to_string()).or_insert_with(Default::default);
        
        plugin_stats.total_operations += 1;
        
        if success {
            plugin_stats.successful_operations += 1;
        } else {
            plugin_stats.failed_operations += 1;
        }
        
        // Update average duration (simple moving average)
        let total_ops = plugin_stats.total_operations as f64;
        plugin_stats.average_duration_ms = 
            (plugin_stats.average_duration_ms * (total_ops - 1.0) + duration_ms as f64) / total_ops;
    }

    /// Start periodic health checks
    pub async fn start_health_check_scheduler(&self) -> Result<()> {
        let interval_duration = self.config.health_check_interval;
        let registry = Arc::clone(&self.registry);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_duration));
            
            loop {
                interval.tick().await;
                
                // Perform health check on all plugins
                let registry_guard = registry.read().await;
                let health_results = registry_guard.health_check_all().await;
                
                // Log any unhealthy plugins
                for (plugin_id, is_healthy) in health_results {
                    if !is_healthy {
                        error!("Plugin {} failed health check", plugin_id);
                    }
                }
            }
        });
        
        info!("Health check scheduler started with interval: {}s", interval_duration);
        Ok(())
    }
}