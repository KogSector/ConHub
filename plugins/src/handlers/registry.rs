use actix_web::{web, Result as ActixResult};
use crate::AppState;

/// List all available source plugin types
pub async fn list_source_types(data: web::Data<AppState>) -> ActixResult<web::Json<serde_json::Value>> {
    let registry = data.registry.read().await;
    let source_types = registry.list_source_types();
    
    Ok(web::Json(serde_json::json!({
        "source_types": source_types
    })))
}

/// List all available agent plugin types
pub async fn list_agent_types(data: web::Data<AppState>) -> ActixResult<web::Json<serde_json::Value>> {
    let registry = data.registry.read().await;
    let agent_types = registry.list_agent_types();
    
    Ok(web::Json(serde_json::json!({
        "agent_types": agent_types
    })))
}

/// List all active source instances
pub async fn list_active_sources(data: web::Data<AppState>) -> ActixResult<web::Json<serde_json::Value>> {
    let registry = data.registry.read().await;
    let active_sources = registry.list_active_sources().await;
    
    Ok(web::Json(serde_json::json!({
        "active_sources": active_sources
    })))
}

/// List all active agent instances
pub async fn list_active_agents(data: web::Data<AppState>) -> ActixResult<web::Json<serde_json::Value>> {
    let registry = data.registry.read().await;
    let active_agents = registry.list_active_agents().await;
    
    Ok(web::Json(serde_json::json!({
        "active_agents": active_agents
    })))
}