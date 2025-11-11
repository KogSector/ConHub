use sqlx::PgPool;
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use anyhow::{Result, Context};
use tracing::{info, warn, error};

use crate::types::*;

/// Manages context and state for connected AI agents
pub struct ContextManager {
    pool: Option<PgPool>,
    data_service_url: String,
    connected_agents: Arc<RwLock<HashMap<Uuid, ConnectedAgent>>>,
    agent_contexts: Arc<RwLock<HashMap<Uuid, AgentContext>>>,
}

impl ContextManager {
    pub fn new(pool: Option<PgPool>, data_service_url: String) -> Self {
        Self {
            pool,
            data_service_url,
            connected_agents: Arc::new(RwLock::new(HashMap::new())),
            agent_contexts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a new agent connection
    pub async fn register_agent(&self, agent: ConnectedAgent) -> Result<()> {
        let agent_id = agent.id;
        
        // Store in memory
        {
            let mut agents = self.connected_agents.write().await;
            agents.insert(agent_id, agent.clone());
        }
        
        // Initialize context for the agent
        self.initialize_agent_context(agent_id, agent.user_id).await?;
        
        info!("Registered agent: {} ({})", agent.name, agent_id);
        Ok(())
    }
    
    /// Unregister an agent
    pub async fn unregister_agent(&self, agent_id: Uuid) -> Result<()> {
        {
            let mut agents = self.connected_agents.write().await;
            agents.remove(&agent_id);
        }
        
        {
            let mut contexts = self.agent_contexts.write().await;
            contexts.remove(&agent_id);
        }
        
        info!("Unregistered agent: {}", agent_id);
        Ok(())
    }
    
    /// Get agent by ID
    pub async fn get_agent(&self, agent_id: Uuid) -> Option<ConnectedAgent> {
        let agents = self.connected_agents.read().await;
        agents.get(&agent_id).cloned()
    }
    
    /// List all connected agents for a user
    pub async fn list_user_agents(&self, user_id: Uuid) -> Vec<ConnectedAgent> {
        let agents = self.connected_agents.read().await;
        agents.values()
            .filter(|agent| agent.user_id == user_id)
            .cloned()
            .collect()
    }
    
    /// Initialize context for an agent
    async fn initialize_agent_context(&self, agent_id: Uuid, user_id: Uuid) -> Result<()> {
        // Fetch user's resources from data service
        let resources = self.fetch_user_resources(user_id).await?;
        let recent_documents = self.fetch_recent_documents(user_id).await?;
        let active_repositories = self.fetch_active_repositories(user_id).await?;
        
        let context = AgentContext {
            agent_id,
            user_id,
            resources,
            recent_documents,
            active_repositories,
            metadata: HashMap::new(),
        };
        
        let mut contexts = self.agent_contexts.write().await;
        contexts.insert(agent_id, context);
        
        Ok(())
    }
    
    /// Get context for an agent
    pub async fn get_agent_context(&self, agent_id: Uuid) -> Option<AgentContext> {
        let contexts = self.agent_contexts.read().await;
        contexts.get(&agent_id).cloned()
    }
    
    /// Update agent context
    pub async fn update_agent_context(&self, agent_id: Uuid, user_id: Uuid) -> Result<()> {
        self.initialize_agent_context(agent_id, user_id).await
    }
    
    /// Fetch user's resources via GraphQL
    async fn fetch_user_resources(&self, user_id: Uuid) -> Result<Vec<Resource>> {
        // TODO: Make GraphQL request to data service
        // For now, return mock data
        Ok(vec![
            Resource {
                uri: format!("conhub://documents/user/{}", user_id),
                name: "User Documents".to_string(),
                description: Some("All documents connected by the user".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: format!("conhub://repositories/user/{}", user_id),
                name: "Code Repositories".to_string(),
                description: Some("Connected code repositories".to_string()),
                mime_type: Some("application/json".to_string()),
            },
        ])
    }
    
    /// Fetch recent documents
    async fn fetch_recent_documents(&self, user_id: Uuid) -> Result<Vec<DocumentContext>> {
        // TODO: Query via GraphQL
        Ok(vec![])
    }
    
    /// Fetch active repositories
    async fn fetch_active_repositories(&self, user_id: Uuid) -> Result<Vec<RepositoryContext>> {
        // TODO: Query via GraphQL
        Ok(vec![])
    }
    
    /// Broadcast a sync message to agents
    pub async fn broadcast_sync_message(&self, message: SyncMessage) -> Result<()> {
        let agents = self.connected_agents.read().await;
        
        let target_agents: Vec<Uuid> = if let Some(to_agents) = &message.to_agents {
            to_agents.clone()
        } else {
            // Broadcast to all agents of the same user
            if let Some(from_agent) = agents.get(&message.from_agent) {
                agents.values()
                    .filter(|agent| agent.user_id == from_agent.user_id && agent.id != message.from_agent)
                    .map(|agent| agent.id)
                    .collect()
            } else {
                vec![]
            }
        };
        
        info!("Broadcasting sync message to {} agents", target_agents.len());
        
        // TODO: Implement actual message delivery (WebSocket, SSE, etc.)
        // For now, just log
        for agent_id in target_agents {
            info!("Would send sync message to agent: {}", agent_id);
        }
        
        Ok(())
    }
    
    /// Update agent activity timestamp
    pub async fn update_agent_activity(&self, agent_id: Uuid) {
        let mut agents = self.connected_agents.write().await;
        if let Some(agent) = agents.get_mut(&agent_id) {
            agent.last_activity = chrono::Utc::now();
        }
    }
}
