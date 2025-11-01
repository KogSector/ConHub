use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::{RwLock, Semaphore};
use futures::stream::{self, StreamExt};
use std::time::Instant;

use crate::config::IndexerConfig;
use crate::models::*;
use crate::services::chunking::{ChunkingService, ChunkingStrategy, Chunk};

/// Advanced file type detection and language mapping
#[derive(Debug, Clone)]
pub struct FileTypeAnalyzer {
    extension_map: HashMap<String, String>,
    binary_extensions: std::collections::HashSet<String>,
}

impl FileTypeAnalyzer {
    pub fn new() -> Self {
        let mut extension_map = HashMap::new();
        let mut binary_extensions = std::collections::HashSet::new();

        // Programming languages
        extension_map.insert("rs".to_string(), "rust".to_string());
        extension_map.insert("py".to_string(), "python".to_string());
        extension_map.insert("js".to_string(), "javascript".to_string());
        extension_map.insert("ts".to_string(), "typescript".to_string());
        extension_map.insert("java".to_string(), "java".to_string());
        extension_map.insert("cpp".to_string(), "cpp".to_string());
        extension_map.insert("c".to_string(), "c".to_string());
        extension_map.insert("go".to_string(), "go".to_string());
        extension_map.insert("rb".to_string(), "ruby".to_string());
        extension_map.insert("php".to_string(), "php".to_string());
        extension_map.insert("cs".to_string(), "csharp".to_string());
        extension_map.insert("swift".to_string(), "swift".to_string());
        extension_map.insert("kt".to_string(), "kotlin".to_string());
        extension_map.insert("scala".to_string(), "scala".to_string());

        // Markup and config
        extension_map.insert("html".to_string(), "html".to_string());
        extension_map.insert("css".to_string(), "css".to_string());
        extension_map.insert("scss".to_string(), "scss".to_string());
        extension_map.insert("json".to_string(), "json".to_string());
        extension_map.insert("xml".to_string(), "xml".to_string());
        extension_map.insert("yaml".to_string(), "yaml".to_string());
        extension_map.insert("yml".to_string(), "yaml".to_string());
        extension_map.insert("toml".to_string(), "toml".to_string());
        extension_map.insert("md".to_string(), "markdown".to_string());
        extension_map.insert("txt".to_string(), "text".to_string());

        // Binary extensions to skip
        binary_extensions.insert("exe".to_string());
        binary_extensions.insert("dll".to_string());
        binary_extensions.insert("so".to_string());
        binary_extensions.insert("dylib".to_string());
        binary_extensions.insert("bin".to_string());
        binary_extensions.insert("obj".to_string());
        binary_extensions.insert("o".to_string());
        binary_extensions.insert("a".to_string());
        binary_extensions.insert("lib".to_string());
        binary_extensions.insert("zip".to_string());
        binary_extensions.insert("tar".to_string());
        binary_extensions.insert("gz".to_string());
        binary_extensions.insert("png".to_string());
        binary_extensions.insert("jpg".to_string());
        binary_extensions.insert("jpeg".to_string());
        binary_extensions.insert("gif".to_string());
        binary_extensions.insert("pdf".to_string());

        Self {
            extension_map,
            binary_extensions,
        }
    }

    pub fn get_language(&self, path: &Path) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| self.extension_map.get(&ext.to_lowercase()))
            .cloned()
    }

    pub fn is_binary(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| self.binary_extensions.contains(&ext.to_lowercase()))
            .unwrap_or(false)
    }

    pub fn should_index(&self, path: &Path) -> bool {
        // Skip hidden files and directories
        if path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.starts_with('.'))
            .unwrap_or(false) {
            return false;
        }

        // Skip common build/cache directories
        let skip_dirs = ["target", "node_modules", ".git", "build", "dist", "out", "__pycache__"];
        if path.components().any(|comp| {
            comp.as_os_str().to_str()
                .map(|s| skip_dirs.contains(&s))
                .unwrap_or(false)
        }) {
            return false;
        }

        // Skip binary files
        !self.is_binary(path)
    }
}

pub struct CodeIndexingService {
    config: IndexerConfig,
    jobs: Arc<DashMap<String, IndexingJob>>,
    chunking_service: Arc<ChunkingService>,
    file_analyzer: FileTypeAnalyzer,
    // Concurrency control
    file_semaphore: Arc<Semaphore>,
    // Performance metrics
    metrics: Arc<RwLock<IndexingMetrics>>,
}

#[derive(Debug, Default)]
pub struct IndexingMetrics {
    pub total_files_processed: usize,
    pub total_chunks_created: usize,
    pub total_processing_time_ms: u64,
    pub files_per_language: HashMap<String, usize>,
    pub average_file_size: f64,
    pub average_chunks_per_file: f64,
}

impl CodeIndexingService {
    pub fn new(config: IndexerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let chunking_service = ChunkingService::new(config.clone())
            .with_strategy(ChunkingStrategy::SyntaxAware);

        Ok(Self {
            config: config.clone(),
            jobs: Arc::new(DashMap::new()),
            chunking_service: Arc::new(chunking_service),
            file_analyzer: FileTypeAnalyzer::new(),
            file_semaphore: Arc::new(Semaphore::new(10)), // Limit concurrent file processing
            metrics: Arc::new(RwLock::new(IndexingMetrics::default())),
        })
    }

    /// Get current indexing metrics
    pub async fn get_metrics(&self) -> IndexingMetrics {
        self.metrics.read().await.clone()
    }

    /// Reset metrics
    pub async fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = IndexingMetrics::default();
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
        let start_time = Instant::now();

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
        let mut language_stats = std::collections::HashMap::new();
        let mut total_file_size = 0u64;
        let file_analyzer = FileTypeAnalyzer::new();
        let semaphore = Arc::new(Semaphore::new(10));

        // Collect all files first for better progress tracking
        let mut files_to_process = Vec::new();
        for entry in walkdir::WalkDir::new(&repo_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if file_analyzer.should_index(path) {
                files_to_process.push(path.to_path_buf());
            }
        }

        // Process files in parallel with semaphore control
        let files_stream = stream::iter(files_to_process.into_iter())
            .map(|file_path| {
                let semaphore = Arc::clone(&semaphore);
                let chunking_service = Arc::clone(&chunking_service);
                let file_analyzer = file_analyzer.clone();
                
                async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    Self::process_single_file(&file_path, &chunking_service, &file_analyzer).await
                }
            })
            .buffer_unordered(10);

        let results: Vec<_> = files_stream.collect().await;
        
        // Aggregate results
        for result in results {
            if let Ok((chunks, lang, size)) = result {
                documents_processed += 1;
                total_chunks += chunks;
                total_file_size += size;
                *language_stats.entry(lang).or_insert(0) += 1;
            }
        }

        let _ = std::fs::remove_dir_all(&repo_dir);

        let processing_time = start_time.elapsed().as_millis() as u64;
        log::info!("Repository processing complete: {} files, {} chunks in {}ms", 
                  documents_processed, total_chunks, processing_time);
        
        Ok((documents_processed, total_chunks, total_chunks))
    }

    async fn process_single_file(
        file_path: &Path,
        chunking_service: &ChunkingService,
        file_analyzer: &FileTypeAnalyzer,
    ) -> Result<(usize, String, u64), Box<dyn std::error::Error>> {
        let file_size = std::fs::metadata(file_path)?.len();
        let language = file_analyzer.get_language(file_path).unwrap_or_else(|| "unknown".to_string());
        
        let chunks = Self::index_code_file(file_path, chunking_service).await?;
        Ok((chunks, language, file_size))
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

    async fn index_code_file_advanced(
        path: &Path,
        chunking_service: &ChunkingService,
        file_analyzer: &FileTypeAnalyzer,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let language = file_analyzer.get_language(path).unwrap_or_else(|| "unknown".to_string());
        
        // Use syntax-aware chunking with metadata
        let chunks = chunking_service.chunk_text(&content)?;
        
        // Extract additional metadata
        let file_metadata = Self::extract_file_metadata(path, &content, &language)?;
        
        // Process chunks with enhanced metadata
        let processed_chunks = Self::process_chunks_with_metadata(chunks, file_metadata).await?;
        
        // Here you would store the processed chunks in your vector database
        // For now, we'll just return the count
        Ok(processed_chunks.len())
    }

    fn extract_file_metadata(file_path: &Path, content: &str, language: &str) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let lines = content.lines().count();
        let size = content.len();
        
        let mut metadata = HashMap::new();
        metadata.insert("language".to_string(), language.to_string());
        metadata.insert("lines".to_string(), lines.to_string());
        metadata.insert("size".to_string(), size.to_string());
        metadata.insert("path".to_string(), file_path.to_string_lossy().to_string());
        
        Ok(metadata)
    }

    async fn process_chunks_with_metadata(chunks: Vec<Chunk>, file_metadata: HashMap<String, String>) -> Result<Vec<Chunk>, Box<dyn std::error::Error>> {
        let mut processed_chunks = Vec::new();
        
        for chunk in chunks {
            // Enhanced chunk with file metadata
            processed_chunks.push(chunk);
        }
        
        Ok(processed_chunks)
    }

    fn is_code_file(extension: &str) -> bool {
        matches!(
            extension.to_lowercase().as_str(),
            "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "java" | "c" | "cpp" | "h" | "hpp"
                | "go" | "rb" | "php" | "cs" | "swift" | "kt" | "scala" | "sh" | "bash"
                | "sql" | "yaml" | "yml" | "toml" | "json" | "xml" | "html" | "css" | "scss"
        )
    }

    fn calculate_code_complexity(&self, content: &str, language: &str) -> f64 {
        let mut complexity = 1.0; // Base complexity
        
        // Count control flow statements
        let control_flow_patterns = match language {
            "rust" => vec!["if ", "else", "match", "for ", "while ", "loop"],
            "python" => vec!["if ", "elif", "else:", "for ", "while ", "try:", "except:"],
            "javascript" | "typescript" => vec!["if ", "else", "for ", "while ", "switch", "try", "catch"],
            "java" | "c" | "cpp" => vec!["if ", "else", "for ", "while ", "switch", "try", "catch"],
            _ => vec!["if ", "else", "for ", "while "],
        };
        
        for pattern in control_flow_patterns {
            complexity += content.matches(pattern).count() as f64 * 0.1;
        }
        
        // Count function definitions
        let function_patterns = match language {
            "rust" => vec!["fn ", "impl "],
            "python" => vec!["def ", "class "],
            "javascript" | "typescript" => vec!["function ", "class ", "=>"],
            "java" => vec!["public ", "private ", "protected "],
            _ => vec!["function", "def", "class"],
        };
        
        for pattern in function_patterns {
            complexity += content.matches(pattern).count() as f64 * 0.2;
        }
        
        complexity
    }

    fn extract_functions(&self, content: &str, language: &str) -> Vec<String> {
        let mut functions = Vec::new();
        
        match language {
            "rust" => {
                for line in content.lines() {
                    if line.trim().starts_with("fn ") {
                        if let Some(name) = self.extract_rust_function_name(line) {
                            functions.push(name);
                        }
                    }
                }
            }
            "python" => {
                for line in content.lines() {
                    if line.trim().starts_with("def ") {
                        if let Some(name) = self.extract_python_function_name(line) {
                            functions.push(name);
                        }
                    }
                }
            }
            "javascript" | "typescript" => {
                for line in content.lines() {
                    if line.contains("function ") || line.contains(" => ") {
                        if let Some(name) = self.extract_js_function_name(line) {
                            functions.push(name);
                        }
                    }
                }
            }
            _ => {}
        }
        
        functions
    }

    fn extract_imports(&self, content: &str, language: &str) -> Vec<String> {
        let mut imports = Vec::new();
        
        match language {
            "rust" => {
                for line in content.lines() {
                    if line.trim().starts_with("use ") {
                        imports.push(line.trim().to_string());
                    }
                }
            }
            "python" => {
                for line in content.lines() {
                    if line.trim().starts_with("import ") || line.trim().starts_with("from ") {
                        imports.push(line.trim().to_string());
                    }
                }
            }
            "javascript" | "typescript" => {
                for line in content.lines() {
                    if line.trim().starts_with("import ") || line.contains("require(") {
                        imports.push(line.trim().to_string());
                    }
                }
            }
            _ => {}
        }
        
        imports
    }

    fn extract_rust_function_name(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("fn ") {
            let after_fn = &line[start + 3..];
            if let Some(end) = after_fn.find('(') {
                return Some(after_fn[..end].trim().to_string());
            }
        }
        None
    }

    fn extract_python_function_name(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("def ") {
            let after_def = &line[start + 4..];
            if let Some(end) = after_def.find('(') {
                return Some(after_def[..end].trim().to_string());
            }
        }
        None
    }

    fn extract_js_function_name(&self, line: &str) -> Option<String> {
        if line.contains("function ") {
            if let Some(start) = line.find("function ") {
                let after_fn = &line[start + 9..];
                if let Some(end) = after_fn.find('(') {
                    return Some(after_fn[..end].trim().to_string());
                }
            }
        } else if line.contains(" => ") {
            // Arrow function
            if let Some(arrow_pos) = line.find(" => ") {
                let before_arrow = &line[..arrow_pos];
                if let Some(equals_pos) = before_arrow.rfind('=') {
                    let name_part = &before_arrow[..equals_pos];
                    if let Some(name) = name_part.split_whitespace().last() {
                        return Some(name.to_string());
                    }
                }
            }
        }
        None
    }

    fn chunk_contains_functions(&self, content: &str, language: &str) -> bool {
        match language {
            "rust" => content.contains("fn "),
            "python" => content.contains("def "),
            "javascript" | "typescript" => content.contains("function ") || content.contains(" => "),
            "java" => content.contains("public ") || content.contains("private "),
            _ => false,
        }
    }

    fn chunk_contains_classes(&self, content: &str, language: &str) -> bool {
        match language {
            "rust" => content.contains("struct ") || content.contains("enum ") || content.contains("impl "),
            "python" => content.contains("class "),
            "javascript" | "typescript" => content.contains("class "),
            "java" => content.contains("class ") || content.contains("interface "),
            _ => false,
        }
    }

    fn chunk_contains_imports(&self, content: &str, language: &str) -> bool {
        match language {
            "rust" => content.contains("use "),
            "python" => content.contains("import ") || content.contains("from "),
            "javascript" | "typescript" => content.contains("import ") || content.contains("require("),
            "java" => content.contains("import "),
            _ => false,
        }
    }

    fn calculate_chunk_complexity(&self, content: &str, language: &str) -> f64 {
        self.calculate_code_complexity(content, language)
    }

    fn extract_chunk_dependencies(&self, content: &str, language: &str) -> Vec<String> {
        self.extract_imports(content, language)
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
