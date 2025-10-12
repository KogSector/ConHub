use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use dashmap::DashMap;

use crate::config::IndexerConfig;
use crate::models::*;
use crate::services::chunking::ChunkingService;

pub struct DocumentIndexingService {
    config: IndexerConfig,
    jobs: Arc<DashMap<String, IndexingJob>>,
    chunking_service: Arc<ChunkingService>,
}

impl DocumentIndexingService {
    pub fn new(config: IndexerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config: config.clone(),
            jobs: Arc::new(DashMap::new()),
            chunking_service: Arc::new(ChunkingService::new(config)),
        })
    }

    pub async fn index_documentation(
        &self,
        doc_url: String,
        crawl_depth: u32,
        metadata: HashMap<String, String>,
    ) -> Result<IndexingJob, Box<dyn std::error::Error>> {
        let mut job = IndexingJob::new(
            SourceType::Documentation,
            doc_url.clone(),
            metadata,
        );
        
        let job_id = job.id.clone();
        job.start();
        self.jobs.insert(job_id.clone(), job.clone());

        // Spawn background task
        let jobs = self.jobs.clone();
        let chunking_service = self.chunking_service.clone();

        tokio::spawn(async move {
            match Self::process_documentation(&doc_url, crawl_depth, chunking_service).await {
                Ok((docs, chunks, embeddings)) => {
                    if let Some(mut job_ref) = jobs.get_mut(&job_id) {
                        job_ref.complete(docs, chunks, embeddings);
                    }
                }
                Err(e) => {
                    log::error!("Documentation indexing failed for {}: {}", job_id, e);
                    if let Some(mut job_ref) = jobs.get_mut(&job_id) {
                        job_ref.fail(e.to_string());
                    }
                }
            }
        });

        Ok(job)
    }

    pub async fn index_file(
        &self,
        file_path: String,
        metadata: HashMap<String, String>,
    ) -> Result<IndexingJob, Box<dyn std::error::Error>> {
        let mut job = IndexingJob::new(
            SourceType::File,
            file_path.clone(),
            metadata,
        );
        
        let job_id = job.id.clone();
        job.start();
        self.jobs.insert(job_id.clone(), job.clone());

        // Spawn background task
        let jobs = self.jobs.clone();
        let chunking_service = self.chunking_service.clone();

        tokio::spawn(async move {
            match Self::process_file(&file_path, chunking_service).await {
                Ok((docs, chunks, embeddings)) => {
                    if let Some(mut job_ref) = jobs.get_mut(&job_id) {
                        job_ref.complete(docs, chunks, embeddings);
                    }
                }
                Err(e) => {
                    log::error!("File indexing failed for {}: {}", job_id, e);
                    if let Some(mut job_ref) = jobs.get_mut(&job_id) {
                        job_ref.fail(e.to_string());
                    }
                }
            }
        });

        Ok(job)
    }

    async fn process_documentation(
        doc_url: &str,
        _crawl_depth: u32,
        chunking_service: Arc<ChunkingService>,
    ) -> Result<(usize, usize, usize), Box<dyn std::error::Error>> {
        log::info!("Processing documentation: {}", doc_url);

        // Fetch documentation content
        let client = reqwest::Client::new();
        let response = client
            .get(doc_url)
            .header("User-Agent", "ConHub-Indexer/1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to fetch documentation: HTTP {}", response.status()).into());
        }

        let content = response.text().await?;
        
        // Extract text from HTML
        let text = Self::extract_text_from_html(&content)?;
        
        // Chunk the content
        let chunks = chunking_service.chunk_text(&text)?;
        let chunk_count = chunks.len();

        log::info!("Documentation processing complete: 1 document, {} chunks", chunk_count);
        
        Ok((1, chunk_count, chunk_count))
    }

    async fn process_file(
        file_path: &str,
        chunking_service: Arc<ChunkingService>,
    ) -> Result<(usize, usize, usize), Box<dyn std::error::Error>> {
        log::info!("Processing file: {}", file_path);

        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path).into());
        }

        // Read file content
        let content = tokio::fs::read_to_string(path).await?;
        
        // Chunk the content
        let chunks = chunking_service.chunk_text(&content)?;
        let chunk_count = chunks.len();

        log::info!("File processing complete: 1 document, {} chunks", chunk_count);
        
        Ok((1, chunk_count, chunk_count))
    }

    fn extract_text_from_html(html: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Use scraper to parse HTML
        let document = scraper::Html::parse_document(html);
        
        // Remove script and style tags
        let selector = scraper::Selector::parse("body").unwrap();
        
        let mut text = String::new();
        for element in document.select(&selector) {
            text.push_str(&element.text().collect::<Vec<_>>().join(" "));
        }
        
        // Clean up whitespace
        let cleaned = text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        
        Ok(cleaned)
    }

    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        _offset: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        log::info!("Searching documents: {}", query);
        
        // Placeholder implementation
        let mut results = Vec::new();
        
        if !query.is_empty() {
            results.push(SearchResult {
                id: uuid::Uuid::new_v4().to_string(),
                title: format!("Document search result for: {}", query),
                content: "Sample document content...".to_string(),
                source_type: "documentation".to_string(),
                source_url: "https://docs.example.com".to_string(),
                score: 0.90,
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
