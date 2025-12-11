use sqlx::{PgPool, Row};
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use reqwest::Client;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use sha2::{Sha256, Digest};

use bcrypt;

use conhub_models::auth::{User, UserRole, SubscriptionTier};

/// Generate a safe debug string for tokens (never logs full token)
/// Returns: "len=N, prefix=XXXXXX, sha256=XXXXXXXXXXXX"
pub fn token_debug(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    let len = token.len();
    let prefix = if len >= 6 { &token[..6] } else { token };
    format!("len={}, prefix={}..., sha256_prefix={}", len, prefix, &hash[..12])
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OAuthProvider {
    Google,
    Microsoft,
    GitHub,
    Bitbucket,
    GitLab,
}

impl std::fmt::Display for OAuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OAuthProvider::Google => write!(f, "google"),
            OAuthProvider::Microsoft => write!(f, "microsoft"),
            OAuthProvider::GitHub => write!(f, "github"),
            OAuthProvider::Bitbucket => write!(f, "bitbucket"),
            OAuthProvider::GitLab => write!(f, "gitlab"),
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
    bitbucket_client_id: String,
    bitbucket_client_secret: String,
    gitlab_client_id: String,
    gitlab_client_secret: String,
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
            bitbucket_client_id: std::env::var("BITBUCKET_CLIENT_ID").unwrap_or_default(),
            bitbucket_client_secret: std::env::var("BITBUCKET_CLIENT_SECRET").unwrap_or_default(),
            gitlab_client_id: std::env::var("GITLAB_CLIENT_ID").unwrap_or_default(),
            gitlab_client_secret: std::env::var("GITLAB_CLIENT_SECRET").unwrap_or_default(),
            redirect_uri: std::env::var("OAUTH_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:3000/auth/callback".to_string()),
        }
    }

    
    pub fn get_authorization_url(&self, provider: OAuthProvider, state: &str) -> String {
        let redirect_with_provider = format!("{}?provider={}", self.redirect_uri, provider);
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
                    urlencoding::encode(&redirect_with_provider),
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
                    urlencoding::encode(&redirect_with_provider),
                    state
                )
            }
            OAuthProvider::GitHub => {
                // Include 'repo' scope for repository access (private repos) 
                // and 'read:user' + 'user:email' for profile info
                let scopes = "repo read:user user:email";
                format!(
                    "https://github.com/login/oauth/authorize?\
                    client_id={}&\
                    redirect_uri={}&\
                    scope={}&\
                    state={}",
                    self.github_client_id,
                    urlencoding::encode(&redirect_with_provider),
                    urlencoding::encode(scopes),
                    state
                )
            }
            OAuthProvider::Bitbucket => {
                let scopes = ["repository:read", "account", "email"].join(" ");
                format!(
                    "https://bitbucket.org/site/oauth2/authorize?\
                    client_id={}&\
                    redirect_uri={}&\
                    response_type=code&\
                    scope={}&\
                    state={}",
                    self.bitbucket_client_id,
                    urlencoding::encode(&redirect_with_provider),
                    urlencoding::encode(&scopes),
                    state
                )
            }
            OAuthProvider::GitLab => {
                // Include 'read_repository' for repo access and 'read_user' for profile
                let scopes = "read_repository read_user";
                format!(
                    "https://gitlab.com/oauth/authorize?\
                    client_id={}&\
                    redirect_uri={}&\
                    response_type=code&\
                    scope={}&\
                    state={}",
                    self.gitlab_client_id,
                    urlencoding::encode(&redirect_with_provider),
                    urlencoding::encode(scopes),
                    state
                )
            }
        }
    }

    
    pub async fn exchange_code_for_token(
        &self,
        provider: OAuthProvider,
        code: &str,
    ) -> Result<OAuthTokenResponse> {
        match provider {
            OAuthProvider::Google => self.exchange_google_code(code).await,
            OAuthProvider::Microsoft => self.exchange_microsoft_code(code).await,
            OAuthProvider::GitHub => self.exchange_github_code(code).await,
            OAuthProvider::Bitbucket => self.exchange_bitbucket_code(code).await,
            OAuthProvider::GitLab => self.exchange_gitlab_code(code).await,
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
        // GitHub requires redirect_uri here if it was included in the authorization request.
        // Our auth flow uses OAUTH_REDIRECT_URI with a provider query parameter, e.g.
        //   http://localhost:3000/auth/callback?provider=github
        // so we must pass the exact same value during the token exchange.
        let redirect_with_provider = format!("{}?provider=github", self.redirect_uri);
        
        let code_preview = if code.len() > 10 { &code[..10] } else { code };
        tracing::info!(
            "[GitHub OAuth] Starting token exchange: client_id={}, redirect_uri={}, code_prefix={}...",
            self.github_client_id,
            redirect_with_provider,
            code_preview
        );

        let params = [
            ("client_id", self.github_client_id.as_str()),
            ("client_secret", self.github_client_secret.as_str()),
            ("code", code),
            ("redirect_uri", redirect_with_provider.as_str()),
        ];

        let response = self.client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await?;

        let status = response.status();
        let body_text = response.text().await?;
        
        tracing::debug!("[GitHub OAuth] Token endpoint response: status={}, body_len={}", status, body_text.len());

        if !status.is_success() {
            tracing::error!(
                "[GitHub OAuth] Token exchange HTTP error: status={}, body={}",
                status, body_text
            );
            return Err(anyhow!(
                "GitHub token exchange failed (status {}): {}",
                status,
                body_text
            ));
        }

        // GitHub returns errors as 200 OK with JSON containing "error" field
        // Check for error field first before trying to decode as OAuthTokenResponse
        let json_value: serde_json::Value = serde_json::from_str(&body_text)
            .map_err(|e| anyhow!("GitHub token exchange failed: invalid JSON response: {}", e))?;

        if let Some(error) = json_value.get("error").and_then(|v| v.as_str()) {
            let description = json_value
                .get("error_description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            tracing::error!(
                "[GitHub OAuth] Token exchange OAuth error: error={}, description={}",
                error, description
            );
            return Err(anyhow!(
                "GitHub OAuth error: {} - {}",
                error,
                description
            ));
        }

        // Now decode into OAuthTokenResponse
        let token_response: OAuthTokenResponse = serde_json::from_value(json_value)
            .map_err(|e| anyhow!("GitHub token exchange failed: could not decode response: {}", e))?;
        
        // Log token details (safely)
        tracing::info!(
            "[GitHub OAuth] ✅ Token exchange successful: token_type={}, scope={:?}, expires_in={:?}, token_debug={}",
            token_response.token_type,
            token_response.scope,
            token_response.expires_in,
            token_debug(&token_response.access_token)
        );
        
        Ok(token_response)
    }

    async fn exchange_bitbucket_code(&self, code: &str) -> Result<OAuthTokenResponse> {
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &self.redirect_uri),
        ];

        let response = self.client
            .post("https://bitbucket.org/site/oauth2/access_token")
            .basic_auth(&self.bitbucket_client_id, Some(&self.bitbucket_client_secret))
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Bitbucket token exchange failed: {}", error_text));
        }

        Ok(response.json().await?)
    }

    async fn exchange_gitlab_code(&self, code: &str) -> Result<OAuthTokenResponse> {
        let params = [
            ("client_id", self.gitlab_client_id.as_str()),
            ("client_secret", self.gitlab_client_secret.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", &self.redirect_uri),
        ];

        let response = self.client
            .post("https://gitlab.com/oauth/token")
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("GitLab token exchange failed: {}", error_text));
        }

        Ok(response.json().await?)
    }

    
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
                    
                    self.get_github_primary_email(access_token).await?
                };
                
                Ok((
                    user_info.id.to_string(),
                    email,
                    user_info.name.unwrap_or(user_info.login),
                    user_info.avatar_url,
                ))
            }
            OAuthProvider::Bitbucket => {
                let user: serde_json::Value = self.client
                    .get("https://api.bitbucket.org/2.0/user")
                    .bearer_auth(access_token)
                    .send()
                    .await?
                    .json()
                    .await?;

                let emails: serde_json::Value = self.client
                    .get("https://api.bitbucket.org/2.0/user/emails")
                    .bearer_auth(access_token)
                    .send()
                    .await?
                    .json()
                    .await?;

                let email = emails["values"].as_array().unwrap_or(&vec![])
                    .iter()
                    .find(|e| e["is_primary"].as_bool().unwrap_or(false) && e["is_confirmed"].as_bool().unwrap_or(false))
                    .and_then(|e| e["email"].as_str())
                    .unwrap_or("")
                    .to_string();

                Ok((
                    user["uuid"].as_str().unwrap_or("").to_string(),
                    email,
                    user["display_name"].as_str().unwrap_or("").to_string(),
                    None,
                ))
            }
            OAuthProvider::GitLab => {
                let user: serde_json::Value = self.client
                    .get("https://gitlab.com/api/v4/user")
                    .bearer_auth(access_token)
                    .send()
                    .await?
                    .json()
                    .await?;

                let email = user["email"].as_str().unwrap_or("").to_string();
                let name = user["name"].as_str().unwrap_or("").to_string();
                let avatar_url = user["avatar_url"].as_str().map(|s| s.to_string());

                Ok((
                    user["id"].to_string(),
                    email,
                    name,
                    avatar_url,
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

    
    pub async fn find_or_create_oauth_user(
        &self,
        provider: OAuthProvider,
        provider_user_id: String,
        email: String,
        name: String,
        avatar_url: Option<String>,
    ) -> Result<User> {
        
        let existing = sqlx::query(
            r#"
            SELECT user_id FROM social_connections
            WHERE platform = $1 AND platform_user_id = $2 AND is_active = true
            "#
        )
        .bind(provider.to_string())
        .bind(provider_user_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(connection) = existing {
            let user_id: Uuid = connection.get("user_id");
            
            let user = sqlx::query(
                r#"
                SELECT id, email, password_hash, name, avatar_url, organization,
                       role::text as role, subscription_tier::text as subscription_tier,
                       is_verified, is_active, created_at, updated_at, last_login_at
                FROM users
                WHERE id = $1 AND is_active = true
                "#
            )
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

            let role = match user.get::<Option<String>, _>("role").as_deref() {
                Some("admin") => UserRole::Admin,
                _ => UserRole::User,
            };

            let subscription_tier = match user.get::<Option<String>, _>("subscription_tier").as_deref() {
                Some("personal") => SubscriptionTier::Personal,
                Some("team") => SubscriptionTier::Team,
                Some("enterprise") => SubscriptionTier::Enterprise,
                _ => SubscriptionTier::Free,
            };

            return Ok(User {
                id: user.get("id"),
                email: user.get("email"),
                password_hash: user.get("password_hash"),
                name: user.get("name"),
                avatar_url: user.get("avatar_url"),
                organization: user.get("organization"),
                role,
                subscription_tier,
                is_verified: user.get("is_verified"),
                is_active: user.get("is_active"),
                is_locked: user.get("is_locked"),
                failed_login_attempts: user.get("failed_login_attempts"),
                locked_until: user.get("locked_until"),
                password_changed_at: user.get("password_changed_at"),
                email_verified_at: user.get("email_verified_at"),
                two_factor_enabled: user.get("two_factor_enabled"),
                two_factor_secret: user.get("two_factor_secret"),
                backup_codes: user.get("backup_codes"),
                created_at: user.get("created_at"),
                updated_at: user.get("updated_at"),
                last_login_at: user.get("last_login_at"),
                last_login_ip: user.get("last_login_ip"),
                last_password_reset: user.get("last_password_reset"),
            });
        }

        
        let existing_user = sqlx::query(
            r#"
            SELECT id, email, password_hash, name, avatar_url, organization,
                   role::text as role, subscription_tier::text as subscription_tier,
                   is_verified, is_active, created_at, updated_at, last_login_at
            FROM users
            WHERE email = $1 AND is_active = true
            "#
        )
        .bind(&email)
        .fetch_optional(&self.pool)
        .await?;

        let user_id = if let Some(user) = existing_user {
            user.get("id")
        } else {
            
            let new_user_id = Uuid::new_v4();
            let now = Utc::now();
            let dummy_password_hash = bcrypt::hash("oauth_user_no_password", bcrypt::DEFAULT_COST)?;

            sqlx::query(
                r#"
                INSERT INTO users (
                    id, email, password_hash, name, avatar_url, organization,
                    role, subscription_tier, is_verified, is_active, created_at, updated_at
                ) VALUES (
                    $1, $2, $3, $4, $5, $6,
                    'user'::user_role, 'free'::subscription_tier, true, true, $7, $8
                )
                "#
            )
            .bind(new_user_id)
            .bind(email)
            .bind(dummy_password_hash)
            .bind(name)
            .bind(avatar_url.as_ref())
            .bind(None::<String>)
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await?;

            new_user_id
        };

        
        let user = sqlx::query(
            r#"
            SELECT id, email, password_hash, name, avatar_url, organization,
                   role::text as role, subscription_tier::text as subscription_tier,
                   is_verified, is_active, is_locked, failed_login_attempts, locked_until,
                   password_changed_at, email_verified_at, two_factor_enabled, two_factor_secret,
                   backup_codes, created_at, updated_at, last_login_at, last_login_ip::text as last_login_ip, last_password_reset
            FROM users
            WHERE id = $1
            "#
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(User {
            id: user.get("id"),
            email: user.get("email"),
            password_hash: user.get("password_hash"),
            name: user.get("name"),
            avatar_url: user.get("avatar_url"),
            organization: user.get("organization"),
            role: match user.get::<String, _>("role").as_str() {
                "admin" => UserRole::Admin,
                _ => UserRole::User,
            },
            subscription_tier: match user.get::<String, _>("subscription_tier").as_str() {
                "personal" => SubscriptionTier::Personal,
                "team" => SubscriptionTier::Team,
                "enterprise" => SubscriptionTier::Enterprise,
                _ => SubscriptionTier::Free,
            },
            is_verified: user.get("is_verified"),
            is_active: user.get("is_active"),
            is_locked: user.get("is_locked"),
            failed_login_attempts: user.get("failed_login_attempts"),
            locked_until: user.get("locked_until"),
            password_changed_at: user.get("password_changed_at"),
            email_verified_at: user.get("email_verified_at"),
            two_factor_enabled: user.get("two_factor_enabled"),
            two_factor_secret: user.get("two_factor_secret"),
            backup_codes: user.get("backup_codes"),
            created_at: user.get("created_at"),
            updated_at: user.get("updated_at"),
            last_login_at: user.get("last_login_at"),
            last_login_ip: user.get("last_login_ip"),
            last_password_reset: user.get("last_password_reset"),
        })
    }

    
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

        
        sqlx::query(
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
            "#
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(provider.to_string())
        .bind(provider_user_id)
        .bind(username)
        .bind(access_token)
        .bind(refresh_token)
        .bind(expires_at)
        .bind(scope.unwrap_or_default())
        .bind(true)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
