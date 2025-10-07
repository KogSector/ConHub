use crate::models::{SearchRequest, AiResponse};
use reqwest::Client;
use std::error::Error;

pub async fn ask_ai_question(
    client: &Client,
    haystack_url: &str,
    req: &SearchRequest,
) -> Result<AiResponse, Box<dyn Error>> {
    let url = format!("{}/api/v1/ask", haystack_url);
    
    let response = client.post(&url)
        .json(&req)
        .send()
        .await?
        .json::<AiResponse>()
        .await?;
    
    Ok(response)
}

pub async fn generate_code(
    client: &Client,
    haystack_url: &str,
    prompt: &str,
) -> Result<String, Box<dyn Error>> {
    let url = format!("{}/api/v1/generate", haystack_url);
    
    let response = client.post(&url)
        .json(&serde_json::json!({
            "prompt": prompt,
            "max_length": 1000,
            "temperature": 0.7
        }))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    
    Ok(response["generated_text"].as_str().unwrap_or("").to_string())
}

pub async fn analyze_code(
    client: &Client,
    haystack_url: &str,
    code: &str,
) -> Result<serde_json::Value, Box<dyn Error>> {
    let url = format!("{}/api/v1/analyze", haystack_url);
    
    let response = client.post(&url)
        .json(&serde_json::json!({
            "code": code,
            "detailed": true
        }))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    
    Ok(response)
}