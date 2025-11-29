//! GitHub Ingestion Service
//! 
//! Handles the actual sync of:
//! - Code files from repositories
//! - Issues and their comments
//! - Pull requests and their reviews/comments
//! 
//! Sends documents to the chunker service for processing

use std::sync::Arc;
use std::collections::HashSet;
use uuid::Uuid;
use tracing::{info, warn, error, debug};
use anyhow::{Result, Context, anyhow};
use chrono::{DateTime, Utc, Duration};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::github_app_client::{
    GitHubAppClient, GitTreeEntry, BlobContent,
    GitHubIssue, GitHubComment, GitHubPullRequest,
    GitHubReview, GitHubReviewComment,
};

use conhub_models::chunking::{SourceKind, SourceItem, StartChunkJobRequest};

// ============================================================================
// Configuration
// ============================================================================

/// Configuration for code sync
#[derive(Debug, Clone)]
pub struct CodeSyncConfig {
    pub branch: String,
    pub exclude_paths: Vec<String>,
    pub include_extensions: Vec<String>,
    pub max_file_size_bytes: i64,
}

impl Default for CodeSyncConfig {
    fn default() -> Self {
        Self {
            branch: "main".to_string(),
            exclude_paths: vec![
                "node_modules".to_string(),
                "dist".to_string(),
                "build".to_string(),
                ".git".to_string(),
                "vendor".to_string(),
                "__pycache__".to_string(),
                ".next".to_string(),
                "target".to_string(),
                ".cargo".to_string(),
                "coverage".to_string(),
            ],
            include_extensions: vec![], // Empty = all text files
            max_file_size_bytes: 5 * 1024 * 1024, // 5MB
        }
    }
}

/// Configuration for issues sync
#[derive(Debug, Clone)]
pub struct IssuesSyncConfig {
    pub include_closed: bool,
    pub since_days: i64,
    pub labels_filter: Vec<String>,
}

impl Default for IssuesSyncConfig {
    fn default() -> Self {
        Self {
            include_closed: false,
            since_days: 90,
            labels_filter: vec![],
        }
    }
}

/// Configuration for PRs sync
#[derive(Debug, Clone)]
pub struct PrsSyncConfig {
    pub include_closed: bool,
    pub include_merged: bool,
    pub since_days: i64,
    pub include_diffs: bool,
}

impl Default for PrsSyncConfig {
    fn default() -> Self {
        Self {
            include_closed: false,
            include_merged: true,
            since_days: 90,
            include_diffs: false,
        }
    }
}

// ============================================================================
// Sync Progress Tracking
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct SyncProgress {
    pub items_total: i32,
    pub items_processed: i32,
    pub items_failed: i32,
    pub chunks_created: i32,
    pub current_phase: String,
    pub errors: Vec<String>,
}

// ============================================================================
// GitHub Ingestion Service
// ============================================================================

pub struct GitHubIngestionService {
    github_client: Arc<GitHubAppClient>,
    http_client: Client,
    chunker_url: String,
}

impl GitHubIngestionService {
    pub fn new(github_client: Arc<GitHubAppClient>, chunker_url: String) -> Self {
        Self {
            github_client,
            http_client: Client::new(),
            chunker_url,
        }
    }
    
    /// Sync code from a repository
    pub async fn sync_code(
        &self,
        installation_id: i64,
        owner: &str,
        repo: &str,
        config: &CodeSyncConfig,
        progress_callback: Option<Box<dyn Fn(&SyncProgress) + Send + Sync>>,
    ) -> Result<SyncProgress> {
        let mut progress = SyncProgress {
            current_phase: "Fetching repository tree".to_string(),
            ..Default::default()
        };
        
        if let Some(ref cb) = progress_callback {
            cb(&progress);
        }
        
        info!("📂 Starting code sync for {}/{} (branch: {})", owner, repo, config.branch);
        
        // Get repository tree
        let tree = self.github_client
            .get_repository_tree(installation_id, owner, repo, &config.branch)
            .await
            .context("Failed to get repository tree")?;
        
        if tree.truncated {
            warn!("⚠️ Repository tree was truncated - some files may be missing");
        }
        
        // Filter to blob entries (files)
        let files: Vec<&GitTreeEntry> = tree.tree.iter()
            .filter(|e| e.entry_type == "blob")
            .filter(|e| !self.should_exclude_path(&e.path, &config.exclude_paths))
            .filter(|e| self.should_include_extension(&e.path, &config.include_extensions))
            .filter(|e| e.size.unwrap_or(0) <= config.max_file_size_bytes)
            .collect();
        
        progress.items_total = files.len() as i32;
        progress.current_phase = format!("Processing {} files", files.len());
        
        if let Some(ref cb) = progress_callback {
            cb(&progress);
        }
        
        info!("📄 Found {} files to process", files.len());
        
        // Process files in batches
        let batch_size = 50;
        let mut source_items: Vec<SourceItem> = Vec::new();
        
        for (idx, file) in files.iter().enumerate() {
            // Get file content
            match self.github_client.get_blob_content(installation_id, owner, repo, &file.sha).await {
                Ok(blob) => {
                    match blob.decode_to_string() {
                        Ok(content) => {
                            let language = detect_language(&file.path);
                            
                            source_items.push(SourceItem {
                                id: Uuid::new_v4(),
                                source_id: Uuid::nil(), // Will be set by caller
                                source_kind: SourceKind::CodeRepo,
                                content_type: format!("text/code:{}", language.as_deref().unwrap_or("unknown")),
                                content,
                                metadata: serde_json::json!({
                                    "path": file.path,
                                    "sha": file.sha,
                                    "size": file.size,
                                    "language": language,
                                    "branch": config.branch,
                                    "repo": format!("{}/{}", owner, repo),
                                    "url": format!("https://github.com/{}/{}/blob/{}/{}", owner, repo, config.branch, file.path),
                                }),
                                created_at: Some(Utc::now()),
                            });
                            
                            progress.items_processed += 1;
                        }
                        Err(e) => {
                            debug!("Skipping binary file {}: {}", file.path, e);
                            progress.items_failed += 1;
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to get content for {}: {}", file.path, e);
                    progress.items_failed += 1;
                    progress.errors.push(format!("{}: {}", file.path, e));
                }
            }
            
            // Send batch to chunker
            if source_items.len() >= batch_size || idx == files.len() - 1 {
                if !source_items.is_empty() {
                    match self.send_to_chunker(Uuid::nil(), SourceKind::CodeRepo, &source_items).await {
                        Ok(chunks) => {
                            progress.chunks_created += chunks;
                        }
                        Err(e) => {
                            error!("Failed to send batch to chunker: {}", e);
                            progress.errors.push(format!("Chunker error: {}", e));
                        }
                    }
                    source_items.clear();
                }
                
                if let Some(ref cb) = progress_callback {
                    cb(&progress);
                }
            }
        }
        
        progress.current_phase = "Completed".to_string();
        info!("✅ Code sync completed: {} files processed, {} chunks created", 
              progress.items_processed, progress.chunks_created);
        
        Ok(progress)
    }
    
    /// Sync issues from a repository
    pub async fn sync_issues(
        &self,
        installation_id: i64,
        owner: &str,
        repo: &str,
        config: &IssuesSyncConfig,
        progress_callback: Option<Box<dyn Fn(&SyncProgress) + Send + Sync>>,
    ) -> Result<SyncProgress> {
        let mut progress = SyncProgress {
            current_phase: "Fetching issues".to_string(),
            ..Default::default()
        };
        
        if let Some(ref cb) = progress_callback {
            cb(&progress);
        }
        
        info!("🎫 Starting issues sync for {}/{}", owner, repo);
        
        let state = if config.include_closed { "all" } else { "open" };
        let since = Some(Utc::now() - Duration::days(config.since_days));
        
        let mut all_issues: Vec<GitHubIssue> = Vec::new();
        let mut page = 1;
        
        // Paginate through issues
        loop {
            let issues = self.github_client
                .list_issues(installation_id, owner, repo, state, since, page, 100)
                .await
                .context("Failed to list issues")?;
            
            if issues.is_empty() {
                break;
            }
            
            // Filter out PRs (GitHub API returns PRs in issues endpoint)
            let pure_issues: Vec<GitHubIssue> = issues.into_iter()
                .filter(|i| !i.is_pull_request())
                .filter(|i| {
                    if config.labels_filter.is_empty() {
                        true
                    } else {
                        i.labels.iter().any(|l| config.labels_filter.contains(&l.name))
                    }
                })
                .collect();
            
            all_issues.extend(pure_issues);
            page += 1;
            
            if page > 10 { // Safety limit
                break;
            }
        }
        
        progress.items_total = all_issues.len() as i32;
        progress.current_phase = format!("Processing {} issues", all_issues.len());
        
        if let Some(ref cb) = progress_callback {
            cb(&progress);
        }
        
        info!("🎫 Found {} issues to process", all_issues.len());
        
        let mut source_items: Vec<SourceItem> = Vec::new();
        
        for issue in &all_issues {
            // Get comments for this issue
            let comments = self.github_client
                .get_issue_comments(installation_id, owner, repo, issue.number, 1, 100)
                .await
                .unwrap_or_default();
            
            // Build issue content
            let content = build_issue_content(issue, &comments);
            
            source_items.push(SourceItem {
                id: Uuid::new_v4(),
                source_id: Uuid::nil(),
                source_kind: SourceKind::Ticketing,
                content_type: "text/issue".to_string(),
                content,
                metadata: serde_json::json!({
                    "type": "issue",
                    "number": issue.number,
                    "title": issue.title,
                    "state": issue.state,
                    "author": issue.user.login,
                    "labels": issue.labels.iter().map(|l| &l.name).collect::<Vec<_>>(),
                    "assignees": issue.assignees.iter().map(|a| &a.login).collect::<Vec<_>>(),
                    "comments_count": comments.len(),
                    "repo": format!("{}/{}", owner, repo),
                    "url": issue.html_url,
                    "created_at": issue.created_at.to_rfc3339(),
                    "updated_at": issue.updated_at.to_rfc3339(),
                }),
                created_at: Some(issue.created_at),
            });
            
            progress.items_processed += 1;
            
            if let Some(ref cb) = progress_callback {
                cb(&progress);
            }
        }
        
        // Send to chunker
        if !source_items.is_empty() {
            match self.send_to_chunker(Uuid::nil(), SourceKind::Ticketing, &source_items).await {
                Ok(chunks) => {
                    progress.chunks_created = chunks;
                }
                Err(e) => {
                    error!("Failed to send issues to chunker: {}", e);
                    progress.errors.push(format!("Chunker error: {}", e));
                }
            }
        }
        
        progress.current_phase = "Completed".to_string();
        info!("✅ Issues sync completed: {} issues processed, {} chunks created",
              progress.items_processed, progress.chunks_created);
        
        Ok(progress)
    }
    
    /// Sync pull requests from a repository
    pub async fn sync_prs(
        &self,
        installation_id: i64,
        owner: &str,
        repo: &str,
        config: &PrsSyncConfig,
        progress_callback: Option<Box<dyn Fn(&SyncProgress) + Send + Sync>>,
    ) -> Result<SyncProgress> {
        let mut progress = SyncProgress {
            current_phase: "Fetching pull requests".to_string(),
            ..Default::default()
        };
        
        if let Some(ref cb) = progress_callback {
            cb(&progress);
        }
        
        info!("🔀 Starting PRs sync for {}/{}", owner, repo);
        
        let state = if config.include_closed { "all" } else { "open" };
        
        let mut all_prs: Vec<GitHubPullRequest> = Vec::new();
        let mut page = 1;
        
        // Paginate through PRs
        loop {
            let prs = self.github_client
                .list_pull_requests(installation_id, owner, repo, state, page, 100)
                .await
                .context("Failed to list pull requests")?;
            
            if prs.is_empty() {
                break;
            }
            
            // Filter by date
            let since = Utc::now() - Duration::days(config.since_days);
            let filtered_prs: Vec<GitHubPullRequest> = prs.into_iter()
                .filter(|pr| pr.updated_at >= since)
                .filter(|pr| {
                    if config.include_merged {
                        true
                    } else {
                        pr.merged_at.is_none()
                    }
                })
                .collect();
            
            all_prs.extend(filtered_prs);
            page += 1;
            
            if page > 10 { // Safety limit
                break;
            }
        }
        
        progress.items_total = all_prs.len() as i32;
        progress.current_phase = format!("Processing {} pull requests", all_prs.len());
        
        if let Some(ref cb) = progress_callback {
            cb(&progress);
        }
        
        info!("🔀 Found {} PRs to process", all_prs.len());
        
        let mut source_items: Vec<SourceItem> = Vec::new();
        
        for pr in &all_prs {
            // Get reviews
            let reviews = self.github_client
                .get_pr_reviews(installation_id, owner, repo, pr.number, 1, 100)
                .await
                .unwrap_or_default();
            
            // Get review comments (inline)
            let review_comments = self.github_client
                .get_pr_review_comments(installation_id, owner, repo, pr.number, 1, 100)
                .await
                .unwrap_or_default();
            
            // Get regular comments
            let comments = self.github_client
                .get_issue_comments(installation_id, owner, repo, pr.number, 1, 100)
                .await
                .unwrap_or_default();
            
            // Build PR content
            let content = build_pr_content(pr, &reviews, &review_comments, &comments);
            
            source_items.push(SourceItem {
                id: Uuid::new_v4(),
                source_id: Uuid::nil(),
                source_kind: SourceKind::Ticketing, // PRs are like tickets with code context
                content_type: "text/pull_request".to_string(),
                content,
                metadata: serde_json::json!({
                    "type": "pull_request",
                    "number": pr.number,
                    "title": pr.title,
                    "state": pr.state,
                    "author": pr.user.login,
                    "labels": pr.labels.iter().map(|l| &l.name).collect::<Vec<_>>(),
                    "assignees": pr.assignees.iter().map(|a| &a.login).collect::<Vec<_>>(),
                    "base_branch": pr.base.branch_ref,
                    "head_branch": pr.head.branch_ref,
                    "merged": pr.merged,
                    "merged_at": pr.merged_at,
                    "additions": pr.additions,
                    "deletions": pr.deletions,
                    "changed_files": pr.changed_files,
                    "reviews_count": reviews.len(),
                    "comments_count": comments.len() + review_comments.len(),
                    "repo": format!("{}/{}", owner, repo),
                    "url": pr.html_url,
                    "created_at": pr.created_at.to_rfc3339(),
                    "updated_at": pr.updated_at.to_rfc3339(),
                }),
                created_at: Some(pr.created_at),
            });
            
            progress.items_processed += 1;
            
            if let Some(ref cb) = progress_callback {
                cb(&progress);
            }
        }
        
        // Send to chunker
        if !source_items.is_empty() {
            match self.send_to_chunker(Uuid::nil(), SourceKind::Ticketing, &source_items).await {
                Ok(chunks) => {
                    progress.chunks_created = chunks;
                }
                Err(e) => {
                    error!("Failed to send PRs to chunker: {}", e);
                    progress.errors.push(format!("Chunker error: {}", e));
                }
            }
        }
        
        progress.current_phase = "Completed".to_string();
        info!("✅ PRs sync completed: {} PRs processed, {} chunks created",
              progress.items_processed, progress.chunks_created);
        
        Ok(progress)
    }
    
    /// Send items to chunker service
    async fn send_to_chunker(
        &self,
        source_id: Uuid,
        source_kind: SourceKind,
        items: &[SourceItem],
    ) -> Result<i32> {
        if items.is_empty() {
            return Ok(0);
        }
        
        let request = StartChunkJobRequest {
            source_id,
            source_kind,
            items: items.to_vec(),
        };
        
        let url = format!("{}/chunk/jobs", self.chunker_url);
        
        let response = self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send to chunker")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Chunker error {}: {}", status, body));
        }
        
        #[derive(Deserialize)]
        struct ChunkResponse {
            job_id: Uuid,
            accepted: usize,
        }
        
        let result: ChunkResponse = response.json().await
            .context("Failed to parse chunker response")?;
        
        info!("📦 Chunker accepted {} items (job: {})", result.accepted, result.job_id);
        
        Ok(result.accepted as i32)
    }
    
    // ========================================================================
    // Helper Methods
    // ========================================================================
    
    fn should_exclude_path(&self, path: &str, exclude_patterns: &[String]) -> bool {
        for pattern in exclude_patterns {
            if path.contains(pattern) || path.starts_with(pattern) {
                return true;
            }
        }
        false
    }
    
    fn should_include_extension(&self, path: &str, include_extensions: &[String]) -> bool {
        if include_extensions.is_empty() {
            // If no filter, include common text file extensions
            let text_extensions: HashSet<&str> = [
                "rs", "py", "js", "ts", "tsx", "jsx", "go", "java", "c", "cpp", "h", "hpp",
                "rb", "php", "swift", "kt", "scala", "cs", "fs", "clj", "ex", "exs",
                "md", "txt", "json", "yaml", "yml", "toml", "xml", "html", "css", "scss",
                "sql", "sh", "bash", "zsh", "ps1", "bat", "dockerfile", "makefile",
                "gitignore", "env", "cfg", "ini", "conf", "lock", "mod", "sum",
            ].into_iter().collect();
            
            let ext = std::path::Path::new(path)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());
            
            let filename = std::path::Path::new(path)
                .file_name()
                .and_then(|f| f.to_str())
                .map(|f| f.to_lowercase());
            
            if let Some(ext) = ext {
                return text_extensions.contains(ext.as_str());
            }
            
            // Check for extensionless files like Dockerfile, Makefile
            if let Some(name) = filename {
                return text_extensions.contains(name.as_str());
            }
            
            false
        } else {
            let ext = std::path::Path::new(path)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());
            
            if let Some(ext) = ext {
                include_extensions.iter().any(|e| e.to_lowercase() == ext)
            } else {
                false
            }
        }
    }
}

// ============================================================================
// Content Building Helpers
// ============================================================================

fn detect_language(path: &str) -> Option<String> {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())?;
    
    let lang = match ext.to_lowercase().as_str() {
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

fn build_issue_content(issue: &GitHubIssue, comments: &[GitHubComment]) -> String {
    let mut content = String::new();
    
    // Title and metadata
    content.push_str(&format!("# Issue #{}: {}\n\n", issue.number, issue.title));
    content.push_str(&format!("**State:** {}\n", issue.state));
    content.push_str(&format!("**Author:** @{}\n", issue.user.login));
    
    if !issue.labels.is_empty() {
        let labels: Vec<&str> = issue.labels.iter().map(|l| l.name.as_str()).collect();
        content.push_str(&format!("**Labels:** {}\n", labels.join(", ")));
    }
    
    if !issue.assignees.is_empty() {
        let assignees: Vec<String> = issue.assignees.iter().map(|a| format!("@{}", a.login)).collect();
        content.push_str(&format!("**Assignees:** {}\n", assignees.join(", ")));
    }
    
    content.push_str("\n---\n\n");
    
    // Body
    if let Some(ref body) = issue.body {
        content.push_str("## Description\n\n");
        content.push_str(body);
        content.push_str("\n\n");
    }
    
    // Comments
    if !comments.is_empty() {
        content.push_str("---\n\n## Comments\n\n");
        
        for comment in comments {
            content.push_str(&format!(
                "### @{} ({})\n\n{}\n\n",
                comment.user.login,
                comment.created_at.format("%Y-%m-%d %H:%M"),
                comment.body
            ));
        }
    }
    
    content
}

fn build_pr_content(
    pr: &GitHubPullRequest,
    reviews: &[GitHubReview],
    review_comments: &[GitHubReviewComment],
    comments: &[GitHubComment],
) -> String {
    let mut content = String::new();
    
    // Title and metadata
    content.push_str(&format!("# Pull Request #{}: {}\n\n", pr.number, pr.title));
    content.push_str(&format!("**State:** {}\n", pr.state));
    content.push_str(&format!("**Author:** @{}\n", pr.user.login));
    content.push_str(&format!("**Branch:** {} → {}\n", pr.head.branch_ref, pr.base.branch_ref));
    
    if let Some(merged) = pr.merged {
        if merged {
            content.push_str(&format!("**Merged:** Yes"));
            if let Some(ref merged_by) = pr.merged_by {
                content.push_str(&format!(" by @{}", merged_by.login));
            }
            content.push('\n');
        }
    }
    
    content.push_str(&format!("**Changes:** +{} -{} in {} files\n", 
                              pr.additions, pr.deletions, pr.changed_files));
    
    if !pr.labels.is_empty() {
        let labels: Vec<&str> = pr.labels.iter().map(|l| l.name.as_str()).collect();
        content.push_str(&format!("**Labels:** {}\n", labels.join(", ")));
    }
    
    content.push_str("\n---\n\n");
    
    // Body
    if let Some(ref body) = pr.body {
        content.push_str("## Description\n\n");
        content.push_str(body);
        content.push_str("\n\n");
    }
    
    // Reviews
    if !reviews.is_empty() {
        content.push_str("---\n\n## Reviews\n\n");
        
        for review in reviews {
            let state_emoji = match review.state.as_str() {
                "APPROVED" => "✅",
                "CHANGES_REQUESTED" => "❌",
                "COMMENTED" => "💬",
                "DISMISSED" => "🚫",
                _ => "📝",
            };
            
            content.push_str(&format!(
                "### {} @{} - {}\n\n",
                state_emoji,
                review.user.login,
                review.state
            ));
            
            if let Some(ref body) = review.body {
                if !body.is_empty() {
                    content.push_str(body);
                    content.push_str("\n\n");
                }
            }
        }
    }
    
    // Review comments (inline)
    if !review_comments.is_empty() {
        content.push_str("---\n\n## Code Review Comments\n\n");
        
        for comment in review_comments {
            content.push_str(&format!(
                "### @{} on `{}`\n\n```diff\n{}\n```\n\n{}\n\n",
                comment.user.login,
                comment.path,
                comment.diff_hunk,
                comment.body
            ));
        }
    }
    
    // Regular comments
    if !comments.is_empty() {
        content.push_str("---\n\n## Discussion\n\n");
        
        for comment in comments {
            content.push_str(&format!(
                "### @{} ({})\n\n{}\n\n",
                comment.user.login,
                comment.created_at.format("%Y-%m-%d %H:%M"),
                comment.body
            ));
        }
    }
    
    content
}
