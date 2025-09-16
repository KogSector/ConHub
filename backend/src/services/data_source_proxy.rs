use reqwest::Client;
use serde_json::json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct DataSourceRequest {
    pub source_type: String,
    pub config: serde_json::Value,
    pub credentials: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct DataSourceResponse {
    pub id: String,
    pub status: String,
    pub message: String,
}

pub async fn connect_data_source(
    client: &Client,
    langchain_url: &str,
    request: &DataSourceRequest,
) -> Result<DataSourceResponse, Box<dyn std::error::Error>> {
    let payload = json!({
        "type": request.source_type,
        "config": request.config,
        "credentials": request.credentials
    });
    
    let response = client
        .post(&format!("{}/api/data-sources/connect", langchain_url))
        .json(&payload)
        .send()
        .await?;
    
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        Ok(DataSourceResponse {
            id: result["id"].as_str().unwrap_or("unknown").to_string(),
            status: "connected".to_string(),
            message: "Data source connected successfully".to_string(),
        })
    } else {
        Err(format!("Data source connection failed: {}", response.status()).into())
    }
}

pub async fn list_data_sources(
    client: &Client,
    langchain_url: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let response = client
        .get(&format!("{}/api/data-sources", langchain_url))
        .send()
        .await?;
    
    Ok(response.json().await?)
}