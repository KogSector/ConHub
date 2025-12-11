//! Local Filesystem Ingestion Handler
//!
//! Handles ingestion of local files from configured server-side directories.
//! Files are processed and sent through the chunker → vector_rag + graph_rag pipeline.

use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{info, warn, error};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::connectors::types::{ContentType, ConnectorType, DocumentForEmbedding};
use crate::services::EmbeddingClient;

/// Request to sync local files
#[derive(Debug, Deserialize)]
pub struct LocalSyncRequest {
    /// Name/profile for this local sync (e.g., "project-root")
    pub profile: Option<String>,
    
    /// Base directory to sync (absolute path on server)
    /// If not provided, uses profile or env-configured default
    pub base_path: Option<String>,
    
    /// File extensions to include (without leading dot)
    /// e.g., ["rs", "ts", "tsx", "md"]
    pub include_extensions: Option<Vec<String>>,
    
    /// Paths/patterns to exclude (relative to base_path)
    /// e.g., ["target", "node_modules", ".git"]
    pub exclude_paths: Option<Vec<String>>,
    
    /// Maximum file size in MB
    pub max_file_size_mb: Option<i64>,
}

/// Response from local sync
#[derive(Debug, Serialize)]
pub struct LocalSyncResponse {
    pub success: bool,
    pub documents_processed: usize,
    pub embeddings_created: usize,
    pub sync_duration_ms: u64,
    pub error_message: Option<String>,
    pub graph_job_id: Option<Uuid>,
}

/// Configuration for local file sync
#[derive(Debug, Clone)]
pub struct LocalSyncConfig {
    pub base_path: PathBuf,
    pub include_extensions: HashSet<String>,
    pub exclude_paths: Vec<String>,
    pub max_file_size_bytes: i64,
}

impl Default for LocalSyncConfig {
    fn default() -> Self {
        let default_extensions: HashSet<String> = [
            "rs", "py", "js", "ts", "tsx", "jsx", "go", "java", "c", "cpp", "h", "hpp",
            "rb", "php", "swift", "kt", "scala", "cs", "fs", "clj", "ex", "exs",
            "md", "txt", "json", "yaml", "yml", "toml", "xml", "html", "css", "scss",
            "sql", "sh", "bash", "dockerfile", "makefile",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        Self {
            base_path: PathBuf::new(),
            include_extensions: default_extensions,
            exclude_paths: vec![
                "node_modules".to_string(),
                "target".to_string(),
                "dist".to_string(),
                "build".to_string(),
                ".git".to_string(),
                ".next".to_string(),
                "__pycache__".to_string(),
                "vendor".to_string(),
                ".cargo".to_string(),
                "coverage".to_string(),
            ],
            max_file_size_bytes: 5 * 1024 * 1024, // 5 MB
        }
    }
}

/// Detect language from file extension
fn detect_language(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    
    let lang = match ext.as_str() {
        "rs" => "rust",
        "py" | "pyx" | "pyi" => "python",
        "js" | "mjs" => "javascript",
        "ts" => "typescript",
        "tsx" => "tsx",
        "jsx" => "jsx",
        "go" => "go",
        "java" => "java",
        "c" | "h" => "c",
        "cpp" | "cxx" | "cc" | "hpp" => "cpp",
        "rb" => "ruby",
        "php" => "php",
        "swift" => "swift",
        "kt" | "kts" => "kotlin",
        "scala" => "scala",
        "cs" => "csharp",
        "fs" => "fsharp",
        "clj" | "cljs" => "clojure",
        "ex" | "exs" => "elixir",
        "md" | "markdown" => "markdown",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "xml" => "xml",
        "html" | "htm" => "html",
        "css" => "css",
        "scss" | "sass" => "scss",
        "sql" => "sql",
        "sh" | "bash" | "zsh" => "shell",
        _ => return None,
    };
    
    Some(lang.to_string())
}

/// Determine content type from file extension
fn detect_content_type(path: &Path) -> ContentType {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "md" | "markdown" => ContentType::Markdown,
        "txt" | "rst" => ContentType::Text,
        "html" | "htm" => ContentType::Html,
        _ => ContentType::Code,
    }
}

/// Check if a path should be excluded
fn should_exclude(path: &Path, base_path: &Path, exclude_patterns: &[String]) -> bool {
    let relative = path.strip_prefix(base_path).unwrap_or(path);
    let relative_str = relative.to_string_lossy();
    
    for pattern in exclude_patterns {
        if relative_str.contains(pattern) {
            return true;
        }
    }
    false
}

/// Check if a file extension is included
fn should_include_extension(path: &Path, include_extensions: &HashSet<String>) -> bool {
    if include_extensions.is_empty() {
        return true;
    }
    
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| include_extensions.contains(&e.to_lowercase()))
        .unwrap_or(false)
}

/// Sync local files from a directory
async fn sync_local_files(
    config: &LocalSyncConfig,
) -> Result<Vec<DocumentForEmbedding>, String> {
    if !config.base_path.exists() {
        return Err(format!(
            "Base path does not exist: {}",
            config.base_path.display()
        ));
    }

    if !config.base_path.is_dir() {
        return Err(format!(
            "Base path is not a directory: {}",
            config.base_path.display()
        ));
    }

    info!(
        "📂 Starting local file sync from: {}",
        config.base_path.display()
    );

    let mut documents = Vec::new();
    let source_id = Uuid::new_v5(&Uuid::NAMESPACE_URL, config.base_path.to_string_lossy().as_bytes());

    for entry in WalkDir::new(&config.base_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip directories
        if path.is_dir() {
            continue;
        }

        // Check exclusions
        if should_exclude(path, &config.base_path, &config.exclude_paths) {
            continue;
        }

        // Check extension filter
        if !should_include_extension(path, &config.include_extensions) {
            continue;
        }

        // Check file size
        let metadata = match std::fs::metadata(path) {
            Ok(m) => m,
            Err(e) => {
                warn!("Failed to read metadata for {}: {}", path.display(), e);
                continue;
            }
        };

        if metadata.len() as i64 > config.max_file_size_bytes {
            continue;
        }

        // Read file content
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                // Skip binary files or files that can't be read as text
                warn!("Skipping {} (not valid UTF-8 or read error): {}", path.display(), e);
                continue;
            }
        };

        let relative_path = path
            .strip_prefix(&config.base_path)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let language = detect_language(path);
        let content_type = detect_content_type(path);

        documents.push(DocumentForEmbedding {
            id: Uuid::new_v4(),
            source_id,
            connector_type: ConnectorType::LocalFile,
            external_id: relative_path.clone(),
            name: file_name,
            path: Some(relative_path.clone()),
            content,
            content_type,
            metadata: serde_json::json!({
                "source": "local_fs",
                "base_path": config.base_path.to_string_lossy(),
                "relative_path": relative_path,
                "size_bytes": metadata.len(),
                "language": language,
            }),
            chunks: None,
            block_type: Some("code".to_string()),
            language,
        });
    }

    info!("📄 Found {} files to process", documents.len());
    Ok(documents)
}

/// POST /api/data/local/sync
/// 
/// Sync local files from a server-side directory into the ingestion pipeline.
pub async fn sync_local_files_handler(
    _http_req: HttpRequest,
    req: web::Json<LocalSyncRequest>,
    embedding_client: web::Data<EmbeddingClient>,
    // Note: Graph ingestion handled via chunker service, not directly here
) -> Result<HttpResponse> {
    let start_time = Instant::now();

    info!("📂 Local file sync request received");

    // Build config
    let mut config = LocalSyncConfig::default();

    // Set base path
    let base_path = if let Some(ref path) = req.base_path {
        PathBuf::from(path)
    } else if let Some(ref profile) = req.profile {
        // Look up profile from env or config
        let env_key = format!("LOCAL_SYNC_PATH_{}", profile.to_uppercase());
        match std::env::var(&env_key) {
            Ok(path) => PathBuf::from(path),
            Err(_) => {
                return Ok(HttpResponse::BadRequest().json(LocalSyncResponse {
                    success: false,
                    documents_processed: 0,
                    embeddings_created: 0,
                    sync_duration_ms: 0,
                    error_message: Some(format!(
                        "No base path configured for profile '{}'. Set {} env var.",
                        profile, env_key
                    )),
                    graph_job_id: None,
                }));
            }
        }
    } else {
        // Use default from env
        match std::env::var("LOCAL_SYNC_PATH_DEFAULT") {
            Ok(path) => PathBuf::from(path),
            Err(_) => {
                return Ok(HttpResponse::BadRequest().json(LocalSyncResponse {
                    success: false,
                    documents_processed: 0,
                    embeddings_created: 0,
                    sync_duration_ms: 0,
                    error_message: Some(
                        "No base_path provided and LOCAL_SYNC_PATH_DEFAULT not set".to_string(),
                    ),
                    graph_job_id: None,
                }));
            }
        }
    };

    config.base_path = base_path;

    // Apply extension filter
    if let Some(ref exts) = req.include_extensions {
        config.include_extensions = exts.iter().map(|s| s.to_lowercase()).collect();
    }

    // Apply exclusion patterns
    if let Some(ref excludes) = req.exclude_paths {
        config.exclude_paths.extend(excludes.iter().cloned());
    }

    // Apply size limit
    if let Some(max_mb) = req.max_file_size_mb {
        config.max_file_size_bytes = max_mb * 1024 * 1024;
    }

    // Sync files
    let documents = match sync_local_files(&config).await {
        Ok(docs) => docs,
        Err(e) => {
            error!("❌ Local file sync failed: {}", e);
            return Ok(HttpResponse::BadRequest().json(LocalSyncResponse {
                success: false,
                documents_processed: 0,
                embeddings_created: 0,
                sync_duration_ms: start_time.elapsed().as_millis() as u64,
                error_message: Some(e),
                graph_job_id: None,
            }));
        }
    };

    let doc_count = documents.len();

    // Send to embedding service
    let mut embeddings_created = 0;
    if let Err(e) = embedding_client.embed_documents(documents.clone()).await {
        warn!("⚠️ Embedding service error (continuing): {}", e);
    } else {
        embeddings_created = doc_count;
        info!("✅ Sent {} documents to embedding service", doc_count);
    }

    // Graph RAG ingestion is handled via the chunker service pipeline
    // Documents go: data → chunker → vector_rag + graph_rag
    let graph_job_id: Option<Uuid> = None;

    let sync_duration = start_time.elapsed().as_millis() as u64;

    info!(
        "✅ Local file sync completed: {} documents in {}ms",
        doc_count, sync_duration
    );

    Ok(HttpResponse::Ok().json(LocalSyncResponse {
        success: true,
        documents_processed: doc_count,
        embeddings_created,
        sync_duration_ms: sync_duration,
        error_message: None,
        graph_job_id,
    }))
}

/// Configure routes for local file sync
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/data/local")
            .route("/sync", web::post().to(sync_local_files_handler)),
    );
}
