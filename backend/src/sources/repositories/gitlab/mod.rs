use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{info, warn, debug};
use base64::Engine;

use crate::sources::core::{DataSourceConnector, DataSource, Document, Repository, SyncResult};

/// GitLab project structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabProject {
    pub id: u64,
    pub name: String,
    pub name_with_namespace: String,
    pub description: Option<String>,
    pub visibility: String,
    pub http_url_to_repo: String,
    pub ssh_url_to_repo: String,
    pub default_branch: String,
    pub last_activity_at: String,
    pub namespace: GitLabNamespace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabNamespace {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabBranch {
    pub name: String,
    pub commit: GitLabCommit,
    pub protected: bool,
    pub merged: bool,
    pub default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabCommit {
    pub id: String,
    pub title: String,
    pub author_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabFile {
    pub file_name: String,
    pub file_path: String,
    pub size: u64,
    pub encoding: String,
    pub content: String,
    pub content_sha256: String,
    pub ref_field: String,
    pub blob_id: String,
}

pub struct GitLabConnector {
    client: Client,
    token: Option<String>,
    instance_url: String,
}

impl GitLabConnector {
    pub fn new() -> Self {
        let instance_url = std::env::var("GITLAB_INSTANCE_URL")
            .unwrap_or_else(|_| "https://gitlab.com".to_string());

        Self {
            client: Client::new(),
            token: None,
            instance_url,
        }
    }

    pub fn with_instance_url(instance_url: String) -> Self {
        Self {
            client: Client::new(),
            token: None,
            instance_url,
        }
    }

    fn get_error_message(&self, status: u16, message: &str) -> String {
        match status {
            401 => "GitLab token invalid or expired. Please reconnect with a valid token.".to_string(),
            403 => "Insufficient permissions. Token needs 'api' or 'read_api' scope.".to_string(),
            404 => "Project not found or no access. Check the project path and your permissions.".to_string(),
            429 => "GitLab API rate limit exceeded. Please wait and try again.".to_string(),
            _ => format!("GitLab API error: {}", message),
        }
    }

    async fn test_project_access(&self, project_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let token = self.token.as_ref().ok_or("GitLab not connected")?;

        // URL encode the project path (e.g., "group/project" -> "group%2Fproject")
        let encoded_path = urlencoding::encode(project_path);
        let url = format!("{}/api/v4/projects/{}", self.instance_url, encoded_path);

        let response = self.client
            .get(&url)
            .header("PRIVATE-TOKEN", token)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            return Err(self.get_error_message(status, &error_text).into());
        }

        info!("GitLab project access verified for: {}", project_path);
        Ok(())
    }

    async fn get_project(&self, project_path: &str) -> Result<GitLabProject, Box<dyn std::error::Error + Send + Sync>> {
        let token = self.token.as_ref().ok_or("GitLab not connected")?;
        let encoded_path = urlencoding::encode(project_path);
        let url = format!("{}/api/v4/projects/{}", self.instance_url, encoded_path);

        let response = self.client
            .get(&url)
            .header("PRIVATE-TOKEN", token)
            .send()
            .await?;

        if response.status().is_success() {
            let project: GitLabProject = response.json().await?;
            Ok(project)
        } else {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            Err(self.get_error_message(status, &error_text).into())
        }
    }

    async fn get_file_content(
        &self,
        project_id: u64,
        file_path: &str,
        ref_name: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let token = self.token.as_ref().ok_or("GitLab not connected")?;
        let encoded_path = urlencoding::encode(file_path);
        let url = format!(
            "{}/api/v4/projects/{}/repository/files/{}?ref={}",
            self.instance_url, project_id, encoded_path, ref_name
        );

        let response = self.client
            .get(&url)
            .header("PRIVATE-TOKEN", token)
            .send()
            .await?;

        if response.status().is_success() {
            let file: GitLabFile = response.json().await?;

            // Decode base64 content
            let decoded = base64::engine::general_purpose::STANDARD
                .decode(&file.content.replace('\n', ""))
                .map_err(|e| format!("Failed to decode base64 content: {}", e))?;

            let content = String::from_utf8_lossy(&decoded).to_string();
            Ok(content)
        } else {
            debug!("Failed to fetch file {}: {}", file_path, response.status());
            Err(format!("Failed to fetch file: {}", file_path).into())
        }
    }
}

#[async_trait]
impl DataSourceConnector for GitLabConnector {
    async fn validate(&self, credentials: &HashMap<String, String>) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let token = credentials.get("accessToken")
            .or_else(|| credentials.get("privateToken"))
            .ok_or("GitLab access token or private token is required")?;

        // Get custom instance URL if provided
        let instance_url = credentials.get("instanceUrl")
            .map(|s| s.as_str())
            .unwrap_or(&self.instance_url);

        let url = format!("{}/api/v4/user", instance_url);

        let response = self.client
            .get(&url)
            .header("PRIVATE-TOKEN", token)
            .send()
            .await?;

        if response.status().is_success() {
            let user: Value = response.json().await?;
            let username = user.get("username").and_then(|u| u.as_str()).unwrap_or("unknown");
            info!("GitLab token validated successfully for user: {}", username);
            Ok(true)
        } else {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            Err(self.get_error_message(status, &error_text).into())
        }
    }

    async fn connect(&mut self, credentials: &HashMap<String, String>, config: &Value) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let token = credentials.get("accessToken")
            .or_else(|| credentials.get("privateToken"))
            .ok_or("GitLab access token or private token is required")?;

        self.token = Some(token.clone());

        // Set custom instance URL if provided
        if let Some(instance_url) = credentials.get("instanceUrl") {
            self.instance_url = instance_url.clone();
        } else if let Some(instance_url) = config.get("instanceUrl").and_then(|u| u.as_str()) {
            self.instance_url = instance_url.to_string();
        }

        // Test repository access if configured
        if let Some(repos) = config.get("repositories").and_then(|r| r.as_array()) {
            if let Some(first_repo) = repos.first().and_then(|r| r.as_str()) {
                self.test_project_access(first_repo).await?;
            }
        }

        info!("GitLab connected successfully to instance: {}", self.instance_url);
        Ok(true)
    }

    async fn sync(&self, data_source: &DataSource) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let token = self.token.as_ref().ok_or("GitLab not connected")?;
        let mut documents = Vec::new();
        let mut repositories = Vec::new();

        let empty_vec = vec![];
        let repo_list = data_source.config.get("repositories")
            .and_then(|r| r.as_array())
            .unwrap_or(&empty_vec);

        for repo_value in repo_list {
            if let Some(project_path) = repo_value.as_str() {
                match self.get_project(project_path).await {
                    Ok(project) => {
                        repositories.push(Repository {
                            id: project.id.to_string(),
                            name: project.name.clone(),
                            full_name: project.name_with_namespace.clone(),
                            description: project.description.clone(),
                            url: project.http_url_to_repo.clone(),
                            private: project.visibility == "private",
                            metadata: json!({
                                "source": "gitlab",
                                "instance_url": self.instance_url,
                                "project_id": project.id,
                                "namespace": format!("{}/{}", project.namespace.path, project.name),
                                "visibility": project.visibility,
                                "default_branch": project.default_branch,
                                "last_activity_at": project.last_activity_at,
                                "ssh_url": project.ssh_url_to_repo,
                                "http_url": project.http_url_to_repo
                            }),
                        });

                        // Fetch README if configured
                        if data_source.config.get("includeReadme").and_then(|v| v.as_bool()).unwrap_or(false) {
                            // Try common README filenames
                            let readme_names = vec!["README.md", "README.rst", "README.txt", "README"];

                            for readme_name in readme_names {
                                if let Ok(content) = self.get_file_content(
                                    project.id,
                                    readme_name,
                                    &project.default_branch
                                ).await {
                                    documents.push(Document {
                                        id: format!("gitlab-{}-readme", project_path),
                                        title: format!("{} README", project.name_with_namespace),
                                        content,
                                        metadata: json!({
                                            "source": "gitlab",
                                            "project": project_path,
                                            "path": readme_name,
                                            "type": "readme",
                                            "url": format!("{}/{}", project.http_url_to_repo, readme_name)
                                        }),
                                    });
                                    break; // Found README, stop trying other names
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to fetch GitLab project {}: {}", project_path, e);
                    }
                }
            }
        }

        Ok(SyncResult { documents, repositories })
    }

    async fn fetch_branches(&self, repo_url: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let token = self.token.as_ref().ok_or("GitLab not connected")?;

        // Extract project path from URL or use as-is
        let project_path = if repo_url.contains("gitlab") {
            repo_url.split("gitlab.com/").nth(1)
                .or_else(|| repo_url.split(&self.instance_url).nth(1))
                .and_then(|s| s.trim_start_matches('/').split(".git").next())
                .ok_or("Invalid GitLab URL format")?
        } else {
            repo_url
        };

        let encoded_path = urlencoding::encode(project_path);
        let url = format!("{}/api/v4/projects/{}/repository/branches", self.instance_url, encoded_path);

        let response = self.client
            .get(&url)
            .header("PRIVATE-TOKEN", token)
            .send()
            .await?;

        if response.status().is_success() {
            let branches: Vec<GitLabBranch> = response.json().await?;
            Ok(branches.into_iter().map(|b| b.name).collect())
        } else {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            Err(self.get_error_message(status, &error_text).into())
        }
    }
}

impl Default for GitLabConnector {
    fn default() -> Self {
        Self::new()
    }
}
