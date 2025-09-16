use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DataSourceRequest {
    pub source_type: String,
    pub credentials: serde_json::Value,
    pub config: serde_json::Value,
}

pub async fn connect_data_source(
    client: &Client,
    langchain_url: &str,
    req: &DataSourceRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("{}/api/data-sources/connect", langchain_url);
    
    let response = client
        .post(&url)
        .json(req)
        .send()
        .await?;

    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        Ok(result)
    } else {
        Err(format!("Data source connection failed: {}", response.status()).into())
    }
}

pub async fn list_data_sources(
    client: &Client,
    langchain_url: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("{}/api/data-sources", langchain_url);
    
    let response = client
        .get(&url)
        .send()
        .await?;

    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        Ok(result)
    } else {
        Err(format!("Failed to list data sources: {}", response.status()).into())
    }
}