use reqwest::Client;
use serde_json::json;
use crate::models::SearchRequest;

pub async fn ask_ai_question(
    client: &Client,
    haystack_url: &str,
    request: &SearchRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let payload = json!({
        "query": request.query,
        "top_k": request.limit.unwrap_or(5)
    });
    
    let response = client
        .post(&format!("{}/ask", haystack_url))
        .json(&payload)
        .send()
        .await?;
    
    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Err(format!("AI service error: {}", response.status()).into())
    }
}