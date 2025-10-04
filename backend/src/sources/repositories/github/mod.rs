use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{info, warn};
use base64::Engine;

use crate::sources::core::{DataSourceConnector, DataSource, Document, Repository, SyncResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    pub node_id: String,
    pub avatar_url: String,
    pub gravatar_id: Option<String>,
    pub url: String,
    pub html_url: String,
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepo {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub private: bool,
    pub html_url: String,
    pub clone_url: String,
    pub ssh_url: String,
    pub default_branch: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubBranch {
    pub name: String,
    pub commit: GitHubCommit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubCommit {
    pub sha: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubContent {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub size: u64,
    pub url: String,
    pub html_url: String,
    pub git_url: String,
    pub download_url: Option<String>,
    #[serde(rename = "type")]
    pub content_type: String,
    pub content: Option<String>,
    pub encoding: Option<String>,
}

pub struct GitHubConnector {
    client: Client,
    token: Option<String>,
}

impl GitHubConnector {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            token: None,
        }
    }

    fn get_error_message(&self, status: u16, message: &str) -> String {
        match status {
            401 => "GitHub token is invalid or expired. Please check your access token.".to_string(),
            403 => "GitHub token lacks required permissions. For public repositories, use 'public_repo' scope. For private repositories, use 'repo' scope.".to_string(),
            404 => "Repository not found or access denied. Check repository URL and permissions.".to_string(),
            _ if message.contains("rate limit") => "GitHub API rate limit exceeded. Please wait before trying again.".to_string(),
            _ => format!("GitHub API error: {}", message),
        }
    }

    async fn test_repository_access(&self, repo_name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let token = self.token.as_ref().ok_or("GitHub not connected")?;
        let parts: Vec<&str> = repo_name.split('/').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid repository format: {}. Expected format: owner/repo", repo_name).into());
        }

        let (owner, repo) = (parts[0], parts[1]);
        let url = format!("https://api.github.com/repos/{}/{}", owner, repo);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "ConHub/1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            return Err(self.get_error_message(status, &error_text).into());
        }

        info!("Repository access verified for: {}", repo_name);
        Ok(())
    }
}

#[async_trait]
impl DataSourceConnector for GitHubConnector {
    async fn validate(&self, credentials: &HashMap<String, String>) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let token = credentials.get("accessToken")
            .ok_or("GitHub access token is required")?;

        let response = self.client
            .get("https://api.github.com/user")
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "ConHub/1.0")
            .send()
            .await?;

        if response.status().is_success() {
            let user: GitHubUser = response.json().await?;
            info!("GitHub token validated successfully for user: {}", user.login);
            Ok(true)
        } else {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            Err(self.get_error_message(status, &error_text).into())
        }
    }

    async fn connect(&mut self, credentials: &HashMap<String, String>, config: &Value) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let token = credentials.get("accessToken")
            .ok_or("GitHub access token is required")?;

        self.token = Some(token.clone());

        // Test repository access if repositories are specified
        if let Some(repos) = config.get("repositories").and_then(|r| r.as_array()) {
            if let Some(first_repo) = repos.first().and_then(|r| r.as_str()) {
                self.test_repository_access(first_repo).await?;
            }
        }

        info!("GitHub connected successfully");
        Ok(true)
    }

    #[allow(dead_code)]
    async fn sync(&self, data_source: &DataSource) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let token = self.token.as_ref().ok_or("GitHub not connected")?;
        let mut documents = Vec::new();
        let mut repositories = Vec::new();

        let empty_vec = vec![];
        let repo_list = data_source.config.get("repositories")
            .and_then(|r| r.as_array())
            .unwrap_or(&empty_vec);

        for repo_value in repo_list {
            if let Some(repo_name) = repo_value.as_str() {
                let parts: Vec<&str> = repo_name.split('/').collect();
                if parts.len() != 2 {
                    warn!("Invalid repository format: {}", repo_name);
                    continue;
                }

                let (owner, repo) = (parts[0], parts[1]);
                
                // Get repository info
                let repo_url = format!("https://api.github.com/repos/{}/{}", owner, repo);
                let response = self.client
                    .get(&repo_url)
                    .header("Authorization", format!("Bearer {}", token))
                    .header("User-Agent", "ConHub/1.0")
                    .send()
                    .await?;

                if let Ok(repo_data) = response.json::<GitHubRepo>().await {
                    repositories.push(Repository {
                        id: repo_data.id.to_string(),
                        name: repo_data.name.clone(),
                        full_name: repo_data.full_name.clone(),
                        description: repo_data.description.clone(),
                        url: repo_data.html_url.clone(),
                        private: repo_data.private,
                        metadata: json!({
                            "source": "github",
                            "clone_url": repo_data.clone_url,
                            "ssh_url": repo_data.ssh_url,
                            "default_branch": repo_data.default_branch,
                            "updated_at": repo_data.updated_at
                        }),
                    });

                    // Get README if requested
                    if data_source.config.get("includeReadme").and_then(|v| v.as_bool()).unwrap_or(false) {
                        let readme_url = format!("https://api.github.com/repos/{}/{}/readme", owner, repo);
                        if let Ok(readme_response) = self.client
                            .get(&readme_url)
                            .header("Authorization", format!("Bearer {}", token))
                            .header("User-Agent", "ConHub/1.0")
                            .send()
                            .await
                        {
                            if let Ok(readme_content) = readme_response.json::<GitHubContent>().await {
                                if let Some(content) = readme_content.content {
                                    let decoded_content = base64::engine::general_purpose::STANDARD.decode(&content.replace('\n', ""))
                                        .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
                                        .unwrap_or_default();

                                    documents.push(Document {
                                        id: format!("github-{}-readme", repo_name),
                                        title: format!("{} README", repo_name),
                                        content: decoded_content,
                                        metadata: json!({
                                            "source": "github",
                                            "repository": repo_name,
                                            "path": readme_content.path,
                                            "type": "readme",
                                            "url": readme_content.html_url
                                        }),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(SyncResult { documents, repositories })
    }

    async fn fetch_branches(&self, repo_url: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let token = self.token.as_ref().ok_or("GitHub not connected")?;
        
        // Extract owner/repo from URL
        let repo_name = if repo_url.contains("github.com") {
            repo_url.split("github.com/").nth(1)
                .and_then(|s| s.split(".git").next())
                .ok_or("Invalid GitHub URL format")?
        } else {
            repo_url
        };

        let parts: Vec<&str> = repo_name.split('/').collect();
        if parts.len() != 2 {
            return Err("Invalid repository format. Expected: owner/repo".into());
        }

        let (owner, repo) = (parts[0], parts[1]);
        let url = format!("https://api.github.com/repos/{}/{}/branches", owner, repo);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "ConHub/1.0")
            .send()
            .await?;

        if response.status().is_success() {
            let branches: Vec<GitHubBranch> = response.json().await?;
            Ok(branches.into_iter().map(|b| b.name).collect())
        } else {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            Err(self.get_error_message(status, &error_text).into())
        }
    }
}

impl Default for GitHubConnector {
    fn default() -> Self {
        Self::new()
    }
}