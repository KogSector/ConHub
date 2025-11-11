use std::sync::Arc;
use parking_lot::RwLock;
use dashmap::DashMap;
use uuid::Uuid;
use sqlx::PgPool;
use tracing::{info, error};

use crate::protocol::{Agent, Resource, Tool, Prompt, SharedContext};
use crate::error::MCPError;

/// MCP Server manages resources, tools, prompts, and agent connections
pub struct MCPServer {
    db_pool: Option<PgPool>,
    agents: Arc<DashMap<Uuid, Agent>>,
    resources: Arc<RwLock<Vec<Resource>>>,
    tools: Arc<RwLock<Vec<Tool>>>,
    prompts: Arc<RwLock<Vec<Prompt>>>,
    shared_contexts: Arc<DashMap<Uuid, SharedContext>>,
}

impl MCPServer {
    pub fn new(db_pool: Option<PgPool>) -> Self {
        let server = Self {
            db_pool,
            agents: Arc::new(DashMap::new()),
            resources: Arc::new(RwLock::new(Vec::new())),
            tools: Arc::new(RwLock::new(Vec::new())),
            prompts: Arc::new(RwLock::new(Vec::new())),
            shared_contexts: Arc::new(DashMap::new()),
        };
        
        // Initialize default resources, tools, and prompts
        server.initialize_defaults();
        
        server
    }
    
    fn initialize_defaults(&self) {
        info!("🔧 Initializing default MCP resources, tools, and prompts...");
        
        // Initialize default tools
        let mut tools = self.tools.write();
        tools.push(Tool {
            name: "query_context".to_string(),
            description: "Query the knowledge base context".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query"
                    },
                    "filters": {
                        "type": "object",
                        "description": "Optional filters for the search"
                    }
                },
                "required": ["query"]
            }),
            returns: Some(serde_json::json!({
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "content": { "type": "string" },
                        "score": { "type": "number" },
                        "metadata": { "type": "object" }
                    }
                }
            })),
        });
        
        tools.push(Tool {
            name: "sync_context".to_string(),
            description: "Synchronize context between agents".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "target_agents": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "List of target agent IDs"
                    },
                    "context": {
                        "type": "object",
                        "description": "Context data to synchronize"
                    }
                },
                "required": ["context"]
            }),
            returns: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "synced_agents": { "type": "number" },
                    "failed_agents": { "type": "number" }
                }
            })),
        });
        
        drop(tools);
        
        // Initialize default prompts
        let mut prompts = self.prompts.write();
        prompts.push(Prompt {
            name: "code_review".to_string(),
            description: "Review code for best practices and issues".to_string(),
            template: "Review the following code and provide feedback:\n\n```{{language}}\n{{code}}\n```\n\nFocus on: {{focus_areas}}".to_string(),
            parameters: vec![
                crate::protocol::PromptParameter {
                    name: "language".to_string(),
                    description: "Programming language".to_string(),
                    required: true,
                },
                crate::protocol::PromptParameter {
                    name: "code".to_string(),
                    description: "Code to review".to_string(),
                    required: true,
                },
                crate::protocol::PromptParameter {
                    name: "focus_areas".to_string(),
                    description: "Specific areas to focus on".to_string(),
                    required: false,
                },
            ],
        });
        
        drop(prompts);
        
        info!("✅ Default MCP resources initialized");
    }
    
    // Resource methods
    pub fn list_resources(&self) -> Vec<Resource> {
        self.resources.read().clone()
    }
    
    pub fn add_resource(&self, resource: Resource) {
        let mut resources = self.resources.write();
        resources.push(resource);
    }
    
    pub fn get_resource(&self, uri: &str) -> Option<Resource> {
        self.resources.read()
            .iter()
            .find(|r| r.uri == uri)
            .cloned()
    }
    
    // Tool methods
    pub fn list_tools(&self) -> Vec<Tool> {
        self.tools.read().clone()
    }
    
    pub fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tools.read()
            .iter()
            .find(|t| t.name == name)
            .cloned()
    }
    
    // Prompt methods
    pub fn list_prompts(&self) -> Vec<Prompt> {
        self.prompts.read().clone()
    }
    
    pub fn get_prompt(&self, name: &str) -> Option<Prompt> {
        self.prompts.read()
            .iter()
            .find(|p| p.name == name)
            .cloned()
    }
    
    // Agent methods
    pub fn register_agent(&self, agent: Agent) -> Result<Agent, MCPError> {
        info!("🤖 Registering agent: {} ({})", agent.name, agent.id);
        self.agents.insert(agent.id, agent.clone());
        Ok(agent)
    }
    
    pub fn unregister_agent(&self, agent_id: Uuid) -> Result<(), MCPError> {
        info!("🔌 Unregistering agent: {}", agent_id);
        self.agents.remove(&agent_id);
        Ok(())
    }
    
    pub fn get_agent(&self, agent_id: Uuid) -> Option<Agent> {
        self.agents.get(&agent_id).map(|a| a.clone())
    }
    
    pub fn list_agents(&self) -> Vec<Agent> {
        self.agents.iter().map(|entry| entry.value().clone()).collect()
    }
    
    // Context methods
    pub fn share_context(&self, context: SharedContext) -> Result<(), MCPError> {
        info!("📤 Sharing context: {} ({})", context.context_type, context.id);
        self.shared_contexts.insert(context.id, context);
        Ok(())
    }
    
    pub fn get_context(&self, context_id: Uuid) -> Option<SharedContext> {
        self.shared_contexts.get(&context_id).map(|c| c.clone())
    }
    
    pub fn list_contexts(&self) -> Vec<SharedContext> {
        self.shared_contexts.iter().map(|entry| entry.value().clone()).collect()
    }
}

impl Clone for MCPServer {
    fn clone(&self) -> Self {
        Self {
            db_pool: self.db_pool.clone(),
            agents: Arc::clone(&self.agents),
            resources: Arc::clone(&self.resources),
            tools: Arc::clone(&self.tools),
            prompts: Arc::clone(&self.prompts),
            shared_contexts: Arc::clone(&self.shared_contexts),
        }
    }
}
