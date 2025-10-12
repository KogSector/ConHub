use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use reqwest::Client;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

use crate::models::auth::User;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OAuthProvider {
    Google,
    Microsoft,
    GitHub,
}

impl std::fmt::Display for OAuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OAuthProvider::Google => write!(f, "google"),
            OAuthProvider::Microsoft => write!(f, "microsoft"),
            OAuthProvider::GitHub => write!(f, "github"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<i64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GoogleUserInfo {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MicrosoftUserInfo {
    pub id: String,
    #[serde(rename = "userPrincipalName")]
    pub email: String,
    #[serde(rename = "displayName")]
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubUserInfo {
    pub id: u64,
    pub email: Option<String>,
    pub name: Option<String>,
    pub login: String,
    pub avatar_url: Option<String>,
}

pub struct OAuthService {
    pool: PgPool,
    client: Client,
    google_client_id: String,
    google_client_secret: String,
    microsoft_client_id: String,
    microsoft_client_secret: String,
    github_client_id: String,
    github_client_secret: String,
    redirect_uri: String,
}

impl OAuthService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            client: Client::new(),
            google_client_id: std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default(),
            google_client_secret: std::env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default(),
            microsoft_client_id: std::env::var("MICROSOFT_CLIENT_ID").unwrap_or_default(),
            microsoft_client_secret: std::env::var("MICROSOFT_CLIENT_SECRET").unwrap_or_default(),
            github_client_id: std::env::var("GITHUB_CLIENT_ID").unwrap_or_default(),
            github_client_secret: std::env::var("GITHUB_CLIENT_SECRET").unwrap_or_default(),
            redirect_uri: std::env::var("OAUTH_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:3000/auth/callback".to_string()),
        }
    }

    /// Get OAuth authorization URL
    pub fn get_authorization_url(&self, provider: OAuthProvider, state: &str) -> String {
        match provider {
            OAuthProvider::Google => {
                format!(
                    "https://accounts.google.com/o/oauth2/v2/auth?\
                    client_id={}&\
                    redirect_uri={}&\
                    response_type=code&\
                    scope=openid%20email%20profile&\
                    state={}",
                    self.google_client_id,
                    urlencoding::encode(&self.redirect_uri),
                    state
                )
            }
            OAuthProvider::Microsoft => {
                let tenant = std::env::var("MICROSOFT_TENANT_ID").unwrap_or_else(|_| "common".to_string());
                format!(
                    "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize?\
                    client_id={}&\
                    redirect_uri={}&\
                    response_type=code&\
                    scope=openid%20email%20profile&\
                    state={}",
                    tenant,
                    self.microsoft_client_id,
                    urlencoding::encode(&self.redirect_uri),
                    state
                )
            }
            OAuthProvider::GitHub => {
                format!(
                    "https://github.com/login/oauth/authorize?\
                    client_id={}&\
                    redirect_uri={}&\
                    scope=user:email&\
                    state={}",
                    self.github_client_id,
                    urlencoding::encode(&self.redirect_uri),
                    state
                )
            }
        }
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code_for_token(
        &self,
        provider: OAuthProvider,
        code: &str,
    ) -> Result<OAuthTokenResponse> {
        match provider {
            OAuthProvider::Google => self.exchange_google_code(code).await,
            OAuthProvider::Microsoft => self.exchange_microsoft_code(code).await,
            OAuthProvider::GitHub => self.exchange_github_code(code).await,
        }
    }

    async fn exchange_google_code(&self, code: &str) -> Result<OAuthTokenResponse> {
        let params = [
            ("client_id", self.google_client_id.as_str()),
            ("client_secret", self.google_client_secret.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", &self.redirect_uri),
        ];

        let response = self.client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Google token exchange failed: {}", error_text));
        }

        Ok(response.json().await?)
    }

    async fn exchange_microsoft_code(&self, code: &str) -> Result<OAuthTokenResponse> {
        let tenant = std::env::var("MICROSOFT_TENANT_ID").unwrap_or_else(|_| "common".to_string());
        let params = [
            ("client_id", self.microsoft_client_id.as_str()),
            ("client_secret", self.microsoft_client_secret.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", &self.redirect_uri),
        ];

        let response = self.client
            .post(&format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", tenant))
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Microsoft token exchange failed: {}", error_text));
        }

        Ok(response.json().await?)
    }

    async fn exchange_github_code(&self, code: &str) -> Result<OAuthTokenResponse> {
        let params = [
            ("client_id", self.github_client_id.as_str()),
            ("client_secret", self.github_client_secret.as_str()),
            ("code", code),
        ];

        let response = self.client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("GitHub token exchange failed: {}", error_text));
        }

        Ok(response.json().await?)
    }

    /// Get user info from provider
    pub async fn get_user_info(
        &self,
        provider: OAuthProvider,
        access_token: &str,
    ) -> Result<(String, String, String, Option<String>)> {
        match provider {
            OAuthProvider::Google => {
                let user_info: GoogleUserInfo = self.client
                    .get("https://www.googleapis.com/oauth2/v2/userinfo")
                    .bearer_auth(access_token)
                    .send()
                    .await?
                    .json()
                    .await?;
                
                Ok((user_info.sub, user_info.email, user_info.name, user_info.picture))
            }
            OAuthProvider::Microsoft => {
                let user_info: MicrosoftUserInfo = self.client
                    .get("https://graph.microsoft.com/v1.0/me")
                    .bearer_auth(access_token)
                    .send()
                    .await?
                    .json()
                    .await?;
                
                Ok((user_info.id, user_info.email, user_info.name, None))
            }
            OAuthProvider::GitHub => {
                let user_info: GitHubUserInfo = self.client
                    .get("https://api.github.com/user")
                    .header("User-Agent", "ConHub")
                    .bearer_auth(access_token)
                    .send()
                    .await?
                    .json()
                    .await?;
                
                let email = if let Some(email) = user_info.email {
                    email
                } else {
                    // Fetch primary email if not in profile
                    self.get_github_primary_email(access_token).await?
                };
                
                Ok((
                    user_info.id.to_string(),
                    email,
                    user_info.name.unwrap_or(user_info.login),
                    user_info.avatar_url,
                ))
            }
        }
    }

    async fn get_github_primary_email(&self, access_token: &str) -> Result<String> {
        #[derive(Deserialize)]
        struct GitHubEmail {
            email: String,
            primary: bool,
            verified: bool,
        }

        let emails: Vec<GitHubEmail> = self.client
            .get("https://api.github.com/user/emails")
            .header("User-Agent", "ConHub")
            .bearer_auth(access_token)
            .send()
            .await?
            .json()
            .await?;

        emails
            .into_iter()
            .find(|e| e.primary && e.verified)
            .map(|e| e.email)
            .ok_or_else(|| anyhow!("No verified primary email found"))
    }

    /// Find or create user from OAuth profile
    pub async fn find_or_create_oauth_user(
        &self,
        provider: OAuthProvider,
        provider_user_id: String,
        email: String,
        name: String,
        avatar_url: Option<String>,
    ) -> Result<User> {
        // Check if social connection exists
        let existing = sqlx::query!(
            r#"
            SELECT user_id FROM social_connections
            WHERE platform = $1 AND platform_user_id = $2 AND is_active = true
            "#,
            provider.to_string(),
            provider_user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(connection) = existing {
            // User exists, return it
            let user = sqlx::query!(
                r#"
                SELECT id, email, password_hash, name, avatar_url, organization,
                       role::text as role, subscription_tier::text as subscription_tier,
                       is_verified, is_active, created_at, updated_at, last_login_at
                FROM users
                WHERE id = $1 AND is_active = true
                "#,
                connection.user_id
            )
            .fetch_one(&self.pool)
            .await?;

            return Ok(User {
                id: user.id,
                email: user.email,
                password_hash: user.password_hash,
                name: user.name,
                avatar_url: user.avatar_url,
                organization: user.organization,
                role: user.role.unwrap_or_else(|| "user".to_string()),
                subscription_tier: user.subscription_tier.unwrap_or_else(|| "free".to_string()),
                is_verified: user.is_verified,
                is_active: user.is_active,
                created_at: user.created_at,
                updated_at: user.updated_at,
                last_login_at: user.last_login_at,
            });
        }

        // Check if user with email exists
        let existing_user = sqlx::query!(
            r#"
            SELECT id, email, password_hash, name, avatar_url, organization,
                   role::text as role, subscription_tier::text as subscription_tier,
                   is_verified, is_active, created_at, updated_at, last_login_at
            FROM users
            WHERE email = $1 AND is_active = true
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        let user_id = if let Some(user) = existing_user {
            user.id
        } else {
            // Create new user
            let new_user_id = Uuid::new_v4();
            let now = Utc::now();
            let dummy_password_hash = bcrypt::hash("oauth_user_no_password", bcrypt::DEFAULT_COST)?;

            sqlx::query!(
                r#"
                INSERT INTO users (
                    id, email, password_hash, name, avatar_url, organization,
                    role, subscription_tier, is_verified, is_active, created_at, updated_at
                ) VALUES (
                    $1, $2, $3, $4, $5, $6,
                    'user'::user_role, 'free'::subscription_tier, true, true, $7, $8
                )
                "#,
                new_user_id, email, dummy_password_hash, name,
                avatar_url.as_ref(), None::<String>, now, now
            )
            .execute(&self.pool)
            .await?;

            new_user_id
        };

        // Fetch the user
        let user = sqlx::query!(
            r#"
            SELECT id, email, password_hash, name, avatar_url, organization,
                   role::text as role, subscription_tier::text as subscription_tier,
                   is_verified, is_active, created_at, updated_at, last_login_at
            FROM users
            WHERE id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(User {
            id: user.id,
            email: user.email,
            password_hash: user.password_hash,
            name: user.name,
            avatar_url: user.avatar_url,
            organization: user.organization,
            role: user.role.unwrap_or_else(|| "user".to_string()),
            subscription_tier: user.subscription_tier.unwrap_or_else(|| "free".to_string()),
            is_verified: user.is_verified,
            is_active: user.is_active,
            created_at: user.created_at,
            updated_at: user.updated_at,
            last_login_at: user.last_login_at,
        })
    }

    /// Store OAuth tokens in social_connections table
    pub async fn store_oauth_connection(
        &self,
        user_id: Uuid,
        provider: OAuthProvider,
        provider_user_id: String,
        username: String,
        access_token: String,
        refresh_token: Option<String>,
        expires_in: Option<i64>,
        scope: Option<String>,
    ) -> Result<()> {
        let expires_at = expires_in.map(|seconds| Utc::now() + Duration::seconds(seconds));
        let now = Utc::now();

        // Upsert social connection
        sqlx::query!(
            r#"
            INSERT INTO social_connections (
                id, user_id, platform, platform_user_id, username,
                access_token, refresh_token, token_expires_at, scope,
                is_active, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12
            )
            ON CONFLICT (user_id, platform, platform_user_id)
            DO UPDATE SET
                access_token = EXCLUDED.access_token,
                refresh_token = EXCLUDED.refresh_token,
                token_expires_at = EXCLUDED.token_expires_at,
                scope = EXCLUDED.scope,
                is_active = true,
                updated_at = EXCLUDED.updated_at
            "#,
            Uuid::new_v4(),
            user_id,
            provider.to_string(),
            provider_user_id,
            username,
            access_token,
            refresh_token,
            expires_at,
            scope.unwrap_or_default(),
            true,
            now,
            now
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
