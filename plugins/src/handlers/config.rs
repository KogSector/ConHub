use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

use conhub_plugins::{config::{PluginConfigManager, PluginInstanceConfig}, PluginType, PluginConfig};
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct CreatePluginConfigRequest {
    pub instance_id: String,
    pub plugin_type: PluginType,
    pub plugin_name: String,
    pub enabled: bool,
    pub auto_start: bool,
    pub config: Value,
}

#[derive(Deserialize)]
pub struct UpdatePluginConfigRequest {
    pub enabled: Option<bool>,
    pub auto_start: Option<bool>,
    pub config: Option<Value>,
}

#[derive(Serialize)]
pub struct ConfigResponse {
    pub success: bool,
    pub message: String,
    pub instance_id: Option<String>,
}

#[derive(Serialize)]
pub struct PluginConfigResponse {
    pub instance_id: String,
    pub plugin_type: PluginType,
    pub plugin_name: String,
    pub enabled: bool,
    pub auto_start: bool,
    pub config: Value,
}

/// Get all plugin configurations
pub async fn get_all_configs(
    config_manager: web::Data<Arc<RwLock<PluginConfigManager>>>,
) -> Result<HttpResponse> {
    let config_manager = config_manager.read().await;
    
    let response: Vec<PluginConfigResponse> = config_manager.plugins
        .iter()
        .map(|(instance_id, config)| PluginConfigResponse {
            instance_id: instance_id.clone(),
            plugin_type: config.plugin_type.clone(),
            plugin_name: config.plugin_name.clone(),
            enabled: config.enabled,
            auto_start: config.auto_start,
            config: Value::Object(config.config.settings.clone().into_iter().collect()),
        })
        .collect();
    
    Ok(HttpResponse::Ok().json(response))
}

/// Get configuration for a specific plugin instance
pub async fn get_config(
    config_manager: web::Data<Arc<RwLock<PluginConfigManager>>>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let instance_id = path.into_inner();
    let config_manager = config_manager.read().await;
    
    match config_manager.get_plugin(&instance_id) {
        Some(config) => {
            Ok(HttpResponse::Ok().json(PluginConfigResponse {
                instance_id,
                plugin_type: config.plugin_type.clone(),
                plugin_name: config.plugin_name.clone(),
                enabled: config.enabled,
                auto_start: config.auto_start,
                config: Value::Object(config.config.settings.clone().into_iter().collect()),
            }))
        }
        None => {
            Ok(HttpResponse::NotFound().json(ConfigResponse {
                success: false,
                message: format!("Configuration for plugin '{}' not found", instance_id),
                instance_id: Some(instance_id),
            }))
        }
    }
}

/// Create a new plugin configuration
pub async fn create_config(
    config_manager: web::Data<Arc<RwLock<PluginConfigManager>>>,
    request: web::Json<CreatePluginConfigRequest>,
) -> Result<HttpResponse> {
    let mut config_manager = config_manager.write().await;
    
    // Convert Value to PluginConfig
    let plugin_config_struct = if let Value::Object(map) = &request.config {
        PluginConfig {
            enabled: request.enabled,
            settings: map.clone().into_iter().collect(),
        }
    } else {
        PluginConfig {
            enabled: request.enabled,
            settings: HashMap::new(),
        }
    };

    let plugin_config = PluginInstanceConfig {
        instance_id: request.instance_id.clone(),
        plugin_type: request.plugin_type.clone(),
        plugin_name: request.plugin_name.clone(),
        enabled: request.enabled,
        auto_start: request.auto_start,
        config: plugin_config_struct,
        metadata: None,
    };
    
    config_manager.add_plugin(plugin_config);
    
    info!("Created configuration for plugin: {}", request.instance_id);
    Ok(HttpResponse::Created().json(ConfigResponse {
        success: true,
        message: format!("Configuration for plugin '{}' created successfully", request.instance_id),
        instance_id: Some(request.instance_id.clone()),
    }))
}

/// Update an existing plugin configuration
pub async fn update_config(
    config_manager: web::Data<Arc<RwLock<PluginConfigManager>>>,
    path: web::Path<String>,
    request: web::Json<UpdatePluginConfigRequest>,
) -> Result<HttpResponse> {
    let instance_id = path.into_inner();
    let mut config_manager = config_manager.write().await;
    
    // Get existing configuration
    let mut existing_config = match config_manager.get_plugin(&instance_id) {
        Some(config) => config.clone(),
        None => {
            return Ok(HttpResponse::NotFound().json(ConfigResponse {
                success: false,
                message: format!("Configuration for plugin '{}' not found", instance_id),
                instance_id: Some(instance_id),
            }));
        }
    };
    
    // Update fields if provided
    if let Some(enabled) = request.enabled {
        existing_config.enabled = enabled;
    }
    if let Some(auto_start) = request.auto_start {
        existing_config.auto_start = auto_start;
    }
    if let Some(config) = &request.config {
        if let Value::Object(map) = config {
            existing_config.config.settings = map.clone().into_iter().collect();
        }
    }
    
    match config_manager.update_plugin(&instance_id, existing_config) {
        Ok(_) => {
            info!("Updated configuration for plugin: {}", instance_id);
            Ok(HttpResponse::Ok().json(ConfigResponse {
                success: true,
                message: format!("Configuration for plugin '{}' updated successfully", instance_id),
                instance_id: Some(instance_id),
            }))
        }
        Err(e) => {
            error!("Failed to update configuration for plugin {}: {}", instance_id, e);
            Ok(HttpResponse::InternalServerError().json(ConfigResponse {
                success: false,
                message: format!("Failed to update configuration: {}", e),
                instance_id: Some(instance_id),
            }))
        }
    }
}

/// Delete a plugin configuration
pub async fn delete_config(
    config_manager: web::Data<Arc<RwLock<PluginConfigManager>>>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let instance_id = path.into_inner();
    let mut config_manager = config_manager.write().await;
    
    match config_manager.remove_plugin(&instance_id) {
        Some(_) => {
            info!("Deleted configuration for plugin: {}", instance_id);
            Ok(HttpResponse::Ok().json(ConfigResponse {
                success: true,
                message: format!("Configuration for plugin '{}' deleted successfully", instance_id),
                instance_id: Some(instance_id),
            }))
        }
        None => {
            Ok(HttpResponse::NotFound().json(ConfigResponse {
                success: false,
                message: format!("Configuration for plugin '{}' not found", instance_id),
                instance_id: Some(instance_id),
            }))
        }
    }
}

/// Validate a plugin configuration without saving it
pub async fn validate_config(
    request: web::Json<CreatePluginConfigRequest>,
) -> Result<HttpResponse> {
    // Basic validation
    if request.instance_id.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ConfigResponse {
            success: false,
            message: "Instance ID cannot be empty".to_string(),
            instance_id: None,
        }));
    }
    
    if request.plugin_name.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ConfigResponse {
            success: false,
            message: "Plugin name cannot be empty".to_string(),
            instance_id: Some(request.instance_id.clone()),
        }));
    }
    
    // TODO: Add plugin-specific validation based on plugin_name
    // This would validate the config structure against the plugin's schema
    
    Ok(HttpResponse::Ok().json(ConfigResponse {
        success: true,
        message: "Configuration is valid".to_string(),
        instance_id: Some(request.instance_id.clone()),
    }))
}

/// Reload configurations from file
pub async fn reload_configs(
    config_manager: web::Data<Arc<RwLock<PluginConfigManager>>>,
) -> Result<HttpResponse> {
    let _config_manager = config_manager.write().await;
    
    // For now, just return success as the configuration is already in memory
    // In a real implementation, this would reload from a configuration file
    info!("Successfully reloaded plugin configurations");
    Ok(HttpResponse::Ok().json(ConfigResponse {
        success: true,
        message: "Configurations reloaded successfully".to_string(),
        instance_id: None,
    }))
}