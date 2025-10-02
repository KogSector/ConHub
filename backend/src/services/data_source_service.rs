use crate::services::connectors::{
    ConnectorInterface, DataSource, SyncResult,
    github::GitHubConnector,
    bitbucket::BitbucketConnector,
    google_drive::GoogleDriveConnector,
    notion::NotionConnector,
    url::UrlConnector,
};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{error, info};

pub struct DataSourceService;

impl DataSourceService {
    pub fn new() -> Self {
        Self
    }

    pub async fn connect_data_source(
        &self,
        source_type: &str,
        credentials: HashMap<String, String>,
        config: Value,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut connector = self.create_connector(source_type)?;
        
        // Validate credentials first
        connector.validate(&credentials).await?;
        
        // Then connect
        connector.connect(&credentials, &config).await?;
        
        info!("Successfully connected to {} data source", source_type);
        Ok(true)
    }

    pub async fn fetch_branches(
        &self,
        source_type: &str,
        repo_url: &str,
        credentials: HashMap<String, String>,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut connector = self.create_connector(source_type)?;
        
        // Connect first
        connector.connect(&credentials, &Value::Null).await?;
        
        // Then fetch branches
        connector.fetch_branches(repo_url).await
    }

    pub async fn sync_data_source(
        &self,
        data_source: &DataSource,
    ) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut connector = self.create_connector(&data_source.source_type)?;
        
        // Connect using stored credentials
        connector.connect(&data_source.credentials, &data_source.config).await?;
        
        // Sync data
        connector.sync(data_source).await
    }

    fn create_connector(&self, source_type: &str) -> Result<Box<dyn ConnectorInterface>, Box<dyn std::error::Error + Send + Sync>> {
        match source_type {
            "github" => Ok(Box::new(GitHubConnector::new())),
            "bitbucket" => Ok(Box::new(BitbucketConnector::new())),
            "google-drive" => Ok(Box::new(GoogleDriveConnector::new())),
            "notion" => Ok(Box::new(NotionConnector::new())),
            "url" => Ok(Box::new(UrlConnector::new())),
            _ => Err(format!("Unsupported data source type: {}", source_type).into()),
        }
    }
}