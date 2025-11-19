use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use tracing::{error, info, warn};

use crate::sources::core::{DataSourceConnector, DataSource, Document, SyncResult};

pub struct UrlConnector {
    client: Client,
}

impl UrlConnector {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("ConHub/1.0")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    async fn crawl_url(&self, url: &str, _allowed_domains: &[String], max_depth: usize, current_depth: usize, visited: &mut HashSet<String>) -> Result<Vec<Document>, Box<dyn std::error::Error + Send + Sync>> {
        if current_depth >= max_depth || visited.contains(url) {
            return Ok(vec![]);
        }

        visited.insert(url.to_string());

        let response = self.client.get(url).send().await?;
        
        if !response.status().is_success() {
            warn!("Failed to fetch URL {}: {}", url, response.status());
            return Ok(vec![]);
        }

        let content = response.text().await?;
        let title = self.extract_title(&content).unwrap_or_else(|| url.to_string());

        Ok(vec![Document {
            id: format!("url-{}", url.replace(['/', ':', '?', '&'], "-")),
            title,
            content,
            metadata: json!({
                "source": "url",
                "url": url,
                "crawl_depth": current_depth
            }),
        }])
    }

    fn extract_title(&self, html: &str) -> Option<String> {
        if let Some(start) = html.find("<title>") {
            if let Some(end) = html[start + 7..].find("</title>") {
                return Some(html[start + 7..start + 7 + end].trim().to_string());
            }
        }
        None
    }
}

#[async_trait]
impl DataSourceConnector for UrlConnector {
    async fn validate(&self, _credentials: &HashMap<String, String>) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(true)
    }

    async fn connect(&mut self, _credentials: &HashMap<String, String>, _config: &Value) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        info!("URL connector ready");
        Ok(true)
    }

    #[allow(dead_code)]
    async fn sync(&self, data_source: &DataSource) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut documents = Vec::new();

        let empty_vec = vec![];
        let urls = data_source.config.get("urls")
            .and_then(|u| u.as_array())
            .unwrap_or(&empty_vec);

        let crawl_depth = data_source.config.get("crawlDepth")
            .and_then(|d| d.as_u64())
            .unwrap_or(1) as usize;

        let allowed_domains: Vec<String> = data_source.config.get("allowedDomains")
            .and_then(|d| d.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        let mut visited = HashSet::new();

        for url_value in urls {
            if let Some(url) = url_value.as_str() {
                match self.crawl_url(url, &allowed_domains, crawl_depth, 0, &mut visited).await {
                    Ok(mut url_documents) => {
                        documents.append(&mut url_documents);
                    }
                    Err(e) => {
                        error!("Failed to crawl URL {}: {}", url, e);
                    }
                }
            }
        }

        Ok(SyncResult { documents, repositories: vec![] })
    }

    async fn fetch_branches(&self, _repo_url: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        Err("URL connector does not support branches".into())
    }
}

impl Default for UrlConnector {
    fn default() -> Self {
        Self::new()
    }
}
