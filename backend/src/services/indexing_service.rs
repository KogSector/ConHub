use std::error::Error;
use std::collections::HashMap;
use std::path::Path;
use log::{info, error, warn, debug};
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tokio::fs;
use url::Url;

use crate::services::vector_db;
use crate::services::feature_toggle_service::FeatureToggleService;

#[derive(Debug, Clone)]
pub struct IndexingRequest {
    pub id: String,
    pub source_type: IndexingSourceType,
    pub source_url: String,
    pub metadata: HashMap<String, String>,
    pub priority: IndexingPriority,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum IndexingSourceType {
    Repository,
    Documentation,
    Url,
    File,
}

#[derive(Debug, Clone)]
pub enum IndexingPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct IndexingResult {
    pub request_id: String,
    pub status: IndexingStatus,
    pub documents_processed: usize,
    pub chunks_created: usize,
    pub embeddings_generated: usize,
    pub error_message: Option<String>,
    pub processing_time_ms: u64,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum IndexingStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

pub struct IndexingService {
    client: Client,
    lexor_url: String,
    doc_search_url: String,
    vector_db_service: vector_db::VectorDbService,
}

impl IndexingService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            lexor_url: std::env::var("LEXOR_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            doc_search_url: std::env::var("DOC_SEARCH_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:8000".to_string()),
            vector_db_service: vector_db::VectorDbService::new(),
        }
    }

    /// Main entry point for indexing various types of content
    pub async fn index_content(&self, request: IndexingRequest) -> Result<IndexingResult, Box<dyn Error>> {
        let start_time = std::time::Instant::now();
        info!("Starting indexing for request: {}", request.id);

        // Check if heavy operations are enabled
        let feature_service = FeatureToggleService::new("feature-toggles.json");
        if !feature_service.is_heavy_enabled().await {
            warn!("Heavy operations disabled - skipping indexing for request: {}", request.id);
            return Ok(IndexingResult {
                request_id: request.id,
                status: IndexingStatus::Cancelled,
                documents_processed: 0,
                chunks_created: 0,
                embeddings_generated: 0,
                error_message: Some("Heavy operations disabled".to_string()),
                processing_time_ms: start_time.elapsed().as_millis() as u64,
                completed_at: Utc::now(),
            });
        }

        let result = match request.source_type {
            IndexingSourceType::Repository => self.index_repository(&request).await,
            IndexingSourceType::Documentation => self.index_documentation(&request).await,
            IndexingSourceType::Url => self.index_url(&request).await,
            IndexingSourceType::File => self.index_file(&request).await,
        };

        match result {
            Ok(mut indexing_result) => {
                indexing_result.processing_time_ms = start_time.elapsed().as_millis() as u64;
                indexing_result.completed_at = Utc::now();
                info!("Indexing completed for request: {} in {}ms", 
                      request.id, indexing_result.processing_time_ms);
                Ok(indexing_result)
            }
            Err(e) => {
                error!("Indexing failed for request: {}: {}", request.id, e);
                Ok(IndexingResult {
                    request_id: request.id,
                    status: IndexingStatus::Failed,
                    documents_processed: 0,
                    chunks_created: 0,
                    embeddings_generated: 0,
                    error_message: Some(e.to_string()),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    completed_at: Utc::now(),
                })
            }
        }
    }

    /// Index a code repository
    async fn index_repository(&self, request: &IndexingRequest) -> Result<IndexingResult, Box<dyn Error>> {
        info!("Indexing repository: {}", request.source_url);

        // Call Lexor service to index the repository
        let lexor_request = json!({
            "repository_url": request.source_url,
            "branch": request.metadata.get("branch").unwrap_or(&"main".to_string()),
            "include_patterns": request.metadata.get("include_patterns"),
            "exclude_patterns": request.metadata.get("exclude_patterns"),
            "language_filters": request.metadata.get("language_filters"),
            "max_file_size": request.metadata.get("max_file_size").unwrap_or(&"1048576".to_string()).parse::<usize>().unwrap_or(1048576),
        });

        let response = self.client
            .post(&format!("{}/api/index/repository", self.lexor_url))
            .json(&lexor_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Lexor indexing failed: {}", error_text).into());
        }

        let lexor_result: serde_json::Value = response.json().await?;
        
        Ok(IndexingResult {
            request_id: request.id.clone(),
            status: IndexingStatus::Completed,
            documents_processed: lexor_result["documents_processed"].as_u64().unwrap_or(0) as usize,
            chunks_created: lexor_result["chunks_created"].as_u64().unwrap_or(0) as usize,
            embeddings_generated: lexor_result["embeddings_generated"].as_u64().unwrap_or(0) as usize,
            error_message: None,
            processing_time_ms: 0, // Will be set by caller
            completed_at: Utc::now(),
        })
    }

    /// Index documentation
    async fn index_documentation(&self, request: &IndexingRequest) -> Result<IndexingResult, Box<dyn Error>> {
        info!("Indexing documentation: {}", request.source_url);

        // Call doc-search service to index documentation
        let doc_request = json!({
            "source_url": request.source_url,
            "doc_type": request.metadata.get("doc_type").unwrap_or(&"markdown".to_string()),
            "crawl_depth": request.metadata.get("crawl_depth").unwrap_or(&"3".to_string()).parse::<u32>().unwrap_or(3),
            "follow_links": request.metadata.get("follow_links").unwrap_or(&"true".to_string()).parse::<bool>().unwrap_or(true),
            "extract_code_blocks": request.metadata.get("extract_code_blocks").unwrap_or(&"true".to_string()).parse::<bool>().unwrap_or(true),
        });

        let response = self.client
            .post(&format!("{}/api/documents/index", self.doc_search_url))
            .json(&doc_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Documentation indexing failed: {}", error_text).into());
        }

        let doc_result: serde_json::Value = response.json().await?;
        
        Ok(IndexingResult {
            request_id: request.id.clone(),
            status: IndexingStatus::Completed,
            documents_processed: doc_result["documents_processed"].as_u64().unwrap_or(0) as usize,
            chunks_created: doc_result["chunks_created"].as_u64().unwrap_or(0) as usize,
            embeddings_generated: doc_result["embeddings_generated"].as_u64().unwrap_or(0) as usize,
            error_message: None,
            processing_time_ms: 0,
            completed_at: Utc::now(),
        })
    }

    /// Index a single URL
    async fn index_url(&self, request: &IndexingRequest) -> Result<IndexingResult, Box<dyn Error>> {
        info!("Indexing URL: {}", request.source_url);

        // Validate URL
        let url = Url::parse(&request.source_url)?;
        
        // Fetch content from URL
        let response = self.client
            .get(url.as_str())
            .header("User-Agent", "ConHub-Indexer/1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to fetch URL: HTTP {}", response.status()).into());
        }

        let content = response.text().await?;
        let content_type = response.headers()
            .get("content-type")
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("text/html");

        // Process content based on type
        let processed_content = self.process_url_content(&content, content_type, &url).await?;

        // Index the processed content
        let chunks = self.chunk_content(&processed_content).await?;
        let embeddings_count = self.generate_and_store_embeddings(&chunks, &request.id).await?;

        Ok(IndexingResult {
            request_id: request.id.clone(),
            status: IndexingStatus::Completed,
            documents_processed: 1,
            chunks_created: chunks.len(),
            embeddings_generated: embeddings_count,
            error_message: None,
            processing_time_ms: 0,
            completed_at: Utc::now(),
        })
    }

    /// Index a local file
    async fn index_file(&self, request: &IndexingRequest) -> Result<IndexingResult, Box<dyn Error>> {
        info!("Indexing file: {}", request.source_url);

        let file_path = Path::new(&request.source_url);
        if !file_path.exists() {
            return Err(format!("File not found: {}", request.source_url).into());
        }

        // Read file content
        let content = fs::read_to_string(file_path).await?;
        
        // Determine file type from extension
        let file_extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("txt");

        // Process content based on file type
        let processed_content = self.process_file_content(&content, file_extension).await?;

        // Index the processed content
        let chunks = self.chunk_content(&processed_content).await?;
        let embeddings_count = self.generate_and_store_embeddings(&chunks, &request.id).await?;

        Ok(IndexingResult {
            request_id: request.id.clone(),
            status: IndexingStatus::Completed,
            documents_processed: 1,
            chunks_created: chunks.len(),
            embeddings_generated: embeddings_count,
            error_message: None,
            processing_time_ms: 0,
            completed_at: Utc::now(),
        })
    }

    /// Process URL content based on content type
    async fn process_url_content(&self, content: &str, content_type: &str, url: &Url) -> Result<String, Box<dyn Error>> {
        match content_type {
            ct if ct.contains("text/html") => {
                // Extract text from HTML
                self.extract_text_from_html(content).await
            }
            ct if ct.contains("text/markdown") || ct.contains("text/x-markdown") => {
                // Process markdown
                Ok(content.to_string())
            }
            ct if ct.contains("application/json") => {
                // Process JSON content
                self.process_json_content(content).await
            }
            ct if ct.contains("text/plain") => {
                Ok(content.to_string())
            }
            _ => {
                warn!("Unsupported content type: {}", content_type);
                Ok(content.to_string())
            }
        }
    }

    /// Process file content based on extension
    async fn process_file_content(&self, content: &str, extension: &str) -> Result<String, Box<dyn Error>> {
        match extension.to_lowercase().as_str() {
            "md" | "markdown" => Ok(content.to_string()),
            "txt" | "text" => Ok(content.to_string()),
            "json" => self.process_json_content(content).await,
            "html" | "htm" => self.extract_text_from_html(content).await,
            "rs" | "py" | "js" | "ts" | "java" | "cpp" | "c" | "go" => {
                // Code files - preserve structure
                Ok(format!("```{}\n{}\n```", extension, content))
            }
            _ => {
                debug!("Processing unknown file type: {}", extension);
                Ok(content.to_string())
            }
        }
    }

    /// Extract text from HTML content
    async fn extract_text_from_html(&self, html: &str) -> Result<String, Box<dyn Error>> {
        // Simple HTML text extraction (in production, use a proper HTML parser)
        let text = html
            .replace("<script", "<SCRIPT_REMOVED")
            .replace("<style", "<STYLE_REMOVED")
            .replace("</script>", "")
            .replace("</style>", "");
        
        // Remove HTML tags (basic implementation)
        let re = regex::Regex::new(r"<[^>]*>")?;
        let clean_text = re.replace_all(&text, " ");
        
        // Clean up whitespace
        let re_whitespace = regex::Regex::new(r"\s+")?;
        let final_text = re_whitespace.replace_all(&clean_text, " ");
        
        Ok(final_text.trim().to_string())
    }

    /// Process JSON content
    async fn process_json_content(&self, json_str: &str) -> Result<String, Box<dyn Error>> {
        match serde_json::from_str::<serde_json::Value>(json_str) {
            Ok(json_value) => {
                // Convert JSON to readable text format
                Ok(format!("JSON Content:\n{}", serde_json::to_string_pretty(&json_value)?))
            }
            Err(_) => {
                // If not valid JSON, treat as plain text
                Ok(json_str.to_string())
            }
        }
    }

    /// Chunk content into smaller pieces for embedding
    async fn chunk_content(&self, content: &str) -> Result<Vec<String>, Box<dyn Error>> {
        const CHUNK_SIZE: usize = 1000;
        const CHUNK_OVERLAP: usize = 200;

        let mut chunks = Vec::new();
        let content_len = content.len();
        
        if content_len <= CHUNK_SIZE {
            chunks.push(content.to_string());
            return Ok(chunks);
        }

        let mut start = 0;
        while start < content_len {
            let end = std::cmp::min(start + CHUNK_SIZE, content_len);
            let chunk = &content[start..end];
            
            // Try to break at word boundaries
            let chunk = if end < content_len {
                if let Some(last_space) = chunk.rfind(' ') {
                    &chunk[..last_space]
                } else {
                    chunk
                }
            } else {
                chunk
            };

            chunks.push(chunk.to_string());
            
            if end >= content_len {
                break;
            }
            
            start = end - CHUNK_OVERLAP;
        }

        Ok(chunks)
    }

    /// Generate embeddings and store in vector database
    async fn generate_and_store_embeddings(&self, chunks: &[String], request_id: &str) -> Result<usize, Box<dyn Error>> {
        let mut embeddings_count = 0;

        for (i, chunk) in chunks.iter().enumerate() {
            let chunk_id = format!("{}_{}", request_id, i);
            
            // Generate embedding (this would call an actual embedding service)
            let embedding = self.generate_embedding(chunk).await?;
            
            // Store in vector database
            self.vector_db_service.store_embedding(&chunk_id, &embedding, chunk).await?;
            embeddings_count += 1;
        }

        Ok(embeddings_count)
    }

    /// Generate embedding for text (placeholder implementation)
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, Box<dyn Error>> {
        // In a real implementation, this would call an embedding service like OpenAI, Cohere, etc.
        // For now, return a dummy embedding
        let embedding_size = 1536; // OpenAI ada-002 size
        let mut embedding = vec![0.0f32; embedding_size];
        
        // Simple hash-based embedding for demonstration
        let hash = text.len() as f32;
        for (i, val) in embedding.iter_mut().enumerate() {
            *val = ((hash + i as f32) % 2.0) - 1.0;
        }
        
        Ok(embedding)
    }

    /// Create indexing request for repository
    pub fn create_repository_request(repo_url: String, metadata: HashMap<String, String>) -> IndexingRequest {
        IndexingRequest {
            id: Uuid::new_v4().to_string(),
            source_type: IndexingSourceType::Repository,
            source_url: repo_url,
            metadata,
            priority: IndexingPriority::Normal,
            created_at: Utc::now(),
        }
    }

    /// Create indexing request for documentation
    pub fn create_documentation_request(doc_url: String, metadata: HashMap<String, String>) -> IndexingRequest {
        IndexingRequest {
            id: Uuid::new_v4().to_string(),
            source_type: IndexingSourceType::Documentation,
            source_url: doc_url,
            metadata,
            priority: IndexingPriority::Normal,
            created_at: Utc::now(),
        }
    }

    /// Create indexing request for URL
    pub fn create_url_request(url: String, metadata: HashMap<String, String>) -> IndexingRequest {
        IndexingRequest {
            id: Uuid::new_v4().to_string(),
            source_type: IndexingSourceType::Url,
            source_url: url,
            metadata,
            priority: IndexingPriority::Normal,
            created_at: Utc::now(),
        }
    }
}

/// Legacy function for backward compatibility
pub async fn index_documents() -> Result<(), Box<dyn Error>> {
    info!("Starting legacy document indexing process");
    
    let indexing_service = IndexingService::new();
    
    // Create a sample indexing request
    let request = IndexingRequest {
        id: Uuid::new_v4().to_string(),
        source_type: IndexingSourceType::Documentation,
        source_url: "https://docs.example.com".to_string(),
        metadata: HashMap::new(),
        priority: IndexingPriority::Normal,
        created_at: Utc::now(),
    };
    
    match indexing_service.index_content(request).await {
        Ok(result) => {
            info!("Legacy indexing completed: {:?}", result);
            Ok(())
        }
        Err(e) => {
            error!("Legacy indexing failed: {}", e);
            Err(e)
        }
    }
}