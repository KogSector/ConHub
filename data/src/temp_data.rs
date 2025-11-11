use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use serde_json::Value;
use chrono::{DateTime, Utc};

lazy_static::lazy_static! {
    static ref TEMP_STORE: TempDataStore = TempDataStore::new();
}

#[derive(Debug, Clone)]
pub struct TempUser {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
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

pub struct TempDataStore {
    users: Arc<RwLock<HashMap<Uuid, TempUser>>>,
    documents: Arc<RwLock<HashMap<Uuid, TempDocument>>>,
}

impl TempDataStore {
    pub fn new() -> Self {
        let store = Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            documents: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Add default dev user
        let dev_user = TempUser {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            email: "dev@conhub.com".to_string(),
            name: Some("Development User".to_string()),
            created_at: Utc::now(),
        };
        
        store.users.write().unwrap().insert(dev_user.id, dev_user);
        
        // Add sample documents
        let sample_docs = vec![
            TempDocument {
                id: Uuid::new_v4(),
                user_id: dev_user.id,
                name: "README.md".to_string(),
                doc_type: "markdown".to_string(),
                source: "local".to_string(),
                size: "2.1 KB".to_string(),
                tags: vec!["documentation".to_string()],
                status: "processed".to_string(),
                created_at: Utc::now(),
            },
            TempDocument {
                id: Uuid::new_v4(),
                user_id: dev_user.id,
                name: "API Guide.pdf".to_string(),
                doc_type: "pdf".to_string(),
                source: "google_drive".to_string(),
                size: "1.8 MB".to_string(),
                tags: vec!["api", "guide".to_string()],
                status: "processed".to_string(),
                created_at: Utc::now(),
            },
        ];
        
        for doc in sample_docs {
            store.documents.write().unwrap().insert(doc.id, doc);
        }
        
        store
    }
    
    pub fn get_documents(&self, user_id: Uuid) -> Vec<TempDocument> {
        self.documents
            .read()
            .unwrap()
            .values()
            .filter(|doc| doc.user_id == user_id)
            .cloned()
            .collect()
    }
    
    pub fn create_document(&self, user_id: Uuid, name: String, doc_type: String, source: String) -> TempDocument {
        let doc = TempDocument {
            id: Uuid::new_v4(),
            user_id,
            name,
            doc_type,
            source,
            size: "1 KB".to_string(),
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
}

pub fn get_temp_store() -> &'static TempDataStore {
    &TEMP_STORE
}