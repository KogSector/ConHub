use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::models::{RepositoryInfo, DataSourceType};
use crate::services::vector_db::VectorDbService;

/// Indexing orchestrator that coordinates between Lexor and AI services
pub struct IndexingOrchestrator {
    client: Client,
    lexor_url: String,
    ai_service_url: String,
    vector_db: Arc<Mutex<VectorDbService>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct IndexingJob {
    pub id: String,
    pub source_id: String,
    pub source_type: DataSourceType,
    pub status: IndexingStatus,
    pub progress: f32,
    pub error: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum IndexingStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct IndexingRequest {
    pub source_id: String,
    pub source_type: DataSourceType,
    pub repository_info: Option<RepositoryInfo>,
    pub content: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl IndexingOrchestrator {
    pub fn new() -> Self {
        let lexor_url = std::env::var("LEXOR_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:3002".to_string());
        let ai_service_url = std::env::var("AI_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8001".to_string());

        Self {
            client: Client::new(),
            lexor_url,
            ai_service_url,
            vector_db: Arc::new(Mutex::new(VectorDbService::new())),
        }
    }

    /// Start indexing process for a data source
    pub async fn start_indexing(&self, request: IndexingRequest) -> Result<IndexingJob, Box<dyn std::error::Error + Send + Sync>> {
        let job_id = uuid::Uuid::new_v4().to_string();
        
        let mut job = IndexingJob {
            id: job_id.clone(),
            source_id: request.source_id.clone(),
            source_type: request.source_type.clone(),
            status: IndexingStatus::Pending,
            progress: 0.0,
            error: None,
            started_at: chrono::Utc::now(),
            completed_at: None,
        };

        // Update job status to in progress
        job.status = IndexingStatus::InProgress;
        job.progress = 10.0;

        match request.source_type {
            DataSourceType::Repository => {
                if let Some(repo_info) = request.repository_info {
                    self.index_repository(&mut job, &repo_info).await?;
                } else {
                    return Err("Repository info required for repository indexing".into());
                }
            }
            DataSourceType::Document => {
                if let Some(content) = request.content {
                    self.index_document(&mut job, &content, &request.metadata).await?;
                } else {
                    return Err("Content required for document indexing".into());
                }
            }
            DataSourceType::Url => {
                self.index_url(&mut job, &request.metadata).await?;
            }
            _ => {
                return Err(format!("Unsupported data source type: {:?}", request.source_type).into());
            }
        }

        job.status = IndexingStatus::Completed;
        job.progress = 100.0;
        job.completed_at = Some(chrono::Utc::now());

        Ok(job)
    }

    /// Index a repository using Lexor service
    async fn index_repository(&self, job: &mut IndexingJob, repo_info: &RepositoryInfo) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Starting repository indexing for: {}", repo_info.name);
        
        // Prepare repository data for Lexor
        let lexor_payload = json!({
            "repository_id": repo_info.id,
            "name": repo_info.name,
            "url": repo_info.url,
            "clone_url": repo_info.clone_url,
            "default_branch": repo_info.default_branch,
            "vcs_type": format!("{:?}", repo_info.vcs_type),
            "provider": format!("{:?}", repo_info.provider),
            "config": {
                "include_code": true,
                "include_readme": true,
                "file_extensions": [".rs", ".py", ".js", ".ts", ".jsx", ".tsx", ".go", ".java", ".md", ".txt", ".json", ".yml", ".yaml"]
            }
        });

        job.progress = 30.0;

        // Call Lexor service to index the repository
        let response = self.client
            .post(&format!("{}/index/repository", self.lexor_url))
            .json(&lexor_payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Lexor indexing failed: {}", error_text).into());
        }

        job.progress = 70.0;

        // Index README and documentation files in vector database for semantic search
        if let Ok(readme_content) = self.extract_readme_content(repo_info).await {
            // Index in local vector database
            let _vector_db = self.vector_db.clone();
            let _repo_id = repo_info.id.clone();
            let _repo_name = repo_info.name.clone();
            let _content = readme_content.clone();
            
            // Note: Vector database indexing will be handled separately

            // Also send to AI service for additional processing
            let ai_payload = json!({
                "content": readme_content,
                "metadata": {
                    "source_type": "repository",
                    "repository_id": repo_info.id,
                    "repository_name": repo_info.name,
                    "file_type": "readme",
                    "url": repo_info.url
                }
            });

            let _ai_response = self.client
                .post(&format!("{}/vector/documents", self.ai_service_url))
                .form(&[
                    ("content", readme_content.as_str()),
                    ("metadata", &serde_json::to_string(&ai_payload["metadata"])?),
                ])
                .send()
                .await;
        }

        job.progress = 90.0;
        println!("Repository indexing completed for: {}", repo_info.name);
        
        Ok(())
    }

    /// Index a document using AI service
    async fn index_document(&self, job: &mut IndexingJob, content: &str, metadata: &HashMap<String, serde_json::Value>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Starting document indexing");
        
        job.progress = 30.0;

        // Index in local vector database
        let _vector_db = self.vector_db.clone();
        let _doc_id = metadata.get("document_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown").to_string();
        let _title = metadata.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled").to_string();
        let _doc_type = metadata.get("document_type")
            .and_then(|v| v.as_str())
            .unwrap_or("document").to_string();
        let _content_clone = content.to_string();
        
        // Note: Vector database indexing will be handled separately

        // Send document to AI service for additional processing
        let response = self.client
            .post(&format!("{}/vector/documents", self.ai_service_url))
            .form(&[
                ("content", content),
                ("metadata", &serde_json::to_string(metadata)?),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("AI service indexing failed: {}", error_text).into());
        }

        job.progress = 90.0;
        println!("Document indexing completed");
        
        Ok(())
    }

    /// Index a URL using AI service
    async fn index_url(&self, job: &mut IndexingJob, metadata: &HashMap<String, serde_json::Value>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Starting URL indexing");
        
        job.progress = 20.0;

        // Extract URL from metadata
        let url = metadata.get("url")
            .and_then(|v| v.as_str())
            .ok_or("URL not found in metadata")?;

        // Crawl the URL content (simplified implementation)
        let content = self.crawl_url(url).await?;
        
        job.progress = 60.0;

        // Send crawled content to AI service
        let response = self.client
            .post(&format!("{}/vector/documents", self.ai_service_url))
            .form(&[
                ("content", content.as_str()),
                ("metadata", &serde_json::to_string(metadata)?),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("AI service indexing failed: {}", error_text).into());
        }

        job.progress = 90.0;
        println!("URL indexing completed for: {}", url);
        
        Ok(())
    }

    /// Extract README content from repository (simplified)
    async fn extract_readme_content(&self, repo_info: &RepositoryInfo) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // This is a simplified implementation
        // In a real implementation, you would use the VCS connector to fetch README files
        Ok(format!("# {}\n\nRepository: {}\nDescription: {}", 
            repo_info.name, 
            repo_info.url,
            repo_info.description.as_deref().unwrap_or("No description available")
        ))
    }

    /// Crawl URL content (simplified)
    async fn crawl_url(&self, url: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.client
            .get(url)
            .header("User-Agent", "ConHub-Indexer/1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to fetch URL: {}", response.status()).into());
        }

        let content = response.text().await?;
        
        // Basic HTML content extraction (you might want to use a proper HTML parser)
        let text_content = if content.contains("<html") {
            // Simple HTML tag removal (use a proper HTML parser in production)
            regex::Regex::new(r"<[^>]*>")
                .unwrap()
                .replace_all(&content, " ")
                .trim()
                .to_string()
        } else {
            content
        };

        Ok(text_content)
    }

    /// Get indexing job status
    #[allow(dead_code)]
    pub async fn get_job_status(&self, _job_id: &str) -> Option<IndexingJob> {
        // In a real implementation, you would store job status in a database
        // For now, this is a placeholder
        None
    }

    /// List all indexing jobs
    #[allow(dead_code)]
    pub async fn list_jobs(&self) -> Vec<IndexingJob> {
        // In a real implementation, you would retrieve jobs from a database
        // For now, return empty list
        vec![]
    }
}

impl Default for IndexingOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}