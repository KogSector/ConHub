pub mod github;
pub mod bitbucket;
pub mod google_drive;
pub mod notion;
pub mod url;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    pub id: String,
    pub name: String,
    pub source_type: String,
    pub credentials: HashMap<String, String>,
    pub config: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: String,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub url: String,
    pub private: bool,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub documents: Vec<Document>,
    pub repositories: Vec<Repository>,
}

#[async_trait]
pub trait ConnectorInterface: Send + Sync {
    async fn validate(&self, credentials: &HashMap<String, String>) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    async fn connect(&mut self, credentials: &HashMap<String, String>, config: &serde_json::Value) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    async fn sync(&self, data_source: &DataSource) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>>;
    async fn fetch_branches(&self, repo_url: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>>;
}