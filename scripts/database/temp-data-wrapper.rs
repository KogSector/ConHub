// Temporary data wrapper for Rust services when Auth=false
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use serde_json::Value;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct TempUser {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TempConnectedAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub connector_type: String,
    pub account_name: String,
    pub account_identifier: String,
    pub credentials: Value,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TempDocument {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub doc_type: String,
    pub source: String,
    pub size: String,
    pub tags: Vec<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TempRepository {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub url: String,
    pub private: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TempAgent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub agent_type: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

pub struct TempDataStore {
    users: Arc<RwLock<HashMap<Uuid, TempUser>>>,
    connected_accounts: Arc<RwLock<HashMap<Uuid, TempConnectedAccount>>>,
    documents: Arc<RwLock<HashMap<Uuid, TempDocument>>>,
    repositories: Arc<RwLock<HashMap<Uuid, TempRepository>>>,
    agents: Arc<RwLock<HashMap<Uuid, TempAgent>>>,
}

impl TempDataStore {
    pub fn new() -> Self {
        let store = Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            connected_accounts: Arc::new(RwLock::new(HashMap::new())),
            documents: Arc::new(RwLock::new(HashMap::new())),
            repositories: Arc::new(RwLock::new(HashMap::new())),
            agents: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Add default dev user
        let dev_user = TempUser {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            email: "dev@conhub.com".to_string(),
            name: Some("Development User".to_string()),
            created_at: Utc::now(),
        };
        
        store.users.write().unwrap().insert(dev_user.id, dev_user);
        store
    }
    
    // Users
    pub fn get_user(&self, id: Uuid) -> Option<TempUser> {
        self.users.read().unwrap().get(&id).cloned()
    }
    
    pub fn create_user(&self, email: String, name: Option<String>) -> TempUser {
        let user = TempUser {
            id: Uuid::new_v4(),
            email,
            name,
            created_at: Utc::now(),
        };
        self.users.write().unwrap().insert(user.id, user.clone());
        user
    }
    
    // Connected Accounts
    pub fn get_connected_accounts(&self, user_id: Uuid) -> Vec<TempConnectedAccount> {
        self.connected_accounts
            .read()
            .unwrap()
            .values()
            .filter(|account| account.user_id == user_id)
            .cloned()
            .collect()
    }
    
    pub fn create_connected_account(
        &self,
        user_id: Uuid,
        connector_type: String,
        account_name: String,
        credentials: Value,
    ) -> TempConnectedAccount {
        let account = TempConnectedAccount {
            id: Uuid::new_v4(),
            user_id,
            connector_type,
            account_name,
            account_identifier: "temp".to_string(),
            credentials,
            status: "connected".to_string(),
            created_at: Utc::now(),
        };
        self.connected_accounts.write().unwrap().insert(account.id, account.clone());
        account
    }
    
    // Documents
    pub fn get_documents(&self, user_id: Uuid) -> Vec<TempDocument> {
        self.documents
            .read()
            .unwrap()
            .values()
            .filter(|doc| doc.user_id == user_id)
            .cloned()
            .collect()
    }
    
    pub fn create_document(
        &self,
        user_id: Uuid,
        name: String,
        doc_type: String,
        source: String,
    ) -> TempDocument {
        let doc = TempDocument {
            id: Uuid::new_v4(),
            user_id,
            name,
            doc_type,
            source,
            size: "1KB".to_string(),
            tags: vec![],
            status: "processed".to_string(),
            created_at: Utc::now(),
        };
        self.documents.write().unwrap().insert(doc.id, doc.clone());
        doc
    }
    
    pub fn delete_document(&self, id: Uuid) -> bool {
        self.documents.write().unwrap().remove(&id).is_some()
    }
    
    // Repositories
    pub fn get_repositories(&self, user_id: Uuid) -> Vec<TempRepository> {
        self.repositories
            .read()
            .unwrap()
            .values()
            .filter(|repo| repo.user_id == user_id)
            .cloned()
            .collect()
    }
    
    pub fn create_repository(
        &self,
        user_id: Uuid,
        name: String,
        full_name: String,
        url: String,
    ) -> TempRepository {
        let repo = TempRepository {
            id: Uuid::new_v4(),
            user_id,
            name,
            full_name,
            description: Some("Sample repository".to_string()),
            url,
            private: false,
            created_at: Utc::now(),
        };
        self.repositories.write().unwrap().insert(repo.id, repo.clone());
        repo
    }
    
    // Agents
    pub fn get_agents(&self, user_id: Uuid) -> Vec<TempAgent> {
        self.agents
            .read()
            .unwrap()
            .values()
            .filter(|agent| agent.user_id == user_id)
            .cloned()
            .collect()
    }
    
    pub fn create_agent(
        &self,
        user_id: Uuid,
        name: String,
        agent_type: String,
    ) -> TempAgent {
        let agent = TempAgent {
            id: Uuid::new_v4(),
            user_id,
            name,
            agent_type,
            status: "Connected".to_string(),
            created_at: Utc::now(),
        };
        self.agents.write().unwrap().insert(agent.id, agent.clone());
        agent
    }
    
    pub fn delete_agent(&self, id: Uuid) -> bool {
        self.agents.write().unwrap().remove(&id).is_some()
    }
    
    // Utility
    pub fn get_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("users".to_string(), self.users.read().unwrap().len());
        stats.insert("connected_accounts".to_string(), self.connected_accounts.read().unwrap().len());
        stats.insert("documents".to_string(), self.documents.read().unwrap().len());
        stats.insert("repositories".to_string(), self.repositories.read().unwrap().len());
        stats.insert("agents".to_string(), self.agents.read().unwrap().len());
        stats
    }
    
    pub fn reset(&self) {
        self.users.write().unwrap().clear();
        self.connected_accounts.write().unwrap().clear();
        self.documents.write().unwrap().clear();
        self.repositories.write().unwrap().clear();
        self.agents.write().unwrap().clear();
        
        // Re-add dev user
        let dev_user = TempUser {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            email: "dev@conhub.com".to_string(),
            name: Some("Development User".to_string()),
            created_at: Utc::now(),
        };
        self.users.write().unwrap().insert(dev_user.id, dev_user);
    }
}

impl Default for TempDataStore {
    fn default() -> Self {
        Self::new()
    }
}

// Singleton instance
use std::sync::Once;
static INIT: Once = Once::new();
static mut TEMP_STORE: Option<TempDataStore> = None;

pub fn get_temp_store() -> &'static TempDataStore {
    unsafe {
        INIT.call_once(|| {
            TEMP_STORE = Some(TempDataStore::new());
        });
        TEMP_STORE.as_ref().unwrap()
    }
}

// Macro for easy conditional database access
#[macro_export]
macro_rules! with_db_or_temp {
    ($pool:expr, $temp_fn:expr, $db_fn:expr) => {
        match $pool {
            Some(pool) => $db_fn(pool).await,
            None => {
                let temp_store = crate::temp_data_wrapper::get_temp_store();
                Ok($temp_fn(temp_store))
            }
        }
    };
}