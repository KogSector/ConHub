use actix_web::{web, HttpRequest, HttpResponse, Result};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{info, error};
use std::sync::Arc;

use crate::server::MCPServer;
use crate::context::ContextManager;
use crate::protocol::{Agent, SyncRequest, ToolExecutionRequest};
use conhub_middleware::auth::Claims;

// Resource handlers

pub async fn list_resources(server: web::Data<MCPServer>) -> Result<HttpResponse> {
    let resources = server.list_resources();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "resources": resources,
    })))
}

pub async fn read_resource(
    server: web::Data<MCPServer>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let uri = path.into_inner();
    
    match server.get_resource(&uri) {
        Some(resource) => {
            // TODO: Actually fetch and return resource content
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "resource": resource,
                "content": "Resource content would be here",
            })))
        }
        None => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "error": format!("Resource not found: {}", uri),
            })))
        }
    }
}

// Tool handlers

pub async fn list_tools(server: web::Data<MCPServer>) -> Result<HttpResponse> {
    let tools = server.list_tools();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "tools": tools,
    })))
}

#[derive(Debug, Deserialize)]
pub struct ExecuteToolRequest {
    pub tool_name: String,
    pub arguments: serde_json::Value,
}

pub async fn execute_tool(
    server: web::Data<MCPServer>,
    body: web::Json<ExecuteToolRequest>,
) -> Result<HttpResponse> {
    match server.get_tool(&body.tool_name) {
        Some(tool) => {
            info!("🔧 Executing tool: {}", body.tool_name);
            
            // TODO: Implement actual tool execution logic
            let result = serde_json::json!({
                "tool": tool.name,
                "result": "Tool execution result would be here",
                "arguments": body.arguments,
            });
            
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "result": result,
            })))
        }
        None => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "error": format!("Tool not found: {}", body.tool_name),
            })))
        }
    }
}

// Prompt handlers

pub async fn list_prompts(server: web::Data<MCPServer>) -> Result<HttpResponse> {
    let prompts = server.list_prompts();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "prompts": prompts,
    })))
}

pub async fn get_prompt(
    server: web::Data<MCPServer>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let name = path.into_inner();
    
    match server.get_prompt(&name) {
        Some(prompt) => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "prompt": prompt,
            })))
        }
        None => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "error": format!("Prompt not found: {}", name),
            })))
        }
    }
}

// Agent handlers

pub async fn list_agents(server: web::Data<MCPServer>) -> Result<HttpResponse> {
    let agents = server.list_agents();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "agents": agents,
        "count": agents.len(),
    })))
}

#[derive(Debug, Deserialize)]
pub struct RegisterAgentRequest {
    pub name: String,
    pub agent_type: String,
    pub capabilities: Vec<String>,
    pub endpoint: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

pub async fn register_agent(
    server: web::Data<MCPServer>,
    claims: web::ReqData<Claims>,
    body: web::Json<RegisterAgentRequest>,
) -> Result<HttpResponse> {
    let user_id = claims.sub;
    
    let agent = Agent {
        id: Uuid::new_v4(),
        name: body.name.clone(),
        agent_type: body.agent_type.clone(),
        capabilities: body.capabilities.clone(),
        endpoint: body.endpoint.clone(),
        metadata: body.metadata.clone(),
        created_at: chrono::Utc::now(),
        last_active: chrono::Utc::now(),
    };
    
    match server.register_agent(agent.clone()) {
        Ok(_) => {
            info!("✅ Agent registered: {} by user {}", agent.id, user_id);
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "agent": agent,
            })))
        }
        Err(e) => {
            error!("Failed to register agent: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": e.to_string(),
            })))
        }
    }
}

pub async fn unregister_agent(
    server: web::Data<MCPServer>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let agent_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": "Invalid agent ID",
            })));
        }
    };
    
    match server.unregister_agent(agent_id) {
        Ok(_) => {
            info!("✅ Agent unregistered: {}", agent_id);
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "Agent unregistered successfully",
            })))
        }
        Err(e) => {
            error!("Failed to unregister agent: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": e.to_string(),
            })))
        }
    }
}

pub async fn get_agent_context(
    server: web::Data<MCPServer>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let agent_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": "Invalid agent ID",
            })));
        }
    };
    
    match server.get_agent(agent_id) {
        Some(agent) => {
            let contexts = server.list_contexts();
            let agent_contexts: Vec<_> = contexts
                .into_iter()
                .filter(|c| c.source_agent_id == Some(agent_id))
                .collect();
            
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "agent": agent,
                "contexts": agent_contexts,
            })))
        }
        None => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "error": format!("Agent not found: {}", agent_id),
            })))
        }
    }
}

// Context synchronization handlers

#[derive(Debug, Deserialize)]
pub struct SyncContextRequest {
    pub source_agent_id: String,
    pub target_agent_ids: Option<Vec<String>>,
    pub context: serde_json::Value,
    pub context_type: String,
}

pub async fn sync_context(
    server: web::Data<MCPServer>,
    body: web::Json<SyncContextRequest>,
) -> Result<HttpResponse> {
    let source_agent_id = match Uuid::parse_str(&body.source_agent_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": "Invalid source agent ID",
            })));
        }
    };
    
    info!("🔄 Syncing context from agent: {}", source_agent_id);
    
    let shared_context = crate::protocol::SharedContext {
        id: Uuid::new_v4(),
        context_type: body.context_type.clone(),
        data: body.context.clone(),
        source_agent_id: Some(source_agent_id),
        created_at: chrono::Utc::now(),
        expires_at: None,
    };
    
    match server.share_context(shared_context.clone()) {
        Ok(_) => {
            info!("✅ Context synchronized: {}", shared_context.id);
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "context_id": shared_context.id,
                "message": "Context synchronized successfully",
            })))
        }
        Err(e) => {
            error!("Failed to sync context: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": e.to_string(),
            })))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BroadcastRequest {
    pub source_agent_id: String,
    pub message: serde_json::Value,
    pub message_type: String,
}

pub async fn broadcast_to_agents(
    server: web::Data<MCPServer>,
    body: web::Json<BroadcastRequest>,
) -> Result<HttpResponse> {
    let source_agent_id = match Uuid::parse_str(&body.source_agent_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": "Invalid source agent ID",
            })));
        }
    };
    
    info!("📢 Broadcasting message from agent: {}", source_agent_id);
    
    let agents = server.list_agents();
    let target_agents: Vec<_> = agents
        .into_iter()
        .filter(|a| a.id != source_agent_id)
        .collect();
    
    info!("📤 Broadcasting to {} agents", target_agents.len());
    
    // TODO: Implement actual broadcasting logic (e.g., via webhooks or message queue)
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "broadcasted_to": target_agents.len(),
        "agents": target_agents.iter().map(|a| a.id).collect::<Vec<_>>(),
    })))
}

// WebSocket handler for MCP protocol
use actix::{Actor, StreamHandler, AsyncContext, ActorContext, Handler, Message as ActixMessage};
use actix_web_actors::ws::Message;

/// WebSocket actor for MCP connections
pub struct McpWebSocket {
    server: MCPServer,
    context_manager: Arc<ContextManager>,
    agent_id: Option<Uuid>,
}

impl Actor for McpWebSocket {
    type Context = ws::WebsocketContext<Self>;
    
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("MCP WebSocket connection established");
    }
    
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        if let Some(agent_id) = self.agent_id {
            info!("MCP WebSocket connection closed for agent: {}", agent_id);
            // Unregister agent
            let _ = self.server.unregister_agent(agent_id);
        }
    }
}

impl StreamHandler<Result<Message, ws::ProtocolError>> for McpWebSocket {
    fn handle(&mut self, msg: Result<Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(Message::Text(text)) => {
                info!("Received MCP message: {}", text);
                
                // Parse JSON-RPC message
                match serde_json::from_str::<crate::types::JsonRpcRequest>(&text) {
                    Ok(request) => {
                        // Handle MCP protocol message
                        let response = self.handle_mcp_request(request);
                        if let Ok(response_json) = serde_json::to_string(&response) {
                            ctx.text(response_json);
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse MCP request: {}", e);
                        let error_response = crate::types::JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: None,
                            result: None,
                            error: Some(crate::types::JsonRpcError {
                                code: -32700,
                                message: "Parse error".to_string(),
                                data: Some(serde_json::json!({ "error": e.to_string() })),
                            }),
                        };
                        if let Ok(json) = serde_json::to_string(&error_response) {
                            ctx.text(json);
                        }
                    }
                }
            }
            Ok(Message::Binary(_)) => {
                error!("Binary messages not supported in MCP protocol");
            }
            Ok(Message::Ping(msg)) => {
                ctx.pong(&msg);
            }
            Ok(Message::Pong(_)) => {}
            Ok(Message::Close(reason)) => {
                info!("MCP WebSocket closing: {:?}", reason);
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

impl McpWebSocket {
    fn handle_mcp_request(&mut self, request: crate::types::JsonRpcRequest) -> crate::types::JsonRpcResponse {
        use crate::types::*;
        
        match request.method.as_str() {
            "initialize" => {
                // Initialize MCP connection
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::json!({
                        "protocolVersion": MCP_PROTOCOL_VERSION,
                        "capabilities": {
                            "resources": { "subscribe": true, "listChanged": true },
                            "tools": { "listChanged": true },
                            "prompts": { "listChanged": true }
                        },
                        "serverInfo": {
                            "name": "conhub-mcp",
                            "version": "1.0.0"
                        }
                    })),
                    error: None,
                }
            }
            "resources/list" => {
                let resources = self.server.list_resources();
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::json!({ "resources": resources })),
                    error: None,
                }
            }
            "tools/list" => {
                let tools = self.server.list_tools();
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::json!({ "tools": tools })),
                    error: None,
                }
            }
            "prompts/list" => {
                let prompts = self.server.list_prompts();
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::json!({ "prompts": prompts })),
                    error: None,
                }
            }
            _ => {
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32601,
                        message: "Method not found".to_string(),
                        data: Some(serde_json::json!({ "method": request.method })),
                    }),
                }
            }
        }
    }
}

pub async fn mcp_websocket(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<MCPServer>,
    context_manager: web::Data<Arc<ContextManager>>,
) -> Result<HttpResponse> {
    let ws = McpWebSocket {
        server: server.get_ref().clone(),
        context_manager: context_manager.get_ref().clone(),
        agent_id: None,
    };
    
    ws::start(ws, &req, stream)
}
