use reqwest::Client;
use crate::models::ServiceStatus;

pub async fn check_service_health(client: &Client, url: &str) -> (String, Option<u64>) {
    let start = std::time::Instant::now();
    
    match client.get(&format!("{}/health", url)).send().await {
        Ok(response) => {
            let duration = start.elapsed().as_millis() as u64;
            if response.status().is_success() {
                ("healthy".to_string(), Some(duration))
            } else {
                ("unhealthy".to_string(), Some(duration))
            }
        }
        Err(_) => ("unavailable".to_string(), None),
    }
}

pub async fn get_all_services_status(
    client: &Client,
    langchain_url: &str,
    haystack_url: &str,
    lexor_url: &str,
) -> Vec<ServiceStatus> {
    let mut services = Vec::new();
    
    let langchain_status = check_service_health(client, langchain_url).await;
    services.push(ServiceStatus {
        name: "LangChain".to_string(),
        url: langchain_url.to_string(),
        status: langchain_status.0,
        response_time_ms: langchain_status.1,
    });
    
    let haystack_status = check_service_health(client, haystack_url).await;
    services.push(ServiceStatus {
        name: "Haystack".to_string(),
        url: haystack_url.to_string(),
        status: haystack_status.0,
        response_time_ms: haystack_status.1,
    });
    
    let lexor_status = check_service_health(client, lexor_url).await;
    services.push(ServiceStatus {
        name: "Lexor".to_string(),
        url: lexor_url.to_string(),
        status: lexor_status.0,
        response_time_ms: lexor_status.1,
    });
    
    services
}