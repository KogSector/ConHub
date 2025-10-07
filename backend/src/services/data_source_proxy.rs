use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataSourceRequest {
    pub url: String,
    pub api_key: Option<String>,
    pub source_type: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataSourceResponse {
    pub id: String,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataSourceListItem {
    pub id: String,
    pub name: String,
    pub source_type: String,
    pub url: String,
    pub last_synced: Option<String>,
}

pub async fn connect_data_source(
    client: &Client,
    langchain_url: &str,
    req: &DataSourceRequest,
) -> Result<DataSourceResponse, Box<dyn Error>> {
    let url = format!("{}/api/datasources/connect", langchain_url);
    
    let response = client.post(&url)
        .json(req)
        .send()
        .await?
        .json::<DataSourceResponse>()
        .await?;
    
    Ok(response)
}

pub async fn list_data_sources(
    client: &Client,
    langchain_url: &str,
) -> Result<Vec<DataSourceListItem>, Box<dyn Error>> {
    let url = format!("{}/api/datasources", langchain_url);
    
    let response = client.get(&url)
        .send()
        .await?
        .json::<Vec<DataSourceListItem>>()
        .await?;
    
    Ok(response)
}

pub async fn sync_data_source(
    client: &Client,
    langchain_url: &str,
    id: &str,
) -> Result<DataSourceResponse, Box<dyn Error>> {
    let url = format!("{}/api/datasources/{}/sync", langchain_url, id);
    
    let response = client.post(&url)
        .send()
        .await?
        .json::<DataSourceResponse>()
        .await?;
    
    Ok(response)
}