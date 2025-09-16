use reqwest::Client;
use serde_json::json;
use crate::models::ConnectRepoRequest;

pub async fn connect_repository(
    client: &Client,
    langchain_url: &str,
    req: &ConnectRepoRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("{}/api/repositories/connect", langchain_url);
    
    let response = client
        .post(&url)
        .json(&json!({
            "url": req.repo_url,
            "access_token": req.access_token,
            "branch": "main"
        }))
        .send()
        .await?;

    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        Ok(result)
    } else {
        Err(format!("Repository connection failed: {}", response.status()).into())
    }
}