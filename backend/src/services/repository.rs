use reqwest::Client;
use serde_json::json;
use crate::models::ConnectRepoRequest;

pub async fn connect_repository(
    client: &Client,
    langchain_url: &str,
    request: &ConnectRepoRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let payload = json!({
        "repoUrl": request.repo_url,
        "accessToken": request.access_token
    });
    
    let response = client
        .post(&format!("{}/api/indexing/repository", langchain_url))
        .json(&payload)
        .send()
        .await?;
    
    if response.status().is_success() {
        let result = response.json().await?;
        Ok(result)
    } else {
        Err(format!("Repository connection failed: {}", response.status()).into())
    }
}