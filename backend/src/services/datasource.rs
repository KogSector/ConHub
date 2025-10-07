use crate::models::DataSourceConfig;
use reqwest::Client;
use std::error::Error;
use std::collections::HashMap;

pub async fn register_datasource(
    client: &Client,
    api_url: &str,
    config: &DataSourceConfig,
) -> Result<String, Box<dyn Error>> {
    let url = format!("{}/api/datasources/register", api_url);
    
    let response = client.post(&url)
        .json(config)
        .send()
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    
    Ok(response.get("id").cloned().unwrap_or_default())
}

pub async fn get_datasource(
    client: &Client,
    api_url: &str,
    id: &str,
) -> Result<DataSourceConfig, Box<dyn Error>> {
    let url = format!("{}/api/datasources/{}", api_url, id);
    
    let response = client.get(&url)
        .send()
        .await?
        .json::<DataSourceConfig>()
        .await?;
    
    Ok(response)
}

pub async fn list_datasources(
    client: &Client,
    api_url: &str,
) -> Result<Vec<DataSourceConfig>, Box<dyn Error>> {
    let url = format!("{}/api/datasources", api_url);
    
    let response = client.get(&url)
        .send()
        .await?
        .json::<Vec<DataSourceConfig>>()
        .await?;
    
    Ok(response)
}

pub async fn update_datasource(
    client: &Client,
    api_url: &str,
    id: &str,
    config: &DataSourceConfig,
) -> Result<(), Box<dyn Error>> {
    let url = format!("{}/api/datasources/{}", api_url, id);
    
    client.put(&url)
        .json(config)
        .send()
        .await?;
    
    Ok(())
}

pub async fn delete_datasource(
    client: &Client,
    api_url: &str,
    id: &str,
) -> Result<(), Box<dyn Error>> {
    let url = format!("{}/api/datasources/{}", api_url, id);
    
    client.delete(&url)
        .send()
        .await?;
    
    Ok(())
}