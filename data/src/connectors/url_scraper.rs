use async_trait::async_trait;
use uuid::Uuid;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};
use std::collections::HashMap;
use url::Url;
use scraper::{Html, Selector};

use super::traits::{Connector, ConnectorFactory};
use super::types::*;
use super::error::ConnectorError;

/// URL scraper connector for crawling web content
pub struct UrlScraperConnector {
    name: String,
    client: Client,
}

#[derive(Debug, Serialize, Deserialize)]
struct UrlConfig {
    urls: Vec<String>,
    follow_links: bool,
    max_depth: usize,
    respect_robots_txt: bool,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
}

impl Default for UrlConfig {
    fn default() -> Self {
        Self {
            urls: Vec::new(),
            follow_links: false,
            max_depth: 1,
            respect_robots_txt: true,
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
        }
    }
}

impl UrlScraperConnector {
    pub fn new() -> Self {
        Self {
            name: "URL Scraper".to_string(),
            client: Client::builder()
                .user_agent("ConHub/1.0 (+https://conhub.ai)")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }
    
    pub fn factory() -> UrlScraperConnectorFactory {
        UrlScraperConnectorFactory
    }
    
    fn parse_config(&self, account: &ConnectedAccount) -> UrlConfig {
        account.metadata
            .as_ref()
            .and_then(|m| serde_json::from_value(m.clone()).ok())
            .unwrap_or_default()
    }
    
    async fn fetch_url_content(&self, url: &str) -> Result<(String, String), ConnectorError> {
        info!("🌐 Fetching content from: {}", url);
        
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| ConnectorError::HttpError(format!("Failed to fetch {}: {}", url, e)))?;
        
        if !response.status().is_success() {
            return Err(ConnectorError::HttpError(
                format!("HTTP {} for {}", response.status(), url)
            ));
        }
        
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("text/html")
            .to_string();
        
        let html = response.text().await
            .map_err(|e| ConnectorError::HttpError(format!("Failed to read response: {}", e)))?;
        
        Ok((html, content_type))
    }
    
    fn extract_text_from_html(&self, html: &str) -> Result<String, ConnectorError> {
        let document = Html::parse_document(html);
        
        // Remove script and style elements
        let script_selector = Selector::parse("script, style, nav, footer, aside").unwrap();
        let mut clean_html = html.to_string();
        
        for element in document.select(&script_selector) {
            if let Some(html_content) = element.html().get(0..std::cmp::min(element.html().len(), 1000)) {
                clean_html = clean_html.replace(html_content, "");
            }
        }
        
        // Parse cleaned HTML
        let clean_document = Html::parse_document(&clean_html);
        
        // Extract text from main content areas
        let content_selectors = [
            "main", "article", ".content", "#content", ".main", "#main",
            "body", // fallback
        ];
        
        for selector_str in &content_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = clean_document.select(&selector).next() {
                    let text = element.text().collect::<Vec<_>>().join(" ");
                    let cleaned_text = text
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ")
                        .trim()
                        .to_string();
                    
                    if !cleaned_text.is_empty() && cleaned_text.len() > 100 {
                        return Ok(cleaned_text);
                    }
                }
            }
        }
        
        // Fallback: extract all text
        let all_text = document.root_element()
            .text()
            .collect::<Vec<_>>()
            .join(" ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        
        Ok(all_text)
    }
    
    fn extract_links_from_html(&self, html: &str, base_url: &Url) -> Vec<String> {
        let document = Html::parse_document(html);
        let link_selector = Selector::parse("a[href]").unwrap();
        
        let mut links = Vec::new();
        
        for element in document.select(&link_selector) {
            if let Some(href) = element.value().attr("href") {
                if let Ok(absolute_url) = base_url.join(href) {
                    let url_str = absolute_url.to_string();
                    // Only include HTTP/HTTPS links
                    if url_str.starts_with("http://") || url_str.starts_with("https://") {
                        links.push(url_str);
                    }
                }
            }
        }
        
        links.sort();
        links.dedup();
        links
    }
    
    fn crawl_url<'a>(
        &'a self,
        url: &'a str,
        config: &'a UrlConfig,
        depth: usize,
        visited: &'a mut std::collections::HashSet<String>,
        documents: &'a mut Vec<DocumentMetadata>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ConnectorError>> + Send + 'a>> {
        Box::pin(async move {
            if depth > config.max_depth || visited.contains(url) {
                return Ok(());
            }
            
            visited.insert(url.to_string());
            
            // Apply include/exclude patterns
            if !config.include_patterns.is_empty() {
                if !config.include_patterns.iter().any(|pattern| url.contains(pattern)) {
                    return Ok(());
                }
            }
            
            if config.exclude_patterns.iter().any(|pattern| url.contains(pattern)) {
                return Ok(());
            }
        
        match self.fetch_url_content(url).await {
            Ok((html, content_type)) => {
                let text_content = if content_type.contains("text/html") {
                    self.extract_text_from_html(&html)?
                } else {
                    html.clone()
                };
                
                // Extract title from HTML if available
                let title = if content_type.contains("text/html") {
                    let document = Html::parse_document(&html);
                    let title_selector = Selector::parse("title").unwrap();
                    document.select(&title_selector)
                        .next()
                        .map(|el| el.text().collect::<String>())
                        .filter(|t| !t.trim().is_empty())
                        .unwrap_or_else(|| url.to_string())
                } else {
                    url.to_string()
                };
                
                documents.push(DocumentMetadata {
                    external_id: url.to_string(),
                    name: title,
                    path: Some(url.to_string()),
                    mime_type: Some(content_type.clone()),
                    size: Some(text_content.len() as i64),
                    created_at: Some(chrono::Utc::now()),
                    modified_at: Some(chrono::Utc::now()),
                    permissions: None,
                    url: Some(url.to_string()),
                    parent_id: None,
                    is_folder: false,
                    metadata: Some(serde_json::json!({
                        "content_type": content_type,
                        "crawl_depth": depth,
                    })),
                });
                
                // Follow links if enabled and we haven't reached max depth
                if config.follow_links && depth < config.max_depth {
                    if let Ok(base_url) = Url::parse(url) {
                        let links = self.extract_links_from_html(&html, &base_url);
                        
                        for link in links.iter().take(50) { // Limit to prevent infinite crawling
                            if let Err(e) = self.crawl_url(link, config, depth + 1, visited, documents).await {
                                warn!("Failed to crawl {}: {}", link, e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to fetch {}: {}", url, e);
            }
        }
        
        Ok(())
        })
    }
    
    fn chunk_content(&self, content: &str, url: &str) -> Vec<DocumentChunk> {
        const CHUNK_SIZE: usize = 1000;
        const CHUNK_OVERLAP: usize = 200;
        
        let mut chunks = Vec::new();
        let content_len = content.len();
        let mut chunk_number = 0;
        let mut start = 0;
        
        while start < content_len {
            let end = (start + CHUNK_SIZE).min(content_len);
            let chunk_content = &content[start..end];
            
            chunks.push(DocumentChunk {
                chunk_number,
                content: chunk_content.to_string(),
                start_offset: start,
                end_offset: end,
                metadata: Some(serde_json::json!({
                    "url": url,
                    "length": chunk_content.len(),
                })),
            });
            
            chunk_number += 1;
            start = end.saturating_sub(CHUNK_OVERLAP);
            
            if start + CHUNK_SIZE >= content_len && end == content_len {
                break;
            }
        }
        
        chunks
    }
}

#[async_trait]
impl Connector for UrlScraperConnector {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn connector_type(&self) -> ConnectorType {
        ConnectorType::UrlScraper
    }
    
    fn validate_config(&self, config: &ConnectorConfigAuth) -> Result<(), ConnectorError> {
        let urls = config.settings.get("urls")
            .and_then(|u| u.as_array())
            .ok_or_else(|| ConnectorError::InvalidConfiguration(
                "URLs list is required".to_string()
            ))?;
        
        if urls.is_empty() {
            return Err(ConnectorError::InvalidConfiguration(
                "At least one URL must be provided".to_string()
            ));
        }
        
        // Validate URLs
        for url_value in urls {
            if let Some(url_str) = url_value.as_str() {
                Url::parse(url_str)
                    .map_err(|_| ConnectorError::InvalidConfiguration(
                        format!("Invalid URL: {}", url_str)
                    ))?;
            }
        }
        
        Ok(())
    }
    
    async fn authenticate(&self, _config: &ConnectorConfigAuth) -> Result<Option<String>, ConnectorError> {
        // URL scraper doesn't need authentication
        Ok(None)
    }
    
    async fn complete_oauth(&self, _callback_data: OAuthCallbackData) -> Result<OAuthCredentials, ConnectorError> {
        Err(ConnectorError::UnsupportedOperation(
            "URL scraper does not support OAuth".to_string()
        ))
    }
    
    async fn connect(&mut self, _account: &ConnectedAccount) -> Result<(), ConnectorError> {
        info!("🌐 Connected to URL scraper");
        Ok(())
    }
    
    async fn check_connection(&self, _account: &ConnectedAccount) -> Result<bool, ConnectorError> {
        // Always available
        Ok(true)
    }
    
    async fn list_documents(
        &self,
        account: &ConnectedAccount,
        _filters: Option<SyncFilters>,
    ) -> Result<Vec<DocumentMetadata>, ConnectorError> {
        let config = self.parse_config(account);
        let mut documents = Vec::new();
        let mut visited = std::collections::HashSet::new();
        
        for url in &config.urls {
            if let Err(e) = self.crawl_url(url, &config, 0, &mut visited, &mut documents).await {
                error!("Failed to crawl {}: {}", url, e);
            }
        }
        
        Ok(documents)
    }
    
    async fn get_document_content(
        &self,
        _account: &ConnectedAccount,
        document_id: &str,
    ) -> Result<DocumentContent, ConnectorError> {
        let (html, content_type) = self.fetch_url_content(document_id).await?;
        
        let text_content = if content_type.contains("text/html") {
            self.extract_text_from_html(&html)?
        } else {
            html.clone()
        };
        
        let title = if content_type.contains("text/html") {
            let document = Html::parse_document(&html);
            let title_selector = Selector::parse("title").unwrap();
            document.select(&title_selector)
                .next()
                .map(|el| el.text().collect::<String>())
                .filter(|t| !t.trim().is_empty())
                .unwrap_or_else(|| document_id.to_string())
        } else {
            document_id.to_string()
        };
        
        Ok(DocumentContent {
            metadata: DocumentMetadata {
                external_id: document_id.to_string(),
                name: title,
                path: Some(document_id.to_string()),
                mime_type: Some(content_type.clone()),
                size: Some(text_content.len() as i64),
                created_at: Some(chrono::Utc::now()),
                modified_at: Some(chrono::Utc::now()),
                permissions: None,
                url: Some(document_id.to_string()),
                parent_id: None,
                is_folder: false,
                metadata: Some(serde_json::json!({
                    "content_type": content_type,
                })),
            },
            content: text_content.into_bytes(),
            content_type: ContentType::Html,
        })
    }
    
    async fn sync(
        &self,
        account: &ConnectedAccount,
        request: &SyncRequestWithFilters,
    ) -> Result<(SyncResult, Vec<DocumentForEmbedding>), ConnectorError> {
        let start_time = std::time::Instant::now();
        
        info!("🔄 Starting URL scraper sync for account: {}", account.account_name);
        
        let documents = self.list_documents(account, request.filters.clone()).await?;
        
        let mut documents_for_embedding = Vec::new();
        let mut errors = Vec::new();
        
        for doc in &documents {
            match self.get_document_content(account, &doc.external_id).await {
                Ok(content) => {
                    let content_str = String::from_utf8_lossy(&content.content).to_string();
                    let chunks = self.chunk_content(&content_str, &doc.external_id);
                    
                    documents_for_embedding.push(DocumentForEmbedding {
                        id: Uuid::new_v4(),
                        source_id: account.id,
                        connector_type: ConnectorType::UrlScraper,
                        external_id: doc.external_id.clone(),
                        name: doc.name.clone(),
                        path: doc.path.clone(),
                        content: content_str,
                        content_type: ContentType::Html,
                        metadata: serde_json::json!({
                            "url": doc.url,
                            "size": doc.size,
                            "mime_type": doc.mime_type,
                        }),
                        chunks: Some(chunks),
                    });
                }
                Err(e) => {
                    error!("Failed to get content for {}: {}", doc.name, e);
                    errors.push(format!("Failed to get {}: {}", doc.name, e));
                }
            }
        }
        
        let sync_duration = start_time.elapsed().as_millis() as u64;
        
        let result = SyncResult {
            total_documents: documents.len(),
            new_documents: documents_for_embedding.len(),
            updated_documents: 0,
            deleted_documents: 0,
            failed_documents: errors.len(),
            sync_duration_ms: sync_duration,
            errors,
        };
        
        info!("✅ URL scraper sync completed: {:?}", result);
        
        Ok((result, documents_for_embedding))
    }
    
    async fn incremental_sync(
        &self,
        account: &ConnectedAccount,
        _since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<DocumentMetadata>, ConnectorError> {
        // For URLs, we always do a full sync since content can change
        self.list_documents(account, None).await
    }
    
    async fn disconnect(&mut self, _account: &ConnectedAccount) -> Result<(), ConnectorError> {
        info!("🌐 Disconnected from URL scraper");
        Ok(())
    }
    
    async fn refresh_credentials(&self, _account: &ConnectedAccount) -> Result<OAuthCredentials, ConnectorError> {
        Err(ConnectorError::UnsupportedOperation(
            "URL scraper does not use credentials".to_string()
        ))
    }
}

pub struct UrlScraperConnectorFactory;

impl ConnectorFactory for UrlScraperConnectorFactory {
    fn create(&self) -> Box<dyn Connector> {
        Box::new(UrlScraperConnector::new())
    }
    
    fn connector_type(&self) -> ConnectorType {
        ConnectorType::UrlScraper
    }
    
    fn supports_oauth(&self) -> bool {
        false
    }
    
    fn supports_webhooks(&self) -> bool {
        false
    }
}
