use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{info, warn, debug};

use crate::sources::core::{DataSourceConnector, DataSource, Document, SyncResult};

/// Dropbox file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropboxFile {
    pub id: String,
    pub name: String,
    pub path_display: String,
    pub path_lower: String,
    pub client_modified: String,
    pub server_modified: String,
    pub size: u64,
    pub rev: String,
    pub content_hash: String,
}

/// Dropbox folder metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropboxFolder {
    pub id: String,
    pub name: String,
    pub path_display: String,
    pub path_lower: String,
}

/// Dropbox list folder response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFolderResponse {
    pub entries: Vec<DropboxEntry>,
    pub cursor: String,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = ".tag")]
pub enum DropboxEntry {
    #[serde(rename = "file")]
    File(DropboxFile),
    #[serde(rename = "folder")]
    Folder(DropboxFolder),
    #[serde(rename = "deleted")]
    Deleted { path_display: String },
}

pub struct DropboxConnector {
    client: Client,
    access_token: Option<String>,
}

impl DropboxConnector {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            access_token: None,
        }
    }

    fn get_error_message(&self, status: u16, message: &str) -> String {
        match status {
            401 => "Dropbox token expired. Please reconnect.".to_string(),
            409 => "Dropbox API conflict. Path may not exist or is invalid.".to_string(),
            429 => "Dropbox API rate limit exceeded. Please wait and try again.".to_string(),
            503 => "Dropbox service unavailable. Please try again later.".to_string(),
            _ => format!("Dropbox API error: {}", message),
        }
    }

    async fn list_folder(
        &self,
        path: &str,
        recursive: bool,
    ) -> Result<ListFolderResponse, Box<dyn std::error::Error + Send + Sync>> {
        let token = self.access_token.as_ref().ok_or("Dropbox not connected")?;

        let body = json!({
            "path": path,
            "recursive": recursive,
            "include_mounted_folders": true,
            "include_non_downloadable_files": false
        });

        let response = self.client
            .post("https://api.dropboxapi.com/2/files/list_folder")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            let list_response: ListFolderResponse = response.json().await?;
            Ok(list_response)
        } else {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            Err(self.get_error_message(status, &error_text).into())
        }
    }

    async fn list_folder_continue(
        &self,
        cursor: &str,
    ) -> Result<ListFolderResponse, Box<dyn std::error::Error + Send + Sync>> {
        let token = self.access_token.as_ref().ok_or("Dropbox not connected")?;

        let body = json!({
            "cursor": cursor
        });

        let response = self.client
            .post("https://api.dropboxapi.com/2/files/list_folder/continue")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            let list_response: ListFolderResponse = response.json().await?;
            Ok(list_response)
        } else {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            Err(self.get_error_message(status, &error_text).into())
        }
    }

    async fn download_file(&self, path: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let token = self.access_token.as_ref().ok_or("Dropbox not connected")?;

        let arg = json!({
            "path": path
        });

        let response = self.client
            .post("https://content.dropboxapi.com/2/files/download")
            .header("Authorization", format!("Bearer {}", token))
            .header("Dropbox-API-Arg", arg.to_string())
            .send()
            .await?;

        if response.status().is_success() {
            let content = response.text().await?;
            Ok(content)
        } else {
            debug!("Failed to download file {}: {}", path, response.status());
            Err(format!("Failed to download file: {}", path).into())
        }
    }

    fn should_include_file(&self, file: &DropboxFile, config: &Value) -> bool {
        // Check file types filter
        if let Some(file_types) = config.get("fileTypes").and_then(|ft| ft.as_array()) {
            let extension = file.name.split('.').last().unwrap_or("");
            let matches = file_types.iter().any(|ft| {
                if let Some(ft_str) = ft.as_str() {
                    ft_str.trim_start_matches('.').eq_ignore_ascii_case(extension)
                } else {
                    false
                }
            });

            if !matches {
                return false;
            }
        }

        // Check max file size
        if let Some(max_size) = config.get("maxFileSize").and_then(|ms| ms.as_u64()) {
            if file.size > max_size {
                return false;
            }
        }

        // Check exclude patterns
        if let Some(exclude_patterns) = config.get("excludePatterns").and_then(|ep| ep.as_array()) {
            for pattern in exclude_patterns {
                if let Some(pattern_str) = pattern.as_str() {
                    // Simple wildcard matching
                    if pattern_str.starts_with('*') && file.name.ends_with(pattern_str.trim_start_matches('*')) {
                        return false;
                    }
                    if pattern_str.ends_with('*') && file.name.starts_with(pattern_str.trim_end_matches('*')) {
                        return false;
                    }
                    if file.name == pattern_str {
                        return false;
                    }
                }
            }
        }

        true
    }

    async fn fetch_all_entries(&self, path: &str, recursive: bool) -> Result<Vec<DropboxEntry>, Box<dyn std::error::Error + Send + Sync>> {
        let mut all_entries = Vec::new();
        let mut response = self.list_folder(path, recursive).await?;

        all_entries.extend(response.entries);

        // Handle pagination
        while response.has_more {
            response = self.list_folder_continue(&response.cursor).await?;
            all_entries.extend(response.entries);
        }

        Ok(all_entries)
    }
}

#[async_trait]
impl DataSourceConnector for DropboxConnector {
    async fn validate(&self, credentials: &HashMap<String, String>) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let access_token = credentials.get("accessToken")
            .ok_or("Dropbox access token is required")?;

        // Test token by getting current account info
        let response = self.client
            .post("https://api.dropboxapi.com/2/users/get_current_account")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        if response.status().is_success() {
            let account: Value = response.json().await?;
            let email = account.get("email").and_then(|e| e.as_str()).unwrap_or("unknown");
            info!("Dropbox token validated successfully for account: {}", email);
            Ok(true)
        } else {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            Err(self.get_error_message(status, &error_text).into())
        }
    }

    async fn connect(&mut self, credentials: &HashMap<String, String>, _config: &Value) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let access_token = credentials.get("accessToken")
            .ok_or("Dropbox access token is required")?;

        self.access_token = Some(access_token.clone());

        info!("Dropbox connected successfully");
        Ok(true)
    }

    async fn sync(&self, data_source: &DataSource) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let token = self.access_token.as_ref().ok_or("Dropbox not connected")?;
        let mut documents = Vec::new();

        let empty_vec = vec![];
        let folder_paths = data_source.config.get("folderPaths")
            .and_then(|fp| fp.as_array())
            .unwrap_or(&empty_vec);

        let recursive = data_source.config.get("recursive")
            .and_then(|r| r.as_bool())
            .unwrap_or(true);

        for folder_path in folder_paths {
            if let Some(path_str) = folder_path.as_str() {
                match self.fetch_all_entries(path_str, recursive).await {
                    Ok(entries) => {
                        info!("Fetched {} entries from Dropbox path: {}", entries.len(), path_str);

                        for entry in entries {
                            if let DropboxEntry::File(file) = entry {
                                // Apply filters
                                if !self.should_include_file(&file, &data_source.config) {
                                    debug!("Skipping file (filtered): {}", file.name);
                                    continue;
                                }

                                // Determine if we should download content
                                let should_download = file.size < 1_048_576; // 1MB limit for text files

                                let content = if should_download {
                                    self.download_file(&file.path_display).await.unwrap_or_default()
                                } else {
                                    debug!("Skipping download for large file: {}", file.name);
                                    String::new()
                                };

                                let mime_type = mime_guess::from_path(&file.name)
                                    .first_or_octet_stream()
                                    .to_string();

                                documents.push(Document {
                                    id: format!("dropbox-{}", file.id),
                                    title: file.name.clone(),
                                    content,
                                    metadata: json!({
                                        "source": "dropbox",
                                        "file_id": file.id,
                                        "path": file.path_display,
                                        "size": file.size,
                                        "server_modified": file.server_modified,
                                        "content_hash": file.content_hash,
                                        "rev": file.rev,
                                        "mime_type": mime_type
                                    }),
                                });
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to fetch Dropbox folder {}: {}", path_str, e);
                    }
                }
            }
        }

        Ok(SyncResult { documents, repositories: vec![] })
    }

    async fn fetch_branches(&self, _repo_url: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        Err("Dropbox does not support branches".into())
    }
}

impl Default for DropboxConnector {
    fn default() -> Self {
        Self::new()
    }
}
