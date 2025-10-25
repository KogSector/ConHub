use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use conhub_models::mcp::*;
use super::mcp::McpClient;

// ConHub MCP Server implementation
#[derive(Clone)]
pub struct ConHubMcpServer {
    client: McpClient,
    server_registry: Arc<RwLock<HashMap<ServerId, ServerInfo>>>,
}

impl ConHubMcpServer {
    pub fn new() -> Self {
        let client = McpClient::with_config(super::mcp::McpClientConfig::default())
            .expect("Failed to create MCP client");
        
        Self {
            client,
            server_registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_server(&self, server_id: ServerId, server_info: ServerInfo) {
        let mut registry = self.server_registry.write().await;
        registry.insert(server_id, server_info);
    }

    pub async fn get_client(&self) -> &McpClient {
        &self.client
    }

    pub fn server_info(&self) -> ServerInfo {
        ServerInfo {
            name: "ConHub MCP Server".to_string(),
            version: "1.0.0".to_string(),
        }
    }

    pub fn capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            resources: Some(ResourceCapabilities {
                subscribe: true,
                list_changed: true,
            }),
            tools: Some(ToolCapabilities {
                list_changed: true,
            }),
            prompts: Some(PromptCapabilities {
                list_changed: true,
            }),
            logging: Some(LoggingCapabilities {}),
        }
    }
}