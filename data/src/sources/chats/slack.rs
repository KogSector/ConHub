use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{info, error};

use crate::sources::core::{DataSourceConnector, DataSource, Document, SyncResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackChannel {
    pub id: String,
    pub name: String,
    pub is_private: bool,
    pub num_members: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackMessage {
    pub ts: String,
    pub user: String,
    pub text: String,
    pub channel: String,
}

pub struct SlackConnector {
    client: Client,
    token: Option<String>,
}

impl SlackConnector {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            token: None,
        }
    }
}

#[async_trait]
impl DataSourceConnector for SlackConnector {
    async fn validate(&self, credentials: &HashMap<String, String>) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let token = credentials.get("token")
            .ok_or("Slack token is required")?;

        let response = self.client
            .get("https://slack.com/api/auth.test")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if response.status().is_success() {
            let result: Value = response.json().await?;
            if result.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                info!("Slack token validated successfully");
                Ok(true)
            } else {
                Err("Invalid Slack token".into())
            }
        } else {
            Err("Failed to validate Slack token".into())
        }
    }

    async fn connect(&mut self, credentials: &HashMap<String, String>, _config: &Value) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let token = credentials.get("token")
            .ok_or("Slack token is required")?;

        self.token = Some(token.clone());
        info!("Slack connected successfully");
        Ok(true)
    }

    async fn sync(&self, data_source: &DataSource) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let token = self.token.as_ref().ok_or("Slack not connected")?;
        let mut documents = Vec::new();

        // Get channels
        let channels_response = self.client
            .get("https://slack.com/api/conversations.list")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if let Ok(channels_data) = channels_response.json::<Value>().await {
            if let Some(channels) = channels_data.get("channels").and_then(|c| c.as_array()) {
                for channel in channels.iter().take(5) { // Limit to 5 channels for demo
                    if let Some(channel_id) = channel.get("id").and_then(|id| id.as_str()) {
                        if let Some(channel_name) = channel.get("name").and_then(|name| name.as_str()) {
                            // Get recent messages from channel
                            let messages_response = self.client
                                .get(&format!("https://slack.com/api/conversations.history?channel={}&limit=10", channel_id))
                                .header("Authorization", format!("Bearer {}", token))
                                .send()
                                .await?;

                            if let Ok(messages_data) = messages_response.json::<Value>().await {
                                if let Some(messages) = messages_data.get("messages").and_then(|m| m.as_array()) {
                                    let mut channel_content = String::new();
                                    for message in messages {
                                        if let Some(text) = message.get("text").and_then(|t| t.as_str()) {
                                            channel_content.push_str(&format!("{}\n", text));
                                        }
                                    }

                                    documents.push(Document {
                                        id: format!("slack-{}-{}", data_source.id, channel_id),
                                        title: format!("#{} - Recent Messages", channel_name),
                                        content: channel_content,
                                        metadata: json!({
                                            "source": "slack",
                                            "channel_id": channel_id,
                                            "channel_name": channel_name,
                                            "message_count": messages.len()
                                        }),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(SyncResult { documents, repositories: Vec::new() })
    }

    async fn fetch_branches(&self, _repo_url: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        Err("Slack does not support branches".into())
    }
}

impl Default for SlackConnector {
    fn default() -> Self {
        Self::new()
    }
}
