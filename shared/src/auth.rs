use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub user_id: Uuid,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
    pub aud: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SocialPlatform {
    Slack,
    Notion,
    GoogleDrive,
    Gmail,
    Dropbox,
    LinkedIn,
    GitHub,
    GitLab,
    Bitbucket,
}

impl SocialPlatform {
    pub fn as_str(&self) -> &str {
        match self {
            SocialPlatform::Slack => "slack",
            SocialPlatform::Notion => "notion",
            SocialPlatform::GoogleDrive => "google_drive",
            SocialPlatform::Gmail => "gmail",
            SocialPlatform::Dropbox => "dropbox",
            SocialPlatform::LinkedIn => "linkedin",
            SocialPlatform::GitHub => "github",
            SocialPlatform::GitLab => "gitlab",
            SocialPlatform::Bitbucket => "bitbucket",
        }
    }
}
