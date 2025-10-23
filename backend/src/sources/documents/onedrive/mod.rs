use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{info, warn, debug};

use crate::sources::core::{DataSourceConnector, DataSource, Document, SyncResult};

/// OneDrive drive metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneDriveDrive {
    pub id: String,
    #[serde(rename = "driveType")]
    pub drive_type: String,
}

/// OneDrive item (file or folder)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneDriveItem {
    pub id: String,
    pub name: String,
    #[serde(rename = "webUrl")]
    pub web_url: String,
    pub size: u64,
    #[serde(rename = "createdDateTime")]
    pub created_date_time: String,
    #[serde(rename = "lastModifiedDateTime")]
    pub last_modified_date_time: String,
    pub file: Option<OneDriveFile>,
    pub folder: Option<OneDriveFolder>,
    #[serde(rename = "parentReference")]
    pub parent_reference: Option<OneDriveParentReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneDriveFile {
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub hashes: Option<OneDriveHashes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneDriveFolder {
    #[serde(rename = "childCount")]
    pub child_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneDriveHashes {
    #[serde(rename = "quickXorHash")]
    pub quick_xor_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneDriveParentReference {
    #[serde(rename = "driveId")]
    pub drive_id: String,
    pub id: String,
    pub path: Option<String>,
}

/// OneDrive children response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildrenResponse {
    pub value: Vec<OneDriveItem>,
    #[serde(rename = "@odata.nextLink")]
    pub next_link: Option<String>,
}

pub struct OneDriveConnector {
    client: Client,
    access_token: Option<String>,
    refresh_token: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
}

impl OneDriveConnector {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            access_token: None,
            refresh_token: None,
            client_id: None,
            client_secret: None,
        }
    }

    fn get_error_message(&self, status: u16, message: &str) -> String {
        match status {
            401 => "OneDrive token expired. Refreshing token...".to_string(),
            403 => "Insufficient permissions. Grant Files.Read scope.".to_string(),
            404 => "File or folder not found on OneDrive.".to_string(),
            429 => "OneDrive API rate limit exceeded. Please wait and try again.".to_string(),
            503 => "OneDrive service unavailable. Please try again later.".to_string(),
            _ => format!("OneDrive API error: {}", message),
        }
    }

    async fn refresh_access_token(&mut self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let refresh_token = self.refresh_token.as_ref()
            .ok_or("Refresh token not available")?;
        let client_id = self.client_id.as_ref()
            .ok_or("Client ID not configured")?;
        let client_secret = self.client_secret.as_ref()
            .ok_or("Client secret not configured")?;

        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token.as_str()),
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("scope", "Files.Read Files.Read.All offline_access"),
        ];

        let response = self.client
            .post("https://login.microsoftonline.com/common/oauth2/v2.0/token")
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let token_data: Value = response.json().await?;
            let access_token = token_data.get("access_token")
                .and_then(|t| t.as_str())
                .ok_or("No access token in response")?;

            // Update refresh token if provided
            if let Some(new_refresh_token) = token_data.get("refresh_token").and_then(|t| t.as_str()) {
                self.refresh_token = Some(new_refresh_token.to_string());
            }

            self.access_token = Some(access_token.to_string());
            info!("OneDrive access token refreshed successfully");
            Ok(access_token.to_string())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to refresh OneDrive token: {}", error_text).into())
        }
    }

    async fn get_default_drive(&self) -> Result<OneDriveDrive, Box<dyn std::error::Error + Send + Sync>> {
        let token = self.access_token.as_ref().ok_or("OneDrive not connected")?;

        let response = self.client
            .get("https://graph.microsoft.com/v1.0/me/drive")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if response.status().is_success() {
            let drive: OneDriveDrive = response.json().await?;
            Ok(drive)
        } else {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            Err(self.get_error_message(status, &error_text).into())
        }
    }

    async fn get_item_by_path(&self, drive_id: &str, path: &str) -> Result<OneDriveItem, Box<dyn std::error::Error + Send + Sync>> {
        let token = self.access_token.as_ref().ok_or("OneDrive not connected")?;

        // Clean and encode path
        let clean_path = path.trim_matches('/');
        let encoded_path = urlencoding::encode(clean_path);

        let url = if clean_path.is_empty() {
            format!("https://graph.microsoft.com/v1.0/drives/{}/root", drive_id)
        } else {
            format!("https://graph.microsoft.com/v1.0/drives/{}/root:/{}", drive_id, encoded_path)
        };

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if response.status().is_success() {
            let item: OneDriveItem = response.json().await?;
            Ok(item)
        } else {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            Err(self.get_error_message(status, &error_text).into())
        }
    }

    async fn list_children(&self, drive_id: &str, item_id: &str) -> Result<Vec<OneDriveItem>, Box<dyn std::error::Error + Send + Sync>> {
        let token = self.access_token.as_ref().ok_or("OneDrive not connected")?;
        let mut all_items = Vec::new();

        let mut url = format!(
            "https://graph.microsoft.com/v1.0/drives/{}/items/{}/children?$top=200",
            drive_id, item_id
        );

        loop {
            let response = self.client
                .get(&url)
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await?;

            if response.status().is_success() {
                let children: ChildrenResponse = response.json().await?;
                all_items.extend(children.value);

                // Handle pagination
                if let Some(next_link) = children.next_link {
                    url = next_link;
                } else {
                    break;
                }
            } else {
                let status = response.status().as_u16();
                let error_text = response.text().await.unwrap_or_default();
                return Err(self.get_error_message(status, &error_text).into());
            }
        }

        Ok(all_items)
    }

    async fn download_file_content(&self, drive_id: &str, item_id: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let token = self.access_token.as_ref().ok_or("OneDrive not connected")?;

        let url = format!(
            "https://graph.microsoft.com/v1.0/drives/{}/items/{}/content",
            drive_id, item_id
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if response.status().is_success() {
            let content = response.text().await?;
            Ok(content)
        } else {
            debug!("Failed to download file content for item {}", item_id);
            Ok(String::new())
        }
    }

    async fn fetch_folder_recursive(
        &self,
        drive_id: &str,
        item_id: &str,
        config: &Value,
    ) -> Result<Vec<OneDriveItem>, Box<dyn std::error::Error + Send + Sync>> {
        let mut all_files = Vec::new();
        let children = self.list_children(drive_id, item_id).await?;

        for child in children {
            if child.folder.is_some() {
                // Recursively fetch folder contents
                let recursive = config.get("recursive")
                    .and_then(|r| r.as_bool())
                    .unwrap_or(true);

                if recursive {
                    let subfolder_files = self.fetch_folder_recursive(drive_id, &child.id, config).await?;
                    all_files.extend(subfolder_files);
                }
            } else if child.file.is_some() {
                all_files.push(child);
            }
        }

        Ok(all_files)
    }

    fn should_include_file(&self, item: &OneDriveItem, config: &Value) -> bool {
        // Check file types filter
        if let Some(file_types) = config.get("fileTypes").and_then(|ft| ft.as_array()) {
            let extension = item.name.split('.').last().unwrap_or("");
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
            if item.size > max_size {
                return false;
            }
        }

        true
    }
}

#[async_trait]
impl DataSourceConnector for OneDriveConnector {
    async fn validate(&self, credentials: &HashMap<String, String>) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let access_token = credentials.get("accessToken")
            .ok_or("OneDrive access token is required")?;

        // Test token by getting user's drive
        let response = self.client
            .get("https://graph.microsoft.com/v1.0/me/drive")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        if response.status().is_success() {
            info!("OneDrive token validated successfully");
            Ok(true)
        } else {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            Err(self.get_error_message(status, &error_text).into())
        }
    }

    async fn connect(&mut self, credentials: &HashMap<String, String>, _config: &Value) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let access_token = credentials.get("accessToken")
            .ok_or("OneDrive access token is required")?;

        self.access_token = Some(access_token.clone());

        // Store refresh credentials if available
        if let Some(refresh_token) = credentials.get("refreshToken") {
            self.refresh_token = Some(refresh_token.clone());
        }
        if let Some(client_id) = credentials.get("clientId") {
            self.client_id = Some(client_id.clone());
        }
        if let Some(client_secret) = credentials.get("clientSecret") {
            self.client_secret = Some(client_secret.clone());
        }

        info!("OneDrive connected successfully");
        Ok(true)
    }

    async fn sync(&self, data_source: &DataSource) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut documents = Vec::new();

        // Get default drive or use configured drive ID
        let drive_id = if let Some(drive_id_str) = data_source.config.get("driveId").and_then(|d| d.as_str()) {
            if drive_id_str == "default" {
                self.get_default_drive().await?.id
            } else {
                drive_id_str.to_string()
            }
        } else {
            self.get_default_drive().await?.id
        };

        let empty_vec = vec![];
        let folder_paths = data_source.config.get("folderPaths")
            .and_then(|fp| fp.as_array())
            .unwrap_or(&empty_vec);

        for folder_path in folder_paths {
            if let Some(path_str) = folder_path.as_str() {
                match self.get_item_by_path(&drive_id, path_str).await {
                    Ok(folder_item) => {
                        let files = self.fetch_folder_recursive(&drive_id, &folder_item.id, &data_source.config).await?;
                        info!("Fetched {} files from OneDrive path: {}", files.len(), path_str);

                        for item in files {
                            // Apply filters
                            if !self.should_include_file(&item, &data_source.config) {
                                debug!("Skipping file (filtered): {}", item.name);
                                continue;
                            }

                            // Download content for small text files
                            let should_download = item.size < 1_048_576; // 1MB limit
                            let content = if should_download {
                                self.download_file_content(&drive_id, &item.id).await.unwrap_or_default()
                            } else {
                                debug!("Skipping download for large file: {}", item.name);
                                String::new()
                            };

                            let mime_type = item.file.as_ref()
                                .map(|f| f.mime_type.clone())
                                .unwrap_or_else(|| {
                                    mime_guess::from_path(&item.name)
                                        .first_or_octet_stream()
                                        .to_string()
                                });

                            let parent_path = item.parent_reference.as_ref()
                                .and_then(|pr| pr.path.clone())
                                .unwrap_or_default();

                            documents.push(Document {
                                id: format!("onedrive-{}", item.id),
                                title: item.name.clone(),
                                content,
                                metadata: json!({
                                    "source": "onedrive",
                                    "drive_id": drive_id,
                                    "item_id": item.id,
                                    "name": item.name,
                                    "web_url": item.web_url,
                                    "size": item.size,
                                    "created_at": item.created_date_time,
                                    "modified_at": item.last_modified_date_time,
                                    "mime_type": mime_type,
                                    "parent_path": parent_path
                                }),
                            });
                        }
                    }
                    Err(e) => {
                        warn!("Failed to fetch OneDrive folder {}: {}", path_str, e);
                    }
                }
            }
        }

        Ok(SyncResult { documents, repositories: vec![] })
    }

    async fn fetch_branches(&self, _repo_url: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        Err("OneDrive does not support branches".into())
    }
}

impl Default for OneDriveConnector {
    fn default() -> Self {
        Self::new()
    }
}
