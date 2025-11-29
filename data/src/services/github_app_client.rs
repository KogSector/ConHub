//! GitHub App Client for managing GitHub App installations and API access
//! 
//! This module handles:
//! - GitHub App JWT generation
//! - Installation access token management
//! - Repository listing and access
//! - Issues and PRs fetching

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::fs;
use tokio::sync::RwLock;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};
use anyhow::{Result, Context, anyhow};
use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
use chrono::{DateTime, Utc};

/// GitHub App configuration
#[derive(Debug, Clone)]
pub struct GitHubAppConfig {
    pub app_id: i64,
    pub app_slug: String,
    pub private_key_pem: String,
    pub webhook_secret: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

impl GitHubAppConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let app_id: i64 = std::env::var("GITHUB_APP_ID")
            .context("GITHUB_APP_ID not set")?
            .parse()
            .context("GITHUB_APP_ID must be a number")?;
        
        let app_slug = std::env::var("GITHUB_APP_SLUG")
            .unwrap_or_else(|_| "conhub".to_string());
        
        // Prefer inline PEM from env, but fall back to reading from a file path.
        let private_key_pem = match std::env::var("GITHUB_APP_PRIVATE_KEY_PEM")
            .or_else(|_| std::env::var("GITHUB_APP_PRIVATE_KEY"))
        {
            Ok(value) => value,
            Err(_) => {
                let path = std::env::var("GITHUB_APP_PRIVATE_KEY_PATH")
                    .context("GITHUB_APP_PRIVATE_KEY_PEM / GITHUB_APP_PRIVATE_KEY / GITHUB_APP_PRIVATE_KEY_PATH must be set")?;

                fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read GitHub App private key from path: {}", path))?
            }
        };
        
        let webhook_secret = std::env::var("GITHUB_APP_WEBHOOK_SECRET").ok();
        let client_id = std::env::var("GITHUB_APP_CLIENT_ID").ok();
        let client_secret = std::env::var("GITHUB_APP_CLIENT_SECRET").ok();
        
        Ok(Self {
            app_id,
            app_slug,
            private_key_pem,
            webhook_secret,
            client_id,
            client_secret,
        })
    }
}

/// JWT claims for GitHub App authentication
#[derive(Debug, Serialize)]
struct GitHubAppJwtClaims {
    iat: i64,
    exp: i64,
    iss: String,
}

/// Cached installation access token
#[derive(Debug, Clone)]
struct CachedToken {
    token: String,
    expires_at: DateTime<Utc>,
}

/// GitHub App Client
#[derive(Clone)]
pub struct GitHubAppClient {
    config: GitHubAppConfig,
    http_client: Client,
    /// Cache of installation_id -> access token
    token_cache: Arc<RwLock<HashMap<i64, CachedToken>>>,
}

impl GitHubAppClient {
    /// Create a new GitHub App client
    pub fn new(config: GitHubAppConfig) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            config,
            http_client,
            token_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create from environment variables
    pub fn from_env() -> Result<Self> {
        let config = GitHubAppConfig::from_env()?;
        Ok(Self::new(config))
    }
    
    /// Generate a JWT for GitHub App authentication
    pub fn create_app_jwt(&self) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Time went backwards")?
            .as_secs() as i64;
        
        let claims = GitHubAppJwtClaims {
            iat: now - 60, // 60 seconds in the past to account for clock drift
            exp: now + (10 * 60), // 10 minutes from now (max allowed)
            iss: self.config.app_id.to_string(),
        };
        
        let key = EncodingKey::from_rsa_pem(self.config.private_key_pem.as_bytes())
            .context("Invalid RSA private key")?;
        
        let header = Header::new(Algorithm::RS256);
        
        encode(&header, &claims, &key)
            .context("Failed to encode JWT")
    }
    
    /// Get the GitHub App installation URL
    pub fn get_install_url(&self, state: &str) -> String {
        format!(
            "https://github.com/apps/{}/installations/new?state={}",
            self.config.app_slug,
            state
        )
    }
    
    /// Get an installation access token (with caching)
    pub async fn get_installation_access_token(&self, installation_id: i64) -> Result<String> {
        // Check cache first
        {
            let cache = self.token_cache.read().await;
            if let Some(cached) = cache.get(&installation_id) {
                // Use token if it has at least 5 minutes of validity left
                if cached.expires_at > Utc::now() + chrono::Duration::minutes(5) {
                    debug!("Using cached installation token for {}", installation_id);
                    return Ok(cached.token.clone());
                }
            }
        }
        
        // Fetch new token
        info!("Fetching new installation access token for {}", installation_id);
        
        let jwt = self.create_app_jwt()?;
        
        let url = format!(
            "https://api.github.com/app/installations/{}/access_tokens",
            installation_id
        );
        
        let response = self.http_client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", jwt))
            .header(header::ACCEPT, "application/vnd.github+json")
            .header(header::USER_AGENT, "ConHub")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
            .context("Failed to request installation token")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error {}: {}", status, body));
        }
        
        let token_response: InstallationTokenResponse = response.json().await
            .context("Failed to parse token response")?;
        
        // Cache the token
        {
            let mut cache = self.token_cache.write().await;
            cache.insert(installation_id, CachedToken {
                token: token_response.token.clone(),
                expires_at: token_response.expires_at,
            });
        }
        
        Ok(token_response.token)
    }
    
    /// List repositories accessible to an installation
    pub async fn list_installation_repositories(
        &self,
        installation_id: i64,
        page: u32,
        per_page: u32,
    ) -> Result<ListReposResponse> {
        let token = self.get_installation_access_token(installation_id).await?;
        
        let url = format!(
            "https://api.github.com/installation/repositories?page={}&per_page={}",
            page, per_page
        );
        
        let response = self.http_client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::ACCEPT, "application/vnd.github+json")
            .header(header::USER_AGENT, "ConHub")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
            .context("Failed to list repositories")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error {}: {}", status, body));
        }
        
        response.json().await.context("Failed to parse repositories response")
    }
    
    /// Get installation details
    pub async fn get_installation(&self, installation_id: i64) -> Result<InstallationDetails> {
        let jwt = self.create_app_jwt()?;
        
        let url = format!("https://api.github.com/app/installations/{}", installation_id);
        
        let response = self.http_client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", jwt))
            .header(header::ACCEPT, "application/vnd.github+json")
            .header(header::USER_AGENT, "ConHub")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
            .context("Failed to get installation")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error {}: {}", status, body));
        }
        
        response.json().await.context("Failed to parse installation response")
    }
    
    /// Get repository tree (for code sync)
    pub async fn get_repository_tree(
        &self,
        installation_id: i64,
        owner: &str,
        repo: &str,
        branch: &str,
    ) -> Result<GitTreeResponse> {
        let token = self.get_installation_access_token(installation_id).await?;
        
        let url = format!(
            "https://api.github.com/repos/{}/{}/git/trees/{}?recursive=1",
            owner, repo, branch
        );
        
        let response = self.http_client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::ACCEPT, "application/vnd.github+json")
            .header(header::USER_AGENT, "ConHub")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
            .context("Failed to get repository tree")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error {}: {}", status, body));
        }
        
        response.json().await.context("Failed to parse tree response")
    }
    
    /// Get file content (blob)
    pub async fn get_blob_content(
        &self,
        installation_id: i64,
        owner: &str,
        repo: &str,
        sha: &str,
    ) -> Result<BlobContent> {
        let token = self.get_installation_access_token(installation_id).await?;
        
        let url = format!(
            "https://api.github.com/repos/{}/{}/git/blobs/{}",
            owner, repo, sha
        );
        
        let response = self.http_client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::ACCEPT, "application/vnd.github+json")
            .header(header::USER_AGENT, "ConHub")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
            .context("Failed to get blob")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error {}: {}", status, body));
        }
        
        response.json().await.context("Failed to parse blob response")
    }
    
    /// List repository issues
    pub async fn list_issues(
        &self,
        installation_id: i64,
        owner: &str,
        repo: &str,
        state: &str,
        since: Option<DateTime<Utc>>,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<GitHubIssue>> {
        let token = self.get_installation_access_token(installation_id).await?;
        
        let mut url = format!(
            "https://api.github.com/repos/{}/{}/issues?state={}&page={}&per_page={}&sort=updated&direction=desc",
            owner, repo, state, page, per_page
        );
        
        if let Some(since_date) = since {
            url.push_str(&format!("&since={}", since_date.to_rfc3339()));
        }
        
        let response = self.http_client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::ACCEPT, "application/vnd.github+json")
            .header(header::USER_AGENT, "ConHub")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
            .context("Failed to list issues")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error {}: {}", status, body));
        }
        
        response.json().await.context("Failed to parse issues response")
    }
    
    /// Get issue comments
    pub async fn get_issue_comments(
        &self,
        installation_id: i64,
        owner: &str,
        repo: &str,
        issue_number: i64,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<GitHubComment>> {
        let token = self.get_installation_access_token(installation_id).await?;
        
        let url = format!(
            "https://api.github.com/repos/{}/{}/issues/{}/comments?page={}&per_page={}",
            owner, repo, issue_number, page, per_page
        );
        
        let response = self.http_client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::ACCEPT, "application/vnd.github+json")
            .header(header::USER_AGENT, "ConHub")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
            .context("Failed to get issue comments")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error {}: {}", status, body));
        }
        
        response.json().await.context("Failed to parse comments response")
    }
    
    /// List pull requests
    pub async fn list_pull_requests(
        &self,
        installation_id: i64,
        owner: &str,
        repo: &str,
        state: &str,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<GitHubPullRequest>> {
        let token = self.get_installation_access_token(installation_id).await?;
        
        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls?state={}&page={}&per_page={}&sort=updated&direction=desc",
            owner, repo, state, page, per_page
        );
        
        let response = self.http_client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::ACCEPT, "application/vnd.github+json")
            .header(header::USER_AGENT, "ConHub")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
            .context("Failed to list pull requests")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error {}: {}", status, body));
        }
        
        response.json().await.context("Failed to parse PRs response")
    }
    
    /// Get PR reviews
    pub async fn get_pr_reviews(
        &self,
        installation_id: i64,
        owner: &str,
        repo: &str,
        pr_number: i64,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<GitHubReview>> {
        let token = self.get_installation_access_token(installation_id).await?;
        
        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}/reviews?page={}&per_page={}",
            owner, repo, pr_number, page, per_page
        );
        
        let response = self.http_client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::ACCEPT, "application/vnd.github+json")
            .header(header::USER_AGENT, "ConHub")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
            .context("Failed to get PR reviews")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error {}: {}", status, body));
        }
        
        response.json().await.context("Failed to parse reviews response")
    }
    
    /// Get PR review comments (inline comments)
    pub async fn get_pr_review_comments(
        &self,
        installation_id: i64,
        owner: &str,
        repo: &str,
        pr_number: i64,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<GitHubReviewComment>> {
        let token = self.get_installation_access_token(installation_id).await?;
        
        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}/comments?page={}&per_page={}",
            owner, repo, pr_number, page, per_page
        );
        
        let response = self.http_client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::ACCEPT, "application/vnd.github+json")
            .header(header::USER_AGENT, "ConHub")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
            .context("Failed to get PR review comments")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error {}: {}", status, body));
        }
        
        response.json().await.context("Failed to parse review comments response")
    }
    
    /// Get branch info including latest commit
    pub async fn get_branch(
        &self,
        installation_id: i64,
        owner: &str,
        repo: &str,
        branch: &str,
    ) -> Result<GitHubBranchInfo> {
        let token = self.get_installation_access_token(installation_id).await?;
        
        let url = format!(
            "https://api.github.com/repos/{}/{}/branches/{}",
            owner, repo, branch
        );
        
        let response = self.http_client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::ACCEPT, "application/vnd.github+json")
            .header(header::USER_AGENT, "ConHub")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
            .context("Failed to get branch")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("GitHub API error {}: {}", status, body));
        }
        
        response.json().await.context("Failed to parse branch response")
    }
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct InstallationTokenResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub permissions: Option<serde_json::Value>,
    pub repository_selection: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListReposResponse {
    pub total_count: i64,
    pub repositories: Vec<GitHubRepository>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepository {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub owner: GitHubOwner,
    pub private: bool,
    pub description: Option<String>,
    pub html_url: String,
    pub clone_url: String,
    pub default_branch: String,
    pub language: Option<String>,
    pub topics: Option<Vec<String>>,
    pub visibility: Option<String>,
    pub pushed_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubOwner {
    pub login: String,
    pub id: i64,
    #[serde(rename = "type")]
    pub owner_type: String,
}

#[derive(Debug, Deserialize)]
pub struct InstallationDetails {
    pub id: i64,
    pub account: GitHubAccount,
    pub app_id: i64,
    pub app_slug: Option<String>,
    pub target_type: String,
    pub permissions: serde_json::Value,
    pub events: Vec<String>,
    pub repository_selection: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub suspended_at: Option<DateTime<Utc>>,
    pub suspended_by: Option<GitHubAccount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAccount {
    pub login: String,
    pub id: i64,
    #[serde(rename = "type")]
    pub account_type: String,
}

#[derive(Debug, Deserialize)]
pub struct GitTreeResponse {
    pub sha: String,
    pub url: String,
    pub tree: Vec<GitTreeEntry>,
    pub truncated: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitTreeEntry {
    pub path: String,
    pub mode: String,
    #[serde(rename = "type")]
    pub entry_type: String, // blob or tree
    pub sha: String,
    pub size: Option<i64>,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BlobContent {
    pub sha: String,
    pub size: i64,
    pub content: String,
    pub encoding: String, // base64
}

impl BlobContent {
    /// Decode base64 content to bytes
    pub fn decode_content(&self) -> Result<Vec<u8>> {
        use base64::{Engine as _, engine::general_purpose};
        let cleaned = self.content.replace('\n', "");
        general_purpose::STANDARD.decode(&cleaned)
            .context("Failed to decode base64 content")
    }
    
    /// Decode content to UTF-8 string
    pub fn decode_to_string(&self) -> Result<String> {
        let bytes = self.decode_content()?;
        String::from_utf8(bytes).context("Content is not valid UTF-8")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssue {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub user: GitHubUser,
    pub labels: Vec<GitHubLabel>,
    pub assignees: Vec<GitHubUser>,
    pub milestone: Option<GitHubMilestone>,
    pub comments: i64,
    pub html_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub pull_request: Option<serde_json::Value>, // Present if this is a PR
}

impl GitHubIssue {
    /// Check if this is actually a pull request
    pub fn is_pull_request(&self) -> bool {
        self.pull_request.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: i64,
    pub html_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubLabel {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubMilestone {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubComment {
    pub id: i64,
    pub body: String,
    pub user: GitHubUser,
    pub html_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubPullRequest {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub user: GitHubUser,
    pub labels: Vec<GitHubLabel>,
    pub assignees: Vec<GitHubUser>,
    pub milestone: Option<GitHubMilestone>,
    pub html_url: String,
    pub head: GitHubPRBranch,
    pub base: GitHubPRBranch,
    pub merged: Option<bool>,
    pub merged_at: Option<DateTime<Utc>>,
    pub merged_by: Option<GitHubUser>,
    pub comments: i64,
    pub review_comments: i64,
    pub commits: i64,
    pub additions: i64,
    pub deletions: i64,
    pub changed_files: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubPRBranch {
    pub label: String,
    #[serde(rename = "ref")]
    pub branch_ref: String,
    pub sha: String,
    pub user: Option<GitHubUser>,
    pub repo: Option<GitHubRepository>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubReview {
    pub id: i64,
    pub user: GitHubUser,
    pub body: Option<String>,
    pub state: String, // APPROVED, CHANGES_REQUESTED, COMMENTED, DISMISSED, PENDING
    pub html_url: String,
    pub submitted_at: Option<DateTime<Utc>>,
    pub commit_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubReviewComment {
    pub id: i64,
    pub body: String,
    pub user: GitHubUser,
    pub path: String,
    pub position: Option<i64>,
    pub original_position: Option<i64>,
    pub diff_hunk: String,
    pub commit_id: String,
    pub original_commit_id: String,
    pub html_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubBranchInfo {
    pub name: String,
    pub commit: GitHubBranchCommit,
    pub protected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubBranchCommit {
    pub sha: String,
    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_blob_decode() {
        let blob = BlobContent {
            sha: "abc123".to_string(),
            size: 12,
            content: "SGVsbG8gV29ybGQh".to_string(), // "Hello World!" in base64
            encoding: "base64".to_string(),
        };
        
        let decoded = blob.decode_to_string().unwrap();
        assert_eq!(decoded, "Hello World!");
    }
}
