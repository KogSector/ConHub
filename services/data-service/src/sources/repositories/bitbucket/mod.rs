use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::info;
use base64::Engine;

use crate::sources::core::{DataSourceConnector, DataSource, Document, Repository, SyncResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitbucketUser {
    pub username: String,
    pub display_name: String,
    pub uuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitbucketRepo {
    pub uuid: String,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub is_private: bool,
    pub links: BitbucketLinks,
    pub updated_on: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitbucketLinks {
    pub html: BitbucketLink,
    pub clone: Vec<BitbucketCloneLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitbucketLink {
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitbucketCloneLink {
    pub name: String,
    pub href: String,
}

pub struct BitbucketConnector {
    client: Client,
    username: Option<String>,
    app_password: Option<String>,
}

impl BitbucketConnector {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            username: None,
            app_password: None,
        }
    }

    fn get_auth_header(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let username = self.username.as_ref().ok_or("Bitbucket not connected")?;
        let password = self.app_password.as_ref().ok_or("Bitbucket not connected")?;
        let credentials = base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", username, password));
        Ok(format!("Basic {}", credentials))
    }
}

#[async_trait]
impl DataSourceConnector for BitbucketConnector {
    async fn validate(&self, credentials: &HashMap<String, String>) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let username = credentials.get("username")
            .ok_or("Bitbucket username is required")?;
        let app_password = credentials.get("appPassword")
            .ok_or("Bitbucket app password is required")?;

        let auth = base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", username, app_password));
        let response = self.client
            .get("https://api.bitbucket.org/2.0/user")
            .header("Authorization", format!("Basic {}", auth))
            .send()
            .await?;

        if response.status().is_success() {
            let user: BitbucketUser = response.json().await?;
            info!("Bitbucket credentials validated successfully for user: {}", user.username);
            Ok(true)
        } else {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Bitbucket authentication failed ({}): {}", status, error_text).into())
        }
    }

    async fn connect(&mut self, credentials: &HashMap<String, String>, _config: &Value) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let username = credentials.get("username")
            .ok_or("Bitbucket username is required")?;
        let app_password = credentials.get("appPassword")
            .ok_or("Bitbucket app password is required")?;

        self.username = Some(username.clone());
        self.app_password = Some(app_password.clone());

        info!("Bitbucket connected successfully");
        Ok(true)
    }

    #[allow(dead_code)]
    async fn sync(&self, data_source: &DataSource) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let auth_header = self.get_auth_header()?;
        let mut documents = Vec::new();
        let mut repositories = Vec::new();

        let empty_vec = vec![];
        let repo_list = data_source.config.get("repositories")
            .and_then(|r| r.as_array())
            .unwrap_or(&empty_vec);

        for repo_value in repo_list {
            if let Some(repo_name) = repo_value.as_str() {
                let url = format!("https://api.bitbucket.org/2.0/repositories/{}", repo_name);
                let response = self.client
                    .get(&url)
                    .header("Authorization", &auth_header)
                    .send()
                    .await?;

                if let Ok(repo_data) = response.json::<BitbucketRepo>().await {
                    repositories.push(Repository {
                        id: repo_data.uuid.clone(),
                        name: repo_data.name.clone(),
                        full_name: repo_data.full_name.clone(),
                        description: repo_data.description.clone(),
                        url: repo_data.links.html.href.clone(),
                        private: repo_data.is_private,
                        metadata: json!({
                            "source": "bitbucket",
                            "clone_urls": repo_data.links.clone,
                            "updated_on": repo_data.updated_on
                        }),
                    });

                    
                    if data_source.config.get("includeReadme").and_then(|v| v.as_bool()).unwrap_or(false) {
                        let readme_url = format!("https://api.bitbucket.org/2.0/repositories/{}/src/main/README.md", repo_name);
                        if let Ok(readme_response) = self.client
                            .get(&readme_url)
                            .header("Authorization", &auth_header)
                            .send()
                            .await
                        {
                            if let Ok(readme_content) = readme_response.text().await {
                                documents.push(Document {
                                    id: format!("bitbucket-{}-readme", repo_name),
                                    title: format!("{} README", repo_name),
                                    content: readme_content,
                                    metadata: json!({
                                        "source": "bitbucket",
                                        "repository": repo_name,
                                        "path": "README.md",
                                        "type": "readme"
                                    }),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(SyncResult { documents, repositories })
    }

    async fn fetch_branches(&self, repo_url: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let auth_header = self.get_auth_header()?;
        
        
        let repo_name = if repo_url.contains("bitbucket.org") {
            repo_url.split("bitbucket.org/").nth(1)
                .and_then(|s| s.split(".git").next())
                .ok_or("Invalid Bitbucket URL format")?
        } else {
            repo_url
        };

        let url = format!("https://api.bitbucket.org/2.0/repositories/{}/refs/branches", repo_name);
        let response = self.client
            .get(&url)
            .header("Authorization", &auth_header)
            .send()
            .await?;

        if response.status().is_success() {
            let response_data: Value = response.json().await?;
            let empty_vec = vec![];
            let branches = response_data.get("values")
                .and_then(|v| v.as_array())
                .unwrap_or(&empty_vec);

            let branch_names: Vec<String> = branches
                .iter()
                .filter_map(|b| b.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                .collect();

            Ok(branch_names)
        } else {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to fetch branches ({}): {}", status, error_text).into())
        }
    }
}

impl Default for BitbucketConnector {
    fn default() -> Self {
        Self::new()
    }
}