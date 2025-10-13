use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;


#[async_trait]
pub trait DataSourceConnector: Send + Sync {
    
    #[allow(dead_code)]
    async fn validate(&self, credentials: &HashMap<String, String>) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    
    
    async fn connect(&mut self, credentials: &HashMap<String, String>, config: &Value) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    
    
    #[allow(dead_code)]
    async fn sync(&self, data_source: &DataSource) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>>;
    
    
    async fn fetch_branches(&self, repo_url: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>>;
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    pub id: String,
    pub source_type: String,
    pub name: String,
    pub config: Value,
    pub credentials: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
    pub metadata: Value,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: String,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub url: String,
    pub private: bool,
    pub metadata: Value,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub documents: Vec<Document>,
    pub repositories: Vec<Repository>,
}


pub struct DataSourceFactory;

impl DataSourceFactory {
    pub fn create_connector(source_type: &str) -> Result<Box<dyn DataSourceConnector>, Box<dyn std::error::Error + Send + Sync>> {
        match source_type {
            "github" => Ok(Box::new(crate::sources::repositories::github::GitHubConnector::new())),
            "bitbucket" => Ok(Box::new(crate::sources::repositories::bitbucket::BitbucketConnector::new())),
            "googledrive" => Ok(Box::new(crate::sources::documents::googledrive::GoogleDriveConnector::new())),
            "notion" => Ok(Box::new(crate::sources::documents::notion::NotionConnector::new())),
            "url" => Ok(Box::new(crate::sources::urls::UrlConnector::new())),
            _ => Err(format!("Unsupported data source type: {}", source_type).into()),
        }
    }
}