use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use crate::models::SearchRequest;

pub async fn universal_search(
    client: &Client,
    langchain_url: &str,
    haystack_url: &str,
    lexor_url: &str,
    request: &SearchRequest,
) -> HashMap<String, serde_json::Value> {
    let mut results = HashMap::new();
    
    
    if let Ok(result) = search_langchain(client, langchain_url, request).await {
        results.insert("semantic".to_string(), result);
    }
    
    
    if let Ok(result) = search_lexor(client, lexor_url, request).await {
        results.insert("code".to_string(), result);
    }
    
    
    if let Ok(result) = search_haystack(client, haystack_url, request).await {
        results.insert("documents".to_string(), result);
    }
    
    results
}

async fn search_langchain(
    client: &Client,
    url: &str,
    request: &SearchRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let payload = json!({
        "query": request.query,
        "limit": request.limit.unwrap_or(10)
    });
    
    let response = client
        .post(&format!("{}/api/search/universal", url))
        .json(&payload)
        .send()
        .await?;
    
    Ok(response.json().await?)
}

async fn search_lexor(
    client: &Client,
    url: &str,
    request: &SearchRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let payload = json!({
        "query": request.query,
        "limit": request.limit.unwrap_or(10)
    });
    
    let response = client
        .post(&format!("{}/api/search", url))
        .json(&payload)
        .send()
        .await?;
    
    Ok(response.json().await?)
}

async fn search_haystack(
    client: &Client,
    url: &str,
    request: &SearchRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let payload = json!({
        "query": request.query,
        "top_k": request.limit.unwrap_or(10)
    });
    
    let response = client
        .post(&format!("{}/search", url))
        .json(&payload)
        .send()
        .await?;
    
    Ok(response.json().await?)
}