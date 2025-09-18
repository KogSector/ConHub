use crate::models::social::*;
use crate::services::platform_data_fetcher::PlatformDataFetcher;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl, basic::BasicClient, reqwest::async_http_client
};
use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;
use anyhow::Result;
use serde_json::json;
use log::{info, error};

pub struct SocialIntegrationService {
    db: PgPool,
    oauth_clients: HashMap<SocialPlatform, BasicClient>,
    data_fetcher: PlatformDataFetcher,
}

impl SocialIntegrationService {
    pub fn new(db: PgPool) -> Result<Self> {
        let mut oauth_clients = HashMap::new();
        
        // Initialize OAuth clients for each platform
        if let Ok(client) = Self::create_slack_client() {
            oauth_clients.insert(SocialPlatform::Slack, client);
        }
        
        if let Ok(client) = Self::create_notion_client() {
            oauth_clients.insert(SocialPlatform::Notion, client);
        }
        
        if let Ok(client) = Self::create_google_client() {
            oauth_clients.insert(SocialPlatform::GoogleDrive, client.clone());
            oauth_clients.insert(SocialPlatform::Gmail, client);
        }
        
        if let Ok(client) = Self::create_dropbox_client() {
            oauth_clients.insert(SocialPlatform::Dropbox, client);
        }
        
        if let Ok(client) = Self::create_linkedin_client() {
            oauth_clients.insert(SocialPlatform::LinkedIn, client);
        }

        Ok(Self { 
            db, 
            oauth_clients,
            data_fetcher: PlatformDataFetcher::new(),
        })
    }

    /// Initialize database tables for social connections
    pub async fn init_database(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS social_connections (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL,
                platform VARCHAR(50) NOT NULL,
                platform_user_id VARCHAR(255) NOT NULL,
                access_token TEXT NOT NULL,
                refresh_token TEXT,
                token_expires_at TIMESTAMP WITH TIME ZONE,
                scope TEXT NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT TRUE,
                last_sync_at TIMESTAMP WITH TIME ZONE,
                metadata JSONB,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                UNIQUE(user_id, platform, platform_user_id)
            );
            
            CREATE INDEX IF NOT EXISTS idx_social_connections_user_id ON social_connections(user_id);
            CREATE INDEX IF NOT EXISTS idx_social_connections_platform ON social_connections(platform);
            CREATE INDEX IF NOT EXISTS idx_social_connections_active ON social_connections(is_active);
            "#,
        )
        .execute(&self.db)
        .await?;
        
        info!("Social connections database tables initialized");
        Ok(())
    }

    /// Generate OAuth authorization URL for a platform
    pub fn get_auth_url(&self, platform: SocialPlatform, _state: Option<String>) -> Result<(String, String)> {
        let client = self.oauth_clients.get(&platform)
            .ok_or_else(|| anyhow::anyhow!("OAuth client not configured for platform: {}", platform))?;

        let scopes = self.get_platform_scopes(&platform);
        let mut auth_request = client.authorize_url(CsrfToken::new_random);
        
        for scope in scopes {
            auth_request = auth_request.add_scope(Scope::new(scope));
        }

        let (auth_url, csrf_token) = auth_request.url();
        
        Ok((auth_url.to_string(), csrf_token.secret().clone()))
    }

    /// Complete OAuth flow and store connection
    pub async fn connect_platform(&self, user_id: Uuid, request: SocialConnectionRequest) -> Result<SocialConnectionResponse> {
        let client = self.oauth_clients.get(&request.platform)
            .ok_or_else(|| anyhow::anyhow!("OAuth client not configured for platform: {}", request.platform))?;

        // Exchange code for token
        let token_result = client
            .exchange_code(AuthorizationCode::new(request.code))
            .request_async(async_http_client)
            .await?;

        let access_token = token_result.access_token().secret();
        let refresh_token = token_result.refresh_token().map(|t| t.secret().clone());
        let expires_at = token_result.expires_in().map(|duration| Utc::now() + chrono::Duration::from_std(duration).unwrap());

        // Get platform user info
        let (platform_user_id, metadata) = self.get_platform_user_info(&request.platform, access_token).await?;

        // Store connection in database
        let connection_id = Uuid::new_v4();
        let scopes = self.get_platform_scopes(&request.platform).join(" ");
        
        sqlx::query(
            r#"
            INSERT INTO social_connections (
                id, user_id, platform, platform_user_id, access_token, refresh_token,
                token_expires_at, scope, is_active, metadata, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (user_id, platform, platform_user_id) 
            DO UPDATE SET 
                access_token = EXCLUDED.access_token,
                refresh_token = EXCLUDED.refresh_token,
                token_expires_at = EXCLUDED.token_expires_at,
                scope = EXCLUDED.scope,
                is_active = TRUE,
                metadata = EXCLUDED.metadata,
                updated_at = NOW()
            "#,
        )
        .bind(connection_id)
        .bind(user_id)
        .bind(request.platform.to_string())
        .bind(&platform_user_id)
        .bind(access_token)
        .bind(refresh_token)
        .bind(expires_at)
        .bind(scopes)
        .bind(true)
        .bind(&metadata)
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(&self.db)
        .await?;

        info!("Connected {} for user {}: {}", request.platform, user_id, platform_user_id);

        Ok(SocialConnectionResponse {
            id: connection_id,
            platform: request.platform,
            platform_user_id,
            connected_at: Utc::now(),
            is_active: true,
            last_sync_at: None,
            metadata: Some(metadata),
        })
    }

    /// Get all connections for a user
    pub async fn get_user_connections(&self, user_id: Uuid) -> Result<Vec<SocialConnectionResponse>> {
        let rows = sqlx::query(
            "SELECT * FROM social_connections WHERE user_id = $1 AND is_active = TRUE ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        let connections = rows.into_iter()
            .map(|row| {
                let conn = SocialConnection {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    platform: row.get("platform"),
                    platform_user_id: row.get("platform_user_id"),
                    access_token: row.get("access_token"),
                    refresh_token: row.get("refresh_token"),
                    token_expires_at: row.get("token_expires_at"),
                    scope: row.get("scope"),
                    is_active: row.get("is_active"),
                    last_sync_at: row.get("last_sync_at"),
                    metadata: row.get("metadata"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };
                conn.into()
            })
            .collect();

        Ok(connections)
    }

    /// Disconnect a platform
    pub async fn disconnect_platform(&self, user_id: Uuid, connection_id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE social_connections SET is_active = FALSE, updated_at = NOW() WHERE id = $1 AND user_id = $2"
        )
        .bind(connection_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        info!("Disconnected social connection {} for user {}", connection_id, user_id);
        Ok(())
    }

    /// Sync data from a connected platform
    pub async fn sync_platform_data(&self, connection_id: Uuid, _sync_request: DataSyncRequest) -> Result<DataSyncResponse> {
        let connection = self.get_connection_by_id(connection_id).await?;
        let platform: SocialPlatform = connection.platform.parse()
            .map_err(|e| anyhow::anyhow!("Invalid platform: {}", e))?;
        
        let sync_started_at = Utc::now();
        let mut items_processed = 0;
        let mut status = "success".to_string();
        let mut error = None;

        // Get access token for the connection
        let token_row = sqlx::query(
            "SELECT access_token FROM social_tokens WHERE connection_id = $1"
        )
        .bind(connection_id)
        .fetch_one(&self.db)
        .await?;
        
        let access_token: String = token_row.get("access_token");

        // Define what data types to fetch for each platform
        let data_types = match platform {
            SocialPlatform::Slack => vec!["channels".to_string(), "messages".to_string()],
            SocialPlatform::Notion => vec!["pages".to_string(), "databases".to_string()],
            SocialPlatform::GoogleDrive => vec!["files".to_string()],
            SocialPlatform::Gmail => vec!["emails".to_string()],
            SocialPlatform::Dropbox => vec!["files".to_string()],
            SocialPlatform::LinkedIn => vec!["profile".to_string(), "posts".to_string()],
        };

        // Fetch data using our data fetcher
        match self.data_fetcher.fetch_platform_data(platform.clone(), &access_token, data_types).await {
            Ok(social_data_items) => {
                // Store the fetched data in the database
                for social_data in social_data_items {
                    match sqlx::query(
                        "INSERT INTO social_data (connection_id, platform, data_type, external_id, title, content, url, metadata, synced_at)
                         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                         ON CONFLICT (connection_id, external_id) 
                         DO UPDATE SET title = $5, content = $6, url = $7, metadata = $8, synced_at = $9"
                    )
                    .bind(connection_id)
                    .bind(platform.to_string())
                    .bind("general") // Default data type - could be more specific based on the data
                    .bind(&social_data.external_id)
                    .bind(&social_data.title)
                    .bind(&social_data.content)
                    .bind(&social_data.url)
                    .bind(&social_data.metadata)
                    .bind(social_data.synced_at)
                    .execute(&self.db)
                    .await {
                        Ok(_) => items_processed += 1,
                        Err(e) => {
                            error!("Failed to store social data: {}", e);
                        }
                    }
                }

                // Update last sync time for the connection
                sqlx::query("UPDATE social_connections SET last_sync = $1 WHERE id = $2")
                    .bind(Utc::now())
                    .bind(connection_id)
                    .execute(&self.db)
                    .await?;
            },
            Err(e) => {
                status = "failed".to_string();
                error = Some(e.to_string());
            }
        }

        Ok(DataSyncResponse {
            connection_id,
            platform,
            sync_started_at,
            items_processed,
            status,
            error,
        })
    }

    /// Get a connection by ID
    async fn get_connection_by_id(&self, connection_id: Uuid) -> Result<SocialConnection> {
        let row = sqlx::query(
            "SELECT id, user_id, platform, platform_user_id, access_token, refresh_token, token_expires_at, scope, last_sync_at, created_at, updated_at 
             FROM social_connections WHERE id = $1"
        )
        .bind(connection_id)
        .fetch_one(&self.db)
        .await?;

        Ok(SocialConnection {
            id: row.get("id"),
            user_id: row.get("user_id"),
            platform: row.get("platform"),
            platform_user_id: row.get("platform_user_id"),
            access_token: row.get("access_token"),
            refresh_token: row.get("refresh_token"),
            token_expires_at: row.get("token_expires_at"),
            scope: row.get("scope"),
            is_active: row.get("is_active"),
            last_sync_at: row.get("last_sync_at"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    // Private helper methods
    fn create_slack_client() -> Result<BasicClient> {
        let client_id = std::env::var("SLACK_CLIENT_ID")?;
        let client_secret = std::env::var("SLACK_CLIENT_SECRET")?;
        let redirect_uri = std::env::var("SLACK_REDIRECT_URI")?;

        Ok(BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new("https://slack.com/oauth/v2/authorize".to_string())?,
            Some(TokenUrl::new("https://slack.com/api/oauth.v2.access".to_string())?),
        ).set_redirect_uri(RedirectUrl::new(redirect_uri)?))
    }

    fn create_notion_client() -> Result<BasicClient> {
        let client_id = std::env::var("NOTION_CLIENT_ID")?;
        let client_secret = std::env::var("NOTION_CLIENT_SECRET")?;
        let redirect_uri = std::env::var("NOTION_REDIRECT_URI")?;

        Ok(BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new("https://api.notion.com/v1/oauth/authorize".to_string())?,
            Some(TokenUrl::new("https://api.notion.com/v1/oauth/token".to_string())?),
        ).set_redirect_uri(RedirectUrl::new(redirect_uri)?))
    }

    fn create_google_client() -> Result<BasicClient> {
        let client_id = std::env::var("GOOGLE_CLIENT_ID")?;
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")?;
        let redirect_uri = std::env::var("GOOGLE_REDIRECT_URI")?;

        Ok(BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?,
            Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string())?),
        ).set_redirect_uri(RedirectUrl::new(redirect_uri)?))
    }

    fn create_dropbox_client() -> Result<BasicClient> {
        let client_id = std::env::var("DROPBOX_CLIENT_ID")?;
        let client_secret = std::env::var("DROPBOX_CLIENT_SECRET")?;
        let redirect_uri = std::env::var("DROPBOX_REDIRECT_URI")?;

        Ok(BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new("https://www.dropbox.com/oauth2/authorize".to_string())?,
            Some(TokenUrl::new("https://api.dropboxapi.com/oauth2/token".to_string())?),
        ).set_redirect_uri(RedirectUrl::new(redirect_uri)?))
    }

    fn create_linkedin_client() -> Result<BasicClient> {
        let client_id = std::env::var("LINKEDIN_CLIENT_ID")?;
        let client_secret = std::env::var("LINKEDIN_CLIENT_SECRET")?;
        let redirect_uri = std::env::var("LINKEDIN_REDIRECT_URI")?;

        Ok(BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new("https://www.linkedin.com/oauth/v2/authorization".to_string())?,
            Some(TokenUrl::new("https://www.linkedin.com/oauth/v2/accessToken".to_string())?),
        ).set_redirect_uri(RedirectUrl::new(redirect_uri)?))
    }

    fn get_platform_scopes(&self, platform: &SocialPlatform) -> Vec<String> {
        match platform {
            SocialPlatform::Slack => vec![
                "channels:read".to_string(),
                "chat:write".to_string(),
                "users:read".to_string(),
                "files:read".to_string(),
            ],
            SocialPlatform::Notion => vec![
                "read_content".to_string(),
            ],
            SocialPlatform::GoogleDrive => vec![
                "https://www.googleapis.com/auth/drive.readonly".to_string(),
                "https://www.googleapis.com/auth/userinfo.profile".to_string(),
            ],
            SocialPlatform::Gmail => vec![
                "https://www.googleapis.com/auth/gmail.readonly".to_string(),
                "https://www.googleapis.com/auth/userinfo.profile".to_string(),
            ],
            SocialPlatform::Dropbox => vec![
                "files.content.read".to_string(),
                "files.metadata.read".to_string(),
            ],
            SocialPlatform::LinkedIn => vec![
                "r_liteprofile".to_string(),
                "r_emailaddress".to_string(),
            ],
        }
    }

    async fn get_platform_user_info(&self, platform: &SocialPlatform, access_token: &str) -> Result<(String, serde_json::Value)> {
        let client = reqwest::Client::new();
        
        match platform {
            SocialPlatform::Slack => {
                let response = client
                    .get("https://slack.com/api/auth.test")
                    .bearer_auth(access_token)
                    .send()
                    .await?
                    .json::<serde_json::Value>()
                    .await?;
                
                let user_id = response["user_id"].as_str().unwrap_or_default().to_string();
                Ok((user_id, response))
            },
            SocialPlatform::Notion => {
                let response = client
                    .get("https://api.notion.com/v1/users/me")
                    .bearer_auth(access_token)
                    .header("Notion-Version", "2022-06-28")
                    .send()
                    .await?
                    .json::<serde_json::Value>()
                    .await?;
                
                let user_id = response["id"].as_str().unwrap_or_default().to_string();
                Ok((user_id, response))
            },
            _ => {
                // Implement other platforms
                Ok(("unknown".to_string(), json!({})))
            }
        }
    }
}