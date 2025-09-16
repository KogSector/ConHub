use actix_web::{web, HttpResponse, Result};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

use crate::models::{
    AgentRecord, CreateAgentRequest, UpdateAgentRequest, AgentInvokeRequest, 
    AgentStatus, AgentUsageStats, ApiResponse,
    AgentContext, RepositoryContext, DocumentContext, UrlContext
};
use crate::services::ai_agents::AgentService;

// In-memory storage for demonstration (replace with actual database)
static mut AGENTS: Option<HashMap<String, AgentRecord>> = None;

fn get_agents_store() -> &'static mut HashMap<String, AgentRecord> {
    unsafe {
        if AGENTS.is_none() {
            AGENTS = Some(HashMap::new());
        }
        AGENTS.as_mut().unwrap()
    }
}

// Get all agents for a user
pub async fn get_agents() -> Result<HttpResponse> {
    let agents = get_agents_store();
    let agent_list: Vec<&AgentRecord> = agents.values().collect();
    
    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "Agents retrieved successfully".to_string(),
        data: Some(agent_list),
        error: None,
    }))
}

// Get specific agent by ID
pub async fn get_agent(path: web::Path<String>) -> Result<HttpResponse> {
    let agent_id = path.into_inner();
    let agents = get_agents_store();
    
    match agents.get(&agent_id) {
        Some(agent) => Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: "Agent retrieved successfully".to_string(),
            data: Some(agent),
            error: None,
        })),
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            message: "Agent not found".to_string(),
            data: None,
            error: Some("Agent with the specified ID does not exist".to_string()),
        })),
    }
}

// Create a new agent
pub async fn create_agent(req: web::Json<CreateAgentRequest>) -> Result<HttpResponse> {
    let agents = get_agents_store();
    let agent_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    
    // Validate agent type
    let valid_types = vec!["openai", "anthropic", "claude", "custom"];
    if !valid_types.contains(&req.agent_type.as_str()) {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            message: "Invalid agent type".to_string(),
            data: None,
            error: Some("Agent type must be one of: openai, anthropic, claude, custom".to_string()),
        }));
    }
    
    // Validate permissions
    let valid_permissions = vec!["read", "write", "context", "repositories", "documents", "urls"];
    for permission in &req.permissions {
        if !valid_permissions.contains(&permission.as_str()) {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                message: "Invalid permission".to_string(),
                data: None,
                error: Some(format!("Invalid permission: {}", permission)),
            }));
        }
    }
    
    let agent = AgentRecord {
        id: agent_id.clone(),
        user_id: "default_user".to_string(), // TODO: Get from auth context
        name: req.name.clone(),
        agent_type: req.agent_type.clone(),
        endpoint: req.endpoint.clone(),
        api_key: req.api_key.clone(), // TODO: Encrypt this
        permissions: req.permissions.clone(),
        status: AgentStatus::Pending,
        config: req.config.clone(),
        created_at: now.clone(),
        updated_at: now.clone(),
        last_used: None,
        usage_stats: AgentUsageStats {
            total_requests: 0,
            total_tokens: 0,
            avg_response_time: None,
            last_error: None,
        },
    };
    
    agents.insert(agent_id.clone(), agent.clone());
    
    Ok(HttpResponse::Created().json(ApiResponse {
        success: true,
        message: "Agent created successfully".to_string(),
        data: Some(agent),
        error: None,
    }))
}

// Update an existing agent
pub async fn update_agent(
    path: web::Path<String>,
    req: web::Json<UpdateAgentRequest>,
) -> Result<HttpResponse> {
    let agent_id = path.into_inner();
    let agents = get_agents_store();
    
    match agents.get_mut(&agent_id) {
        Some(agent) => {
            let now = Utc::now().to_rfc3339();
            
            if let Some(name) = &req.name {
                agent.name = name.clone();
            }
            if let Some(endpoint) = &req.endpoint {
                agent.endpoint = Some(endpoint.clone());
            }
            if let Some(api_key) = &req.api_key {
                agent.api_key = api_key.clone(); // TODO: Encrypt this
            }
            if let Some(permissions) = &req.permissions {
                agent.permissions = permissions.clone();
            }
            if let Some(config) = &req.config {
                agent.config = config.clone();
            }
            if let Some(status) = &req.status {
                agent.status = status.clone();
            }
            agent.updated_at = now;
            
            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: "Agent updated successfully".to_string(),
                data: Some(agent.clone()),
                error: None,
            }))
        }
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            message: "Agent not found".to_string(),
            data: None,
            error: Some("Agent with the specified ID does not exist".to_string()),
        })),
    }
}

// Delete an agent
pub async fn delete_agent(path: web::Path<String>) -> Result<HttpResponse> {
    let agent_id = path.into_inner();
    let agents = get_agents_store();
    
    match agents.remove(&agent_id) {
        Some(_) => Ok(HttpResponse::Ok().json(ApiResponse::<()> {
            success: true,
            message: "Agent deleted successfully".to_string(),
            data: None,
            error: None,
        })),
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            message: "Agent not found".to_string(),
            data: None,
            error: Some("Agent with the specified ID does not exist".to_string()),
        })),
    }
}

// Get context for an agent (filtered by permissions)
pub async fn get_agent_context(path: web::Path<String>) -> Result<HttpResponse> {
    let agent_id = path.into_inner();
    let agents = get_agents_store();
    
    match agents.get(&agent_id) {
        Some(agent) => {
            let mut context = AgentContext {
                repositories: vec![],
                documents: vec![],
                urls: vec![],
            };
            
            // Filter context based on agent permissions
            if agent.permissions.contains(&"repositories".to_string()) {
                context.repositories = vec![
                    RepositoryContext {
                        id: "repo1".to_string(),
                        name: "ConHub".to_string(),
                        description: Some("Universal context hub for developers".to_string()),
                        language: "TypeScript/Rust".to_string(),
                        recent_files: vec!["src/main.rs".to_string(), "frontend/app/page.tsx".to_string()],
                        recent_commits: vec!["feat: add agent integration".to_string()],
                    }
                ];
            }
            
            if agent.permissions.contains(&"documents".to_string()) {
                context.documents = vec![
                    DocumentContext {
                        id: "doc1".to_string(),
                        name: "API Documentation".to_string(),
                        doc_type: "markdown".to_string(),
                        summary: Some("ConHub API reference".to_string()),
                        tags: vec!["api".to_string(), "docs".to_string()],
                    }
                ];
            }
            
            if agent.permissions.contains(&"urls".to_string()) {
                context.urls = vec![
                    UrlContext {
                        id: "url1".to_string(),
                        url: "https://github.com/openai/openai-api".to_string(),
                        title: Some("OpenAI API Documentation".to_string()),
                        summary: Some("Official OpenAI API reference".to_string()),
                        tags: vec!["openai".to_string(), "api".to_string()],
                    }
                ];
            }
            
            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: "Agent context retrieved successfully".to_string(),
                data: Some(context),
                error: None,
            }))
        }
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            message: "Agent not found".to_string(),
            data: None,
            error: Some("Agent with the specified ID does not exist".to_string()),
        })),
    }
}

// Invoke an agent (send message and get response)
pub async fn invoke_agent(
    path: web::Path<String>,
    req: web::Json<AgentInvokeRequest>,
) -> Result<HttpResponse> {
    let agent_id = path.into_inner();
    let agents = get_agents_store();
    
    match agents.get_mut(&agent_id) {
        Some(agent) => {
            let agent_service = AgentService::new();
            
            // Get context based on agent permissions and request
            let context = get_filtered_context(agent, &req.context_type);
            
            match agent_service.invoke_agent(agent, &req, context.as_ref()).await {
                Ok(response) => {
                    // Update usage stats
                    agent.usage_stats.total_requests += 1;
                    agent.usage_stats.total_tokens += response.usage.tokens_used as u64;
                    agent.last_used = Some(Utc::now().to_rfc3339());
                    
                    // Update average response time
                    let current_avg = agent.usage_stats.avg_response_time.unwrap_or(0.0);
                    let new_avg = if agent.usage_stats.total_requests == 1 {
                        response.usage.response_time_ms as f32
                    } else {
                        (current_avg * (agent.usage_stats.total_requests - 1) as f32 + response.usage.response_time_ms as f32) 
                        / agent.usage_stats.total_requests as f32
                    };
                    agent.usage_stats.avg_response_time = Some(new_avg);
                    
                    Ok(HttpResponse::Ok().json(ApiResponse {
                        success: true,
                        message: "Agent invoked successfully".to_string(),
                        data: Some(response),
                        error: None,
                    }))
                }
                Err(e) => {
                    agent.usage_stats.last_error = Some(e.to_string());
                    Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                        success: false,
                        message: "Failed to invoke agent".to_string(),
                        data: None,
                        error: Some(e.to_string()),
                    }))
                }
            }
        }
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            message: "Agent not found".to_string(),
            data: None,
            error: Some("Agent with the specified ID does not exist".to_string()),
        })),
    }
}

fn get_filtered_context(agent: &AgentRecord, context_type: &Option<String>) -> Option<AgentContext> {
    let mut context = AgentContext {
        repositories: vec![],
        documents: vec![],
        urls: vec![],
    };
    
    let include_all = context_type.is_none() || context_type.as_ref().unwrap() == "all";
    
    // Filter context based on agent permissions and requested context type
    if (include_all || context_type.as_ref().unwrap() == "repositories") 
       && agent.permissions.contains(&"repositories".to_string()) {
        context.repositories = vec![
            RepositoryContext {
                id: "repo1".to_string(),
                name: "ConHub".to_string(),
                description: Some("Universal context hub for developers".to_string()),
                language: "TypeScript/Rust".to_string(),
                recent_files: vec!["src/main.rs".to_string(), "frontend/app/page.tsx".to_string()],
                recent_commits: vec!["feat: add agent integration".to_string()],
            }
        ];
    }
    
    if (include_all || context_type.as_ref().unwrap() == "documents") 
       && agent.permissions.contains(&"documents".to_string()) {
        context.documents = vec![
            DocumentContext {
                id: "doc1".to_string(),
                name: "API Documentation".to_string(),
                doc_type: "markdown".to_string(),
                summary: Some("ConHub API reference".to_string()),
                tags: vec!["api".to_string(), "docs".to_string()],
            }
        ];
    }
    
    if (include_all || context_type.as_ref().unwrap() == "urls") 
       && agent.permissions.contains(&"urls".to_string()) {
        context.urls = vec![
            UrlContext {
                id: "url1".to_string(),
                url: "https://github.com/openai/openai-api".to_string(),
                title: Some("OpenAI API Documentation".to_string()),
                summary: Some("Official OpenAI API reference".to_string()),
                tags: vec!["openai".to_string(), "api".to_string()],
            }
        ];
    }
    
    // Return context only if it has any data
    if !context.repositories.is_empty() || !context.documents.is_empty() || !context.urls.is_empty() {
        Some(context)
    } else {
        None
    }
}

// Test agent connection
pub async fn test_agent(path: web::Path<String>) -> Result<HttpResponse> {
    let agent_id = path.into_inner();
    let agents = get_agents_store();
    
    match agents.get_mut(&agent_id) {
        Some(agent) => {
            let agent_service = AgentService::new();
            
            match agent_service.test_agent_connection(agent).await {
                Ok(connected) => {
                    if connected {
                        agent.status = AgentStatus::Connected;
                        Ok(HttpResponse::Ok().json(ApiResponse {
                            success: true,
                            message: "Agent connection test successful".to_string(),
                            data: Some(json!({"connected": true})),
                            error: None,
                        }))
                    } else {
                        agent.status = AgentStatus::Error;
                        agent.usage_stats.last_error = Some("Connection test failed".to_string());
                        Ok(HttpResponse::BadRequest().json(ApiResponse {
                            success: false,
                            message: "Agent connection test failed".to_string(),
                            data: Some(json!({"connected": false})),
                            error: Some("Could not establish connection to agent".to_string()),
                        }))
                    }
                }
                Err(e) => {
                    agent.status = AgentStatus::Error;
                    agent.usage_stats.last_error = Some(e.to_string());
                    Ok(HttpResponse::InternalServerError().json(ApiResponse {
                        success: false,
                        message: "Agent connection test failed".to_string(),
                        data: Some(json!({"connected": false})),
                        error: Some(e.to_string()),
                    }))
                }
            }
        }
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            message: "Agent not found".to_string(),
            data: None,
            error: Some("Agent with the specified ID does not exist".to_string()),
        })),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/agents")
            .route("", web::get().to(get_agents))
            .route("", web::post().to(create_agent))
            .route("/{id}", web::get().to(get_agent))
            .route("/{id}", web::put().to(update_agent))
            .route("/{id}", web::delete().to(delete_agent))
            .route("/{id}/context", web::get().to(get_agent_context))
            .route("/{id}/invoke", web::post().to(invoke_agent))
            .route("/{id}/test", web::post().to(test_agent))
    );
}