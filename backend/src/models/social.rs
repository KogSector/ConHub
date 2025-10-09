use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SocialPlatform {
    Slack,
    Notion,
    GoogleDrive,
    Gmail,
    Dropbox,
    LinkedIn,
}

impl FromStr for SocialPlatform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "slack" => Ok(SocialPlatform::Slack),
            "notion" => Ok(SocialPlatform::Notion),
            "google_drive" => Ok(SocialPlatform::GoogleDrive),
            "gmail" => Ok(SocialPlatform::Gmail),
            "dropbox" => Ok(SocialPlatform::Dropbox),
            "linkedin" => Ok(SocialPlatform::LinkedIn),
            _ => Err(format!("Unknown platform: {}", s)),
        }
    }
}

impl std::fmt::Display for SocialPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SocialPlatform::Slack => "slack",
            SocialPlatform::Notion => "notion",
            SocialPlatform::GoogleDrive => "google_drive",
            SocialPlatform::Gmail => "gmail",
            SocialPlatform::Dropbox => "dropbox",
            SocialPlatform::LinkedIn => "linkedin",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SocialConnection {
    pub id: Uuid,
    pub user_id: Uuid,
    pub platform: String, // Store as string in DB
    pub platform_user_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub scope: String,
    pub is_active: bool,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>, // Store platform-specific data
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SocialConnectionRequest {
    pub platform: SocialPlatform,
    pub code: String,
    pub state: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SocialConnectionResponse {
    pub id: Uuid,
    pub platform: SocialPlatform,
    pub platform_user_id: String,
    pub connected_at: DateTime<Utc>,
    pub is_active: bool,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

impl From<SocialConnection> for SocialConnectionResponse {
    fn from(conn: SocialConnection) -> Self {
        Self {
            id: conn.id,
            platform: conn.platform.parse().unwrap_or(SocialPlatform::Slack),
            platform_user_id: conn.platform_user_id,
            connected_at: conn.created_at,
            is_active: conn.is_active,
            last_sync_at: conn.last_sync_at,
            metadata: conn.metadata,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SlackData {
    pub workspace_name: String,
    pub workspace_id: String,
    pub user_name: String,
    pub channels: Vec<SlackChannel>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SlackChannel {
    pub id: String,
    pub name: String,
    pub is_member: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotionData {
    pub workspace_name: String,
    pub workspace_id: String,
    pub pages: Vec<NotionPage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotionPage {
    pub id: String,
    pub title: String,
    pub url: String,
    pub last_edited_time: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleDriveData {
    pub email: String,
    pub name: String,
    pub storage_quota: u64,
    pub files_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DropboxData {
    pub email: String,
    pub name: String,
    pub account_id: String,
    pub used_space: u64,
    pub allocated_space: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkedInData {
    pub name: String,
    pub headline: String,
    pub industry: Option<String>,
    pub location: Option<String>,
    pub connections: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataSyncRequest {
    pub connection_id: Uuid,
    pub sync_type: String, // "full" or "incremental"
    pub options: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataSyncResponse {
    pub connection_id: Uuid,
    pub platform: SocialPlatform,
    pub sync_started_at: DateTime<Utc>,
    pub items_processed: u32,
    pub status: String, // "success", "partial", "failed"
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialData {
    pub external_id: String,
    pub title: String,
    pub content: String,
    pub url: Option<String>,
    pub metadata: serde_json::Value,
    pub synced_at: DateTime<Utc>,
}