use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use dashmap::DashMap;
use url::Url;

use crate::config::IndexerConfig;
use crate::models::*;
use crate::services::chunking::ChunkingService;

pub struct WebIndexingService {
    config: IndexerConfig,
    jobs: Arc<DashMap<String, IndexingJob>>,
    chunking_service: Arc<ChunkingService>,
}

impl WebIndexingService {
    pub fn new(config: IndexerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config: config.clone(),
            jobs: Arc::new(DashMap::new()),
            chunking_service: Arc::new(ChunkingService::new(config)),
        })
    }

    pub async fn index_url(
        &self,
        url: String,
        max_depth: u32,
        metadata: HashMap<String, String>,
    ) -> Result<IndexingJob, Box<dyn std::error::Error>> {
        let mut job = IndexingJob::new(
            SourceType::Url,
            url.clone(),
            metadata,
        );
        
        let job_id = job.id.clone();
        job.start();
        self.jobs.insert(job_id.clone(), job.clone());

        
        let jobs = self.jobs.clone();
        let chunking_service = self.chunking_service.clone();

        tokio::spawn(async move {
            match Self::process_url(&url, max_depth, chunking_service).await {
                Ok((docs, chunks, embeddings)) => {
                    if let Some(mut job_ref) = jobs.get_mut(&job_id) {
                        job_ref.complete(docs, chunks, embeddings);
                    }
                }
                Err(e) => {
                    log::error!("URL indexing failed for {}: {}", job_id, e);
                    if let Some(mut job_ref) = jobs.get_mut(&job_id) {
                        job_ref.fail(e.to_string());
                    }
                }
            }
        });

        Ok(job)
    }

    async fn process_url(
        url: &str,
        max_depth: u32,
        chunking_service: Arc<ChunkingService>,
    ) -> Result<(usize, usize, usize), Box<dyn std::error::Error>> {
        log::info!("Processing URL: {} (max_depth: {})", url, max_depth);

        let mut visited = HashSet::new();
        let mut to_visit = vec![(url.to_string(), 0u32)];
        let mut total_docs = 0;
        let mut total_chunks = 0;

        let client = reqwest::Client::builder()
            .user_agent("ConHub-Indexer/1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        while let Some((current_url, depth)) = to_visit.pop() {
            if visited.contains(&current_url) || depth > max_depth {
                continue;
            }

            visited.insert(current_url.clone());

            match Self::crawl_single_url(&client, &current_url, &chunking_service).await {
                Ok((chunks, links)) => {
                    total_docs += 1;
                    total_chunks += chunks;

                    
                    if depth < max_depth {
                        for link in links {
                            if !visited.contains(&link) {
                                to_visit.push((link, depth + 1));
                            }
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to crawl {}: {}", current_url, e);
                }
            }
        }

        log::info!("URL processing complete: {} documents, {} chunks", total_docs, total_chunks);
        
        Ok((total_docs, total_chunks, total_chunks))
    }

    async fn crawl_single_url(
        client: &reqwest::Client,
        url: &str,
        chunking_service: &ChunkingService,
    ) -> Result<(usize, Vec<String>), Box<dyn std::error::Error>> {
        log::debug!("Crawling: {}", url);

        let response = client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(format!("HTTP {}", response.status()).into());
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("");

        if !content_type.contains("text/html") {
            return Err("Not HTML content".into());
        }

        let html = response.text().await?;
        
        
        let document = scraper::Html::parse_document(&html);
        
        
        let text = Self::extract_text_from_html(&document)?;
        
        
        let chunks = chunking_service.chunk_text(&text)?;
        let chunk_count = chunks.len();

        
        let links = Self::extract_links(&document, url)?;

        Ok((chunk_count, links))
    }

    fn extract_text_from_html(document: &scraper::Html) -> Result<String, Box<dyn std::error::Error>> {
        
        let selector = scraper::Selector::parse("body").unwrap();
        
        let mut text = String::new();
        for element in document.select(&selector) {
            text.push_str(&element.text().collect::<Vec<_>>().join(" "));
        }
        
        
        let cleaned = text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        
        Ok(cleaned)
    }

    fn extract_links(document: &scraper::Html, base_url: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let base = Url::parse(base_url)?;
        let selector = scraper::Selector::parse("a[href]").unwrap();
        
        let mut links = Vec::new();
        
        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                
                if let Ok(absolute_url) = base.join(href) {
                    
                    if absolute_url.scheme() == "http" || absolute_url.scheme() == "https" {
                        if absolute_url.domain() == base.domain() {
                            links.push(absolute_url.to_string());
                        }
                    }
                }
            }
        }
        
        Ok(links)
    }

    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        _offset: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        log::info!("Searching web content: {}", query);
        
        
        let mut results = Vec::new();
        
        if !query.is_empty() {
            results.push(SearchResult {
                id: uuid::Uuid::new_v4().to_string(),
                title: format!("Web search result for: {}", query),
                content: "Sample web content...".to_string(),
                source_type: "url".to_string(),
                source_url: "https://example.com".to_string(),
                score: 0.85,
                metadata: HashMap::new(),
            });
        }
        
        results.truncate(limit);
        Ok(results)
    }

    pub async fn get_stats(&self) -> StatusResponse {
        let jobs: Vec<_> = self.jobs.iter().map(|e| e.value().clone()).collect();
        
        let active = jobs.iter().filter(|j| matches!(j.status, IndexingStatus::InProgress)).count();
        let completed = jobs.iter().filter(|j| matches!(j.status, IndexingStatus::Completed)).count();
        let failed = jobs.iter().filter(|j| matches!(j.status, IndexingStatus::Failed)).count();
        let pending = jobs.iter().filter(|j| matches!(j.status, IndexingStatus::Pending)).count();

        StatusResponse {
            active_jobs: active,
            completed_jobs: completed,
            failed_jobs: failed,
            queue_size: pending,
        }
    }

    pub async fn get_job(&self, job_id: &str) -> Option<IndexingJob> {
        self.jobs.get(job_id).map(|e| e.value().clone())
    }
}
