use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use conhub_plugins::{PluginStatus, PluginType};
use crate::AppState;

#[derive(Deserialize)]
pub struct StartPluginRequest {
    pub instance_id: String,
}

#[derive(Deserialize)]
pub struct StopPluginRequest {
    pub instance_id: String,
}

#[derive(Serialize)]
pub struct PluginStatusResponse {
    pub instance_id: String,
    pub status: PluginStatus,
    pub plugin_type: String,
    pub plugin_name: String,
    pub health_status: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Serialize)]
pub struct LifecycleResponse {
    pub success: bool,
    pub message: String,
    pub instance_id: String,
}

/// Start a plugin instance
pub async fn start_plugin(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let instance_id = path.into_inner();
    
    info!("Starting plugin instance: {}", instance_id);
    
    // Get plugin configuration
    let config = {
        let config_manager = app_state.config_manager.read().await;
        match config_manager.get_plugin(&instance_id) {
            Some(config) => config.clone(),
            None => {
                error!("Plugin configuration not found: {}", instance_id);
                return Ok(HttpResponse::NotFound().json(LifecycleResponse {
                    success: false,
                    message: format!("Plugin configuration not found: {}", instance_id),
                    instance_id,
                }));
            }
        }
    };
    
    // Start the plugin using the registry
    let registry = app_state.registry.read().await;
    
    let result = match config.plugin_type {
        PluginType::Source => {
            registry.load_source(&config.plugin_name, &instance_id, config.config.clone()).await
        }
        PluginType::Agent => {
            registry.load_agent(&config.plugin_name, &instance_id, config.config.clone()).await
        }
    };
    
    match result {
        Ok(_) => {
            info!("Successfully started plugin: {}", instance_id);
            Ok(HttpResponse::Ok().json(LifecycleResponse {
                success: true,
                message: format!("Plugin '{}' started successfully", instance_id),
                instance_id,
            }))
        }
        Err(e) => {
            error!("Failed to start plugin {}: {}", instance_id, e);
            Ok(HttpResponse::InternalServerError().json(LifecycleResponse {
                success: false,
                message: format!("Failed to start plugin: {}", e),
                instance_id,
            }))
        }
    }
}

/// Stop a plugin instance
pub async fn stop_plugin(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let instance_id = path.into_inner();
    
    info!("Stopping plugin instance: {}", instance_id);
    
    // Get plugin configuration to determine type
    let config = {
        let config_manager = app_state.config_manager.read().await;
        match config_manager.get_plugin(&instance_id) {
            Some(config) => config.clone(),
            None => {
                error!("Plugin configuration not found: {}", instance_id);
                return Ok(HttpResponse::NotFound().json(LifecycleResponse {
                    success: false,
                    message: format!("Plugin configuration not found: {}", instance_id),
                    instance_id,
                }));
            }
        }
    };
    
    // Stop the plugin using the registry
    let registry = app_state.registry.read().await;
    
    let result = match config.plugin_type {
        PluginType::Source => {
            registry.unload_source(&instance_id).await
        }
        PluginType::Agent => {
            registry.unload_agent(&instance_id).await
        }
    };
    
    match result {
        Ok(_) => {
            info!("Successfully stopped plugin: {}", instance_id);
            Ok(HttpResponse::Ok().json(LifecycleResponse {
                success: true,
                message: format!("Plugin '{}' stopped successfully", instance_id),
                instance_id,
            }))
        }
        Err(e) => {
            error!("Failed to stop plugin {}: {}", instance_id, e);
            Ok(HttpResponse::InternalServerError().json(LifecycleResponse {
                success: false,
                message: format!("Failed to stop plugin: {}", e),
                instance_id,
            }))
        }
    }
}

/// Restart a plugin instance
pub async fn restart_plugin(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let instance_id = path.into_inner();
    
    info!("Restarting plugin instance: {}", instance_id);
    
    // Get plugin configuration
    let config = {
        let config_manager = app_state.config_manager.read().await;
        match config_manager.get_plugin(&instance_id) {
            Some(config) => config.clone(),
            None => {
                error!("Plugin configuration not found: {}", instance_id);
                return Ok(HttpResponse::NotFound().json(LifecycleResponse {
                    success: false,
                    message: format!("Plugin configuration not found: {}", instance_id),
                    instance_id,
                }));
            }
        }
    };
    
    let registry = app_state.registry.read().await;
    
    // First stop the plugin
    let stop_result = match config.plugin_type {
        PluginType::Source => {
            registry.unload_source(&instance_id).await
        }
        PluginType::Agent => {
            registry.unload_agent(&instance_id).await
        }
    };
    
    if let Err(e) = stop_result {
        error!("Failed to stop plugin {} during restart: {}", instance_id, e);
        return Ok(HttpResponse::InternalServerError().json(LifecycleResponse {
            success: false,
            message: format!("Failed to stop plugin during restart: {}", e),
            instance_id,
        }));
    }
    
    // Then start it again
    let start_result = match config.plugin_type {
        PluginType::Source => {
            registry.load_source(&config.plugin_name, &instance_id, config.config.clone()).await
        }
        PluginType::Agent => {
            registry.load_agent(&config.plugin_name, &instance_id, config.config.clone()).await
        }
    };
    
    match start_result {
        Ok(_) => {
            info!("Successfully restarted plugin: {}", instance_id);
            Ok(HttpResponse::Ok().json(LifecycleResponse {
                success: true,
                message: format!("Plugin '{}' restarted successfully", instance_id),
                instance_id,
            }))
        }
        Err(e) => {
            error!("Failed to start plugin {} during restart: {}", instance_id, e);
            Ok(HttpResponse::InternalServerError().json(LifecycleResponse {
                success: false,
                message: format!("Failed to start plugin during restart: {}", e),
                instance_id,
            }))
        }
    }
}

/// Get status of a specific plugin instance
pub async fn get_plugin_status(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let instance_id = path.into_inner();
    
    // Get plugin configuration
    let config = {
        let config_manager = app_state.config_manager.read().await;
        match config_manager.get_plugin(&instance_id) {
            Some(config) => config.clone(),
            None => {
                return Ok(HttpResponse::NotFound().json(LifecycleResponse {
                    success: false,
                    message: format!("Plugin '{}' not found", instance_id),
                    instance_id,
                }));
            }
        }
    };
    
    let registry = app_state.registry.read().await;
    
    match registry.get_plugin_status(&instance_id).await {
        Some(status) => {
            // Get health status from health check
            let health_results = registry.health_check_all().await;
            let health_status = health_results.get(&instance_id)
                .map(|&healthy| if healthy { "Healthy".to_string() } else { "Unhealthy".to_string() });
            
            Ok(HttpResponse::Ok().json(PluginStatusResponse {
                instance_id,
                status,
                plugin_type: format!("{:?}", config.plugin_type),
                plugin_name: config.plugin_name,
                health_status,
                last_error: None, // TODO: Implement error tracking
            }))
        }
        None => {
            Ok(HttpResponse::NotFound().json(LifecycleResponse {
                success: false,
                message: format!("Plugin '{}' not found or not active", instance_id),
                instance_id,
            }))
        }
    }
}

/// Get status of all plugin instances
pub async fn get_all_plugin_status(
    app_state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let config_manager = app_state.config_manager.read().await;
    let registry = app_state.registry.read().await;
    
    let mut statuses = Vec::new();
    let health_results = registry.health_check_all().await;
    
    // Iterate through all configured plugins
    for (instance_id, config) in &config_manager.plugins {
        let status = registry.get_plugin_status(instance_id).await
            .unwrap_or(PluginStatus::Inactive);
        
        let health_status = health_results.get(instance_id)
            .map(|&healthy| if healthy { "Healthy".to_string() } else { "Unhealthy".to_string() });
        
        statuses.push(PluginStatusResponse {
            instance_id: instance_id.clone(),
            status,
            plugin_type: format!("{:?}", config.plugin_type),
            plugin_name: config.plugin_name.clone(),
            health_status,
            last_error: None, // TODO: Implement error tracking
        });
    }
    
    Ok(HttpResponse::Ok().json(statuses))
}