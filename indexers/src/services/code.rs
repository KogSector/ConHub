use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::RwLock;

use crate::config::IndexerConfig;
use crate::models::*;
use crate::services::chunking::ChunkingService;

pub struct CodeIndexingService {
    config: IndexerConfig,
    jobs: Arc<DashMap<String, IndexingJob>>,
    chunking_service: Arc<ChunkingService>,
}

impl CodeIndexingService {
    pub fn new(config: IndexerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config: config.clone(),
            jobs: Arc::new(DashMap::new()),
            chunking_service: Arc::new(ChunkingService::new(config)),
        })
    }

    pub async fn index_repository(
        &self,
        repository_url: String,
        branch: String,
        metadata: HashMap<String, String>,
    ) -> Result<IndexingJob, Box<dyn std::error::Error>> {
        let mut job = IndexingJob::new(
            SourceType::Repository,
            repository_url.clone(),
            metadata.clone(),
        );
        
        let job_id = job.id.clone();
        job.start();
        self.jobs.insert(job_id.clone(), job.clone());

        
        let jobs = self.jobs.clone();
        let chunking_service = self.chunking_service.clone();
        let temp_dir = self.config.temp_dir.clone();

        tokio::spawn(async move {
            match Self::process_repository(&repository_url, &branch, &temp_dir, chunking_service).await {
                Ok((docs, chunks, embeddings)) => {
                    if let Some(mut job_ref) = jobs.get_mut(&job_id) {
                        job_ref.complete(docs, chunks, embeddings);
                    }
                }
                Err(e) => {
                    log::error!("Repository indexing failed for {}: {}", job_id, e);
                    if let Some(mut job_ref) = jobs.get_mut(&job_id) {
                        job_ref.fail(e.to_string());
                    }
                }
            }
        });

        Ok(job)
    }

    async fn process_repository(
        repo_url: &str,
        branch: &str,
        temp_dir: &str,
        chunking_service: Arc<ChunkingService>,
    ) -> Result<(usize, usize, usize), Box<dyn std::error::Error>> {
        log::info!("Processing repository: {} (branch: {})", repo_url, branch);

        
        let repo_dir = format!("{}/{}", temp_dir, uuid::Uuid::new_v4());
        std::fs::create_dir_all(&repo_dir)?;

        
        log::info!("Cloning repository to: {}", repo_dir);
        let repo = git2::Repository::clone(repo_url, &repo_dir)?;
        
        
        let (object, reference) = repo.revparse_ext(branch)?;
        repo.checkout_tree(&object, None)?;
        
        if let Some(reference) = reference {
            repo.set_head(reference.name().unwrap())?;
        }

        
        let mut documents_processed = 0;
        let mut total_chunks = 0;

        for entry in walkdir::WalkDir::new(&repo_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            
            
            if path.to_str().unwrap_or("").contains("/.") {
                continue;
            }

            
            if let Some(extension) = path.extension() {
                let ext = extension.to_str().unwrap_or("");
                if Self::is_code_file(ext) {
                    match Self::index_code_file(path, &chunking_service).await {
                        Ok(chunks) => {
                            documents_processed += 1;
                            total_chunks += chunks;
                        }
                        Err(e) => {
                            log::warn!("Failed to index file {:?}: {}", path, e);
                        }
                    }
                }
            }
        }

        
        let _ = std::fs::remove_dir_all(&repo_dir);

        log::info!("Repository processing complete: {} files, {} chunks", documents_processed, total_chunks);
        
        
        Ok((documents_processed, total_chunks, total_chunks))
    }

    async fn index_code_file(
        path: &Path,
        chunking_service: &ChunkingService,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        
        
        let chunks = chunking_service.chunk_text(&content)?;
        
        
        
        log::debug!("Indexed file {:?}: {} chunks", path, chunks.len());
        
        Ok(chunks.len())
    }

    fn is_code_file(extension: &str) -> bool {
        matches!(
            extension.to_lowercase().as_str(),
            "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "java" | "c" | "cpp" | "h" | "hpp"
                | "go" | "rb" | "php" | "cs" | "swift" | "kt" | "scala" | "sh" | "bash"
                | "sql" | "yaml" | "yml" | "toml" | "json" | "xml" | "html" | "css" | "scss"
        )
    }

    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        _offset: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        log::info!("Searching code: {}", query);
        
        
        
        let mut results = Vec::new();
        
        
        if !query.is_empty() {
            results.push(SearchResult {
                id: uuid::Uuid::new_v4().to_string(),
                title: format!("Code search result for: {}", query),
                content: "Sample code content...".to_string(),
                source_type: "code".to_string(),
                source_url: "https://github.com/example/repo".to_string(),
                score: 0.95,
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
