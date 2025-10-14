use super::{ConnectorInterface, DataSource, Document, SyncResult};
use async_trait::async_trait;
use reqwest::Client;

use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::info;

pub struct NotionConnector {
    client: Client,
    api_key: Option<String>,
}

impl NotionConnector {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_key: None,
        }
    }

    async fn get_page_content(&self, page_id: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let api_key = self.api_key.as_ref().ok_or("Notion not connected")?;
        
        let url = format!("https://api.notion.com/v1/blocks/{}/children", page_id);
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Notion-Version", "2022-06-28")
            .send()
            .await?;

        if response.status().is_success() {
            let blocks_data: Value = response.json().await?;
            let mut content = String::new();

            if let Some(results) = blocks_data.get("results").and_then(|r| r.as_array()) {
                for block in results {
                    if let Some(block_type) = block.get("type").and_then(|t| t.as_str()) {
                        match block_type {
                            "paragraph" => {
                                if let Some(text_array) = block.get("paragraph")
                                    .and_then(|p| p.get("rich_text"))
                                    .and_then(|rt| rt.as_array()) {
                                    for text_obj in text_array {
                                        if let Some(text) = text_obj.get("plain_text").and_then(|t| t.as_str()) {
                                            content.push_str(text);
                                        }
                                    }
                                    content.push('\n');
                                }
                            },
                            "heading_1" | "heading_2" | "heading_3" => {
                                if let Some(text_array) = block.get(block_type)
                                    .and_then(|h| h.get("rich_text"))
                                    .and_then(|rt| rt.as_array()) {
                                    let level = match block_type {
                                        "heading_1" => "#",
                                        "heading_2" => "##",
                                        "heading_3" => "###",
                                        _ => "",
                                    };
                                    content.push_str(level);
                                    content.push(' ');
                                    for text_obj in text_array {
                                        if let Some(text) = text_obj.get("plain_text").and_then(|t| t.as_str()) {
                                            content.push_str(text);
                                        }
                                    }
                                    content.push('\n');
                                }
                            },
                            _ => {
                                
                            }
                        }
                    }
                }
            }

            Ok(content)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to get Notion page content: {}", error_text).into())
        }
    }
}

#[async_trait]
impl ConnectorInterface for NotionConnector {
    async fn validate(&self, credentials: &HashMap<String, String>) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let api_key = credentials.get("apiKey")
            .ok_or("Notion API key is required")?;

        let response = self.client
            .get("https://api.notion.com/v1/users/me")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Notion-Version", "2022-06-28")
            .send()
            .await?;

        if response.status().is_success() {
            info!("Notion API key validated successfully");
            Ok(true)
        } else {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Notion authentication failed ({}): {}", status, error_text).into())
        }
    }

    async fn connect(&mut self, credentials: &HashMap<String, String>, _config: &Value) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let api_key = credentials.get("apiKey")
            .ok_or("Notion API key is required")?;

        self.api_key = Some(api_key.clone());

        info!("Notion connected successfully");
        Ok(true)
    }

    async fn sync(&self, data_source: &DataSource) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let api_key = self.api_key.as_ref().ok_or("Notion not connected")?;
        let mut documents = Vec::new();

        
        let empty_vec = vec![];
        let database_ids = data_source.config.get("databaseIds")
            .and_then(|d| d.as_array())
            .unwrap_or(&empty_vec);

        for database_id in database_ids {
            if let Some(db_id) = database_id.as_str() {
                let url = format!("https://api.notion.com/v1/databases/{}/query", db_id);
                let response = self.client
                    .post(&url)
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("Notion-Version", "2022-06-28")
                    .json(&json!({}))
                    .send()
                    .await?;

                if let Ok(response_data) = response.json::<Value>().await {
                    if let Some(results) = response_data.get("results").and_then(|r| r.as_array()) {
                        for page in results {
                            if let Some(page_id) = page.get("id").and_then(|id| id.as_str()) {
                                let title = page.get("properties")
                                    .and_then(|props| {
                                        
                                        for (_, prop) in props.as_object()? {
                                            if let Some(prop_type) = prop.get("type").and_then(|t| t.as_str()) {
                                                if prop_type == "title" {
                                                    if let Some(title_array) = prop.get("title").and_then(|t| t.as_array()) {
                                                        if let Some(first_title) = title_array.first() {
                                                            return first_title.get("plain_text").and_then(|t| t.as_str());
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        None
                                    })
                                    .unwrap_or("Untitled")
                                    .to_string();

                                let content = self.get_page_content(page_id).await.unwrap_or_default();

                                documents.push(Document {
                                    id: format!("notion-{}", page_id),
                                    title,
                                    content,
                                    metadata: json!({
                                        "source": "notion",
                                        "page_id": page_id,
                                        "database_id": db_id,
                                        "type": "database_page"
                                    }),
                                });
                            }
                        }
                    }
                }
            }
        }

        
        let empty_vec2 = vec![];
        let page_ids = data_source.config.get("pageIds")
            .and_then(|p| p.as_array())
            .unwrap_or(&empty_vec2);

        for page_id in page_ids {
            if let Some(page_id_str) = page_id.as_str() {
                let url = format!("https://api.notion.com/v1/pages/{}", page_id_str);
                let response = self.client
                    .get(&url)
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("Notion-Version", "2022-06-28")
                    .send()
                    .await?;

                if let Ok(page_data) = response.json::<Value>().await {
                    let title = page_data.get("properties")
                        .and_then(|props| {
                            for (_, prop) in props.as_object()? {
                                if let Some(prop_type) = prop.get("type").and_then(|t| t.as_str()) {
                                    if prop_type == "title" {
                                        if let Some(title_array) = prop.get("title").and_then(|t| t.as_array()) {
                                            if let Some(first_title) = title_array.first() {
                                                return first_title.get("plain_text").and_then(|t| t.as_str());
                                            }
                                        }
                                    }
                                }
                            }
                            None
                        })
                        .unwrap_or("Untitled")
                        .to_string();

                    let content = self.get_page_content(page_id_str).await.unwrap_or_default();

                    documents.push(Document {
                        id: format!("notion-{}", page_id_str),
                        title,
                        content,
                        metadata: json!({
                            "source": "notion",
                            "page_id": page_id_str,
                            "type": "page"
                        }),
                    });
                }
            }
        }

        Ok(SyncResult { documents, repositories: vec![] })
    }

    async fn fetch_branches(&self, _repo_url: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        Err("Notion does not support branches".into())
    }
}