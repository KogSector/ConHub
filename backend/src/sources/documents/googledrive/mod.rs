use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::info;

use crate::sources::core::{DataSourceConnector, DataSource, Document, SyncResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleDriveFile {
    pub id: String,
    pub name: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub size: Option<String>,
    #[serde(rename = "modifiedTime")]
    pub modified_time: String,
    #[serde(rename = "webViewLink")]
    pub web_view_link: Option<String>,
}

pub struct GoogleDriveConnector {
    client: Client,
    access_token: Option<String>,
}

impl GoogleDriveConnector {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            access_token: None,
        }
    }

    async fn refresh_access_token(&mut self, refresh_token: &str, client_id: &str, client_secret: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", client_id),
            ("client_secret", client_secret),
        ];

        let response = self.client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let token_data: Value = response.json().await?;
            let access_token = token_data.get("access_token")
                .and_then(|t| t.as_str())
                .ok_or("No access token in response")?;
            
            self.access_token = Some(access_token.to_string());
            Ok(access_token.to_string())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to refresh Google Drive token: {}", error_text).into())
        }
    }

    async fn export_google_doc(&self, file_id: &str, mime_type: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let token = self.access_token.as_ref().ok_or("Google Drive not connected")?;
        
        let export_mime_type = match mime_type {
            "application/vnd.google-apps.document" => "text/plain",
            "application/vnd.google-apps.presentation" => "text/plain",
            "application/vnd.google-apps.spreadsheet" => "text/csv",
            _ => return Err("Unsupported Google Apps file type".into()),
        };

        let url = format!("https://www.googleapis.com/drive/v3/files/{}/export?mimeType={}", file_id, export_mime_type);
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.text().await?)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to export Google Doc: {}", error_text).into())
        }
    }
}

#[async_trait]
impl DataSourceConnector for GoogleDriveConnector {
    async fn validate(&self, credentials: &HashMap<String, String>) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let client_id = credentials.get("clientId")
            .ok_or("Google Drive client ID is required")?;
        let client_secret = credentials.get("clientSecret")
            .ok_or("Google Drive client secret is required")?;
        let refresh_token = credentials.get("refreshToken")
            .ok_or("Google Drive refresh token is required")?;

        
        let mut temp_connector = GoogleDriveConnector::new();
        temp_connector.refresh_access_token(refresh_token, client_id, client_secret).await?;

        info!("Google Drive credentials validated successfully");
        Ok(true)
    }

    async fn connect(&mut self, credentials: &HashMap<String, String>, _config: &Value) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let client_id = credentials.get("clientId")
            .ok_or("Google Drive client ID is required")?;
        let client_secret = credentials.get("clientSecret")
            .ok_or("Google Drive client secret is required")?;
        let refresh_token = credentials.get("refreshToken")
            .ok_or("Google Drive refresh token is required")?;

        self.refresh_access_token(refresh_token, client_id, client_secret).await?;

        info!("Google Drive connected successfully");
        Ok(true)
    }

    #[allow(dead_code)]
    async fn sync(&self, data_source: &DataSource) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let token = self.access_token.as_ref().ok_or("Google Drive not connected")?;
        let mut documents = Vec::new();

        let empty_vec = vec![];
        let folder_ids = data_source.config.get("folderIds")
            .and_then(|f| f.as_array())
            .unwrap_or(&empty_vec);

        let file_types = data_source.config.get("fileTypes")
            .and_then(|f| f.as_array())
            .unwrap_or(&empty_vec);

        for folder_id in folder_ids {
            if let Some(folder_id_str) = folder_id.as_str() {
                let mut query = format!("'{}' in parents", folder_id_str);
                
                if !file_types.is_empty() {
                    let mime_types: Vec<String> = file_types
                        .iter()
                        .filter_map(|t| t.as_str())
                        .map(|t| format!("mimeType='{}'", t))
                        .collect();
                    
                    if !mime_types.is_empty() {
                        query.push_str(&format!(" and ({})", mime_types.join(" or ")));
                    }
                }

                let url = format!("https://www.googleapis.com/drive/v3/files?q={}&fields=files(id,name,mimeType,size,modifiedTime,webViewLink)", 
                    urlencoding::encode(&query));

                let response = self.client
                    .get(&url)
                    .header("Authorization", format!("Bearer {}", token))
                    .send()
                    .await?;

                if let Ok(response_data) = response.json::<Value>().await {
                    if let Some(files) = response_data.get("files").and_then(|f| f.as_array()) {
                        for file in files {
                            if let Ok(drive_file) = serde_json::from_value::<GoogleDriveFile>(file.clone()) {
                                let content = if drive_file.mime_type.starts_with("application/vnd.google-apps") {
                                    self.export_google_doc(&drive_file.id, &drive_file.mime_type).await.unwrap_or_default()
                                } else {
                                    
                                    String::new()
                                };

                                documents.push(Document {
                                    id: format!("gdrive-{}", drive_file.id),
                                    title: drive_file.name.clone(),
                                    content,
                                    metadata: json!({
                                        "source": "google_drive",
                                        "file_id": drive_file.id,
                                        "mime_type": drive_file.mime_type,
                                        "size": drive_file.size,
                                        "modified_time": drive_file.modified_time,
                                        "web_view_link": drive_file.web_view_link
                                    }),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(SyncResult { documents, repositories: vec![] })
    }

    async fn fetch_branches(&self, _repo_url: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        Err("Google Drive does not support branches".into())
    }
}

impl Default for GoogleDriveConnector {
    fn default() -> Self {
        Self::new()
    }
}