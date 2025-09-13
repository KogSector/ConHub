use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;

pub struct ServiceOrchestrator {
    client: Client,
    services: HashMap<String, String>,
}

impl ServiceOrchestrator {
    pub fn new(langchain_url: String, haystack_url: String, lexor_url: String) -> Self {
        let mut services = HashMap::new();
        services.insert("langchain".to_string(), langchain_url);
        services.insert("haystack".to_string(), haystack_url);
        services.insert("lexor".to_string(), lexor_url);
        
        Self {
            client: Client::new(),
            services,
        }
    }
    
    pub async fn start_full_indexing(&self, repo_url: &str, access_token: Option<&str>) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let mut results = HashMap::new();
        
        // Start LangChain indexing
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
        
        // Start Lexor indexing
        if let Some(lexor_url) = self.services.get("lexor") {
            let payload = json!({
                "repository_url": repo_url,
                "access_token": access_token
            });
            
            if let Ok(response) = self.client
                .post(&format!("{}/api/projects", lexor_url))
                .json(&payload)
                .send()
                .await
            {
                if let Ok(result) = response.json::<serde_json::Value>().await {
                    results.insert("lexor", result);
                }
            }
        }
        
        Ok(json!({
            "status": "indexing_started",
            "services": results
        }))
    }
    
    pub async fn get_service_statuses(&self) -> HashMap<String, serde_json::Value> {
        let mut statuses = HashMap::new();
        
        for (service_name, service_url) in &self.services {
            let status = match self.client
                .get(&format!("{}/health", service_url))
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        json!({"status": "healthy", "url": service_url})
                    } else {
                        json!({"status": "unhealthy", "url": service_url})
                    }
                }
                Err(_) => json!({"status": "unavailable", "url": service_url})
            };
            
            statuses.insert(service_name.clone(), status);
        }
        
        statuses
    }
}