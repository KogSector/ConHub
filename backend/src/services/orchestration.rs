
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
#[allow(dead_code)]
pub struct ServiceOrchestrator {
    client: Client,
    services: Arc<HashMap<String, String>>,
}

impl ServiceOrchestrator {
    #[allow(dead_code)]
    pub fn new(langchain_url: String, haystack_url: String, unified_indexer_url: String) -> Self {
        let mut services = HashMap::new();
        services.insert("langchain".to_string(), langchain_url);
        services.insert("haystack".to_string(), haystack_url);
        services.insert("unified_indexer".to_string(), unified_indexer_url);
        
        Self {
            client: Client::new(),
            services: Arc::new(services),
        }
    }
    
    #[allow(dead_code)]
    pub async fn start_full_indexing(&self, repo_url: &str, access_token: Option<&str>) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let mut results = HashMap::new();
        
        
        if let Some(langchain_url) = self.services.get("langchain") {
            let payload = json!({
                "repoUrl": repo_url,
                "accessToken": access_token
            });
            
            if let Ok(response) = self.client
                .post(&format!("{}/api/indexing/repository", langchain_url))
                .json(&payload)
                .send()
                .await
            {
                if let Ok(result) = response.json::<serde_json::Value>().await {
                    results.insert("langchain", result);
                }
            }
        }
        
        
        if let Some(unified_indexer_url) = self.services.get("unified_indexer") {
            let payload = json!({
                "repository_url": repo_url,
                "access_token": access_token
            });
            
            if let Ok(response) = self.client
                .post(&format!("{}/api/projects", unified_indexer_url))
                .json(&payload)
                .send()
                .await
            {
                if let Ok(result) = response.json::<serde_json::Value>().await {
                    results.insert("unified_indexer", result);
                }
            }
        }
        
        Ok(json!({
            "status": "indexing_started",
            "services": results
        }))
    }
    
    #[allow(dead_code)]
    pub async fn get_service_statuses(&self) -> HashMap<String, serde_json::Value> {
        let mut statuses = HashMap::new();
        
        for (service_name, service_url) in self.services.iter() {
            let status = match self.client
                .get(&format!("{}/health", service_url))
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        json!({"status": "healthy", "url": service_url})
                    } else {
                        json!({"status": "unhealthy", "url": service_url, "statusCode": response.status().as_u16()})
                    }
                }
                Err(_) => json!({"status": "unavailable", "url": service_url})
            };
            
            statuses.insert(service_name.clone(), status);
        }
        
        statuses
    }

    
    #[allow(dead_code)]
    pub async fn run_pr_review_workflow(&self, pr_url: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        
        
        
        
        
        
        println!("Starting PR review workflow for: {}", pr_url);
        
        
        Ok(json!({
            "status": "workflow_completed",
            "pr_url": pr_url,
            "review": {
                "summary": "AI-generated review summary.",
                "suggestions": [
                    "Consider refactoring the authentication logic.",
                    "Add more unit tests for the new endpoint."
                ]
            }
        }))
    }
}
