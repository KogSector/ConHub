use anyhow::Result;
use chrono::Utc;
use serde_json::Value;
use reqwest::Client;
use conhub_models::social::{SocialPlatform, SocialData};

pub struct PlatformDataFetcher {
    client: Client,
}

impl PlatformDataFetcher {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    
    pub async fn fetch_platform_data(
        &self,
        platform: SocialPlatform,
        access_token: &str,
        data_types: Vec<String>,
    ) -> Result<Vec<SocialData>> {
        match platform {
            SocialPlatform::Slack => self.fetch_slack_data(access_token, data_types).await,
            SocialPlatform::Notion => self.fetch_notion_data(access_token, data_types).await,
            SocialPlatform::GoogleDrive => self.fetch_google_drive_data(access_token, data_types).await,
            SocialPlatform::Gmail => self.fetch_gmail_data(access_token, data_types).await,
            SocialPlatform::Dropbox => self.fetch_dropbox_data(access_token, data_types).await,
            SocialPlatform::LinkedIn => self.fetch_linkedin_data(access_token, data_types).await,
        }
    }

    
    async fn fetch_slack_data(&self, access_token: &str, data_types: Vec<String>) -> Result<Vec<SocialData>> {
        let mut results = Vec::new();

        for data_type in data_types {
            match data_type.as_str() {
                "channels" => {
                    let channels = self.fetch_slack_channels(access_token).await?;
                    results.extend(channels);
                },
                "messages" => {
                    let messages = self.fetch_slack_messages(access_token).await?;
                    results.extend(messages);
                },
                "users" => {
                    let users = self.fetch_slack_users(access_token).await?;
                    results.extend(users);
                },
                _ => continue,
            }
        }

        Ok(results)
    }

    async fn fetch_slack_channels(&self, access_token: &str) -> Result<Vec<SocialData>> {
        let response = self.client
            .get("https://slack.com/api/conversations.list")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        let data: Value = response.json().await?;
        let empty_vec = vec![];
        let channels = data["channels"].as_array().unwrap_or(&empty_vec);

        let mut results = Vec::new();
        for channel in channels {
            let social_data = SocialData {
                external_id: channel["id"].as_str().unwrap_or("").to_string(),
                title: channel["name"].as_str().unwrap_or("").to_string(),
                content: format!("Slack channel: {}", channel["name"].as_str().unwrap_or("")),
                url: Some(format!("https://app.slack.com/client/{}/{}", 
                    channel["context_team_id"].as_str().unwrap_or(""), 
                    channel["id"].as_str().unwrap_or(""))),
                metadata: channel.clone(),
                synced_at: Utc::now(),
            };
            results.push(social_data);
        }

        Ok(results)
    }

    async fn fetch_slack_messages(&self, access_token: &str) -> Result<Vec<SocialData>> {
        
        let channels_response = self.client
            .get("https://slack.com/api/conversations.list")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        let channels_data: Value = channels_response.json().await?;
        let empty_vec = vec![];
        let channels = channels_data["channels"].as_array().unwrap_or(&empty_vec);

        let mut results = Vec::new();

        
        for channel in channels.iter().take(5) {
            let channel_id = channel["id"].as_str().unwrap_or("");
            let messages_response = self.client
                .get("https://slack.com/api/conversations.history")
                .header("Authorization", format!("Bearer {}", access_token))
                .query(&[("channel", channel_id), ("limit", "10")])
                .send()
                .await?;

            let messages_data: Value = messages_response.json().await?;
            let empty_vec = vec![];
            let messages = messages_data["messages"].as_array().unwrap_or(&empty_vec);

            for message in messages {
                let social_data = SocialData {
                    external_id: format!("{}_{}", channel_id, message["ts"].as_str().unwrap_or("")),
                    title: format!("Message in #{}", channel["name"].as_str().unwrap_or("")),
                    content: message["text"].as_str().unwrap_or("").to_string(),
                    url: Some(format!("https://app.slack.com/client/{}/{}", 
                        channel["context_team_id"].as_str().unwrap_or(""), 
                        channel_id)),
                    metadata: message.clone(),
                    synced_at: Utc::now(),
                };
                results.push(social_data);
            }
        }

        Ok(results)
    }

    async fn fetch_slack_users(&self, access_token: &str) -> Result<Vec<SocialData>> {
        let response = self.client
            .get("https://slack.com/api/users.list")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        let data: Value = response.json().await?;
        let empty_vec = vec![];
        let users = data["members"].as_array().unwrap_or(&empty_vec);

        let mut results = Vec::new();
        for user in users {
            let social_data = SocialData {
                external_id: user["id"].as_str().unwrap_or("").to_string(),
                title: user["real_name"].as_str().unwrap_or("").to_string(),
                content: format!("Slack user: {}", user["real_name"].as_str().unwrap_or("")),
                url: None,
                metadata: user.clone(),
                synced_at: Utc::now(),
            };
            results.push(social_data);
        }

        Ok(results)
    }

    
    async fn fetch_notion_data(&self, access_token: &str, data_types: Vec<String>) -> Result<Vec<SocialData>> {
        let mut results = Vec::new();

        for data_type in data_types {
            match data_type.as_str() {
                "pages" => {
                    let pages = self.fetch_notion_pages(access_token).await?;
                    results.extend(pages);
                },
                "databases" => {
                    let databases = self.fetch_notion_databases(access_token).await?;
                    results.extend(databases);
                },
                _ => continue,
            }
        }

        Ok(results)
    }

    async fn fetch_notion_pages(&self, access_token: &str) -> Result<Vec<SocialData>> {
        let response = self.client
            .post("https://api.notion.com/v1/search")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Notion-Version", "2022-06-28")
            .json(&serde_json::json!({
                "filter": {
                    "property": "object",
                    "value": "page"
                }
            }))
            .send()
            .await?;

        let data: Value = response.json().await?;
        let empty_vec = vec![];
        let pages = data["results"].as_array().unwrap_or(&empty_vec);

        let mut results = Vec::new();
        for page in pages {
            let title = page["properties"]["title"]["title"][0]["plain_text"]
                .as_str()
                .unwrap_or("Untitled");
            
            let social_data = SocialData {
                external_id: page["id"].as_str().unwrap_or("").to_string(),
                title: title.to_string(),
                content: format!("Notion page: {}", title),
                url: Some(page["url"].as_str().unwrap_or("").to_string()),
                metadata: page.clone(),
                synced_at: Utc::now(),
            };
            results.push(social_data);
        }

        Ok(results)
    }

    async fn fetch_notion_databases(&self, access_token: &str) -> Result<Vec<SocialData>> {
        let response = self.client
            .post("https://api.notion.com/v1/search")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Notion-Version", "2022-06-28")
            .json(&serde_json::json!({
                "filter": {
                    "property": "object",
                    "value": "database"
                }
            }))
            .send()
            .await?;

        let data: Value = response.json().await?;
        let empty_vec = vec![];
        let databases = data["results"].as_array().unwrap_or(&empty_vec);

        let mut results = Vec::new();
        for database in databases {
            let title = database["title"][0]["plain_text"]
                .as_str()
                .unwrap_or("Untitled Database");
            
            let social_data = SocialData {
                external_id: database["id"].as_str().unwrap_or("").to_string(),
                title: title.to_string(),
                content: format!("Notion database: {}", title),
                url: Some(database["url"].as_str().unwrap_or("").to_string()),
                metadata: database.clone(),
                synced_at: Utc::now(),
            };
            results.push(social_data);
        }

        Ok(results)
    }

    
    async fn fetch_google_drive_data(&self, access_token: &str, data_types: Vec<String>) -> Result<Vec<SocialData>> {
        let mut results = Vec::new();

        for data_type in data_types {
            match data_type.as_str() {
                "files" => {
                    let files = self.fetch_google_drive_files(access_token).await?;
                    results.extend(files);
                },
                _ => continue,
            }
        }

        Ok(results)
    }

    async fn fetch_google_drive_files(&self, access_token: &str) -> Result<Vec<SocialData>> {
        let response = self.client
            .get("https://www.googleapis.com/drive/v3/files")
            .header("Authorization", format!("Bearer {}", access_token))
            .query(&[("pageSize", "20"), ("fields", "files(id,name,mimeType,webViewLink,modifiedTime)")])
            .send()
            .await?;

        let data: Value = response.json().await?;
        let empty_vec = vec![];
        let files = data["files"].as_array().unwrap_or(&empty_vec);

        let mut results = Vec::new();
        for file in files {
            let social_data = SocialData {
                external_id: file["id"].as_str().unwrap_or("").to_string(),
                title: file["name"].as_str().unwrap_or("").to_string(),
                content: format!("Google Drive file: {}", file["name"].as_str().unwrap_or("")),
                url: Some(file["webViewLink"].as_str().unwrap_or("").to_string()),
                metadata: file.clone(),
                synced_at: Utc::now(),
            };
            results.push(social_data);
        }

        Ok(results)
    }

    
    async fn fetch_gmail_data(&self, access_token: &str, data_types: Vec<String>) -> Result<Vec<SocialData>> {
        let mut results = Vec::new();

        for data_type in data_types {
            match data_type.as_str() {
                "emails" => {
                    let emails = self.fetch_gmail_emails(access_token).await?;
                    results.extend(emails);
                },
                _ => continue,
            }
        }

        Ok(results)
    }

    async fn fetch_gmail_emails(&self, access_token: &str) -> Result<Vec<SocialData>> {
        let response = self.client
            .get("https://gmail.googleapis.com/gmail/v1/users/me/messages")
            .header("Authorization", format!("Bearer {}", access_token))
            .query(&[("maxResults", "10")])
            .send()
            .await?;

        let data: Value = response.json().await?;
        let empty_vec = vec![];
        let messages = data["messages"].as_array().unwrap_or(&empty_vec);

        let mut results = Vec::new();
        for message in messages {
            let message_id = message["id"].as_str().unwrap_or("");
            
            
            let message_response = self.client
                .get(&format!("https://gmail.googleapis.com/gmail/v1/users/me/messages/{}", message_id))
                .header("Authorization", format!("Bearer {}", access_token))
                .send()
                .await?;

            let message_data: Value = message_response.json().await?;
            let payload = &message_data["payload"];
            let empty_vec = vec![];
            let headers = payload["headers"].as_array().unwrap_or(&empty_vec);
            
            let mut subject = String::new();
            let mut from = String::new();
            
            for header in headers {
                let name = header["name"].as_str().unwrap_or("");
                let value = header["value"].as_str().unwrap_or("");
                match name {
                    "Subject" => subject = value.to_string(),
                    "From" => from = value.to_string(),
                    _ => {}
                }
            }

            let social_data = SocialData {
                external_id: message_id.to_string(),
                title: if subject.is_empty() { "No Subject".to_string() } else { subject },
                content: format!("Email from: {}", from),
                url: Some(format!("https://mail.google.com/mail/u/0/#inbox/{}", message_id)),
                metadata: message_data.clone(),
                synced_at: Utc::now(),
            };
            results.push(social_data);
        }

        Ok(results)
    }

    
    async fn fetch_dropbox_data(&self, access_token: &str, data_types: Vec<String>) -> Result<Vec<SocialData>> {
        let mut results = Vec::new();

        for data_type in data_types {
            match data_type.as_str() {
                "files" => {
                    let files = self.fetch_dropbox_files(access_token).await?;
                    results.extend(files);
                },
                _ => continue,
            }
        }

        Ok(results)
    }

    async fn fetch_dropbox_files(&self, access_token: &str) -> Result<Vec<SocialData>> {
        let response = self.client
            .post("https://api.dropboxapi.com/2/files/list_folder")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "path": "",
                "recursive": false,
                "include_media_info": false,
                "include_deleted": false,
                "include_has_explicit_shared_members": false
            }))
            .send()
            .await?;

        let data: Value = response.json().await?;
        let empty_vec = vec![];
        let entries = data["entries"].as_array().unwrap_or(&empty_vec);

        let mut results = Vec::new();
        for entry in entries {
            let social_data = SocialData {
                external_id: entry["id"].as_str().unwrap_or("").to_string(),
                title: entry["name"].as_str().unwrap_or("").to_string(),
                content: format!("Dropbox file: {}", entry["name"].as_str().unwrap_or("")),
                url: None, 
                metadata: entry.clone(),
                synced_at: Utc::now(),
            };
            results.push(social_data);
        }

        Ok(results)
    }

    
    async fn fetch_linkedin_data(&self, access_token: &str, data_types: Vec<String>) -> Result<Vec<SocialData>> {
        let mut results = Vec::new();

        for data_type in data_types {
            match data_type.as_str() {
                "posts" => {
                    let posts = self.fetch_linkedin_posts(access_token).await?;
                    results.extend(posts);
                },
                "profile" => {
                    let profile = self.fetch_linkedin_profile(access_token).await?;
                    results.push(profile);
                },
                _ => continue,
            }
        }

        Ok(results)
    }

    async fn fetch_linkedin_posts(&self, access_token: &str) -> Result<Vec<SocialData>> {
        
        
        let response = self.client
            .get("https://api.linkedin.com/v2/posts")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        let data: Value = response.json().await?;
        let empty_vec = vec![];
        let posts = data["elements"].as_array().unwrap_or(&empty_vec);

        let mut results = Vec::new();
        for post in posts {
            let social_data = SocialData {
                external_id: post["id"].as_str().unwrap_or("").to_string(),
                title: "LinkedIn Post".to_string(),
                content: post["commentary"].as_str().unwrap_or("").to_string(),
                url: None,
                metadata: post.clone(),
                synced_at: Utc::now(),
            };
            results.push(social_data);
        }

        Ok(results)
    }

    async fn fetch_linkedin_profile(&self, access_token: &str) -> Result<SocialData> {
        let response = self.client
            .get("https://api.linkedin.com/v2/people/~")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        let data: Value = response.json().await?;
        
        Ok(SocialData {
            external_id: data["id"].as_str().unwrap_or("").to_string(),
            title: format!("{} {}", 
                data["firstName"]["localized"]["en_US"].as_str().unwrap_or(""),
                data["lastName"]["localized"]["en_US"].as_str().unwrap_or("")),
            content: "LinkedIn Profile".to_string(),
            url: None,
            metadata: data,
            synced_at: Utc::now(),
        })
    }
}