use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, error, warn};
use uuid::Uuid;

/// Response from internal OAuth token endpoint
#[derive(Debug, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub refresh_token: Option<String>,
}

/// Response from internal OAuth status endpoint
#[derive(Debug, Deserialize)]
pub struct OAuthStatusResponse {
    pub connected: bool,
    pub connection_id: Option<Uuid>,
    pub username: Option<String>,
    pub is_expired: Option<bool>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Client for communicating with the auth service's internal endpoints
#[derive(Clone)]
pub struct AuthClient {
    client: Client,
    base_url: String,
}

impl AuthClient {
    /// Create a new auth client
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Self {
        let base_url = std::env::var("AUTH_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:3010".to_string());
        Self::new(base_url)
    }

    /// Get OAuth token for a user and provider
    /// 
    /// This calls the internal auth endpoint to retrieve the stored OAuth token
    /// for the specified user and provider (e.g., "github", "bitbucket")
    pub async fn get_oauth_token(
        &self,
        user_id: Uuid,
        provider: &str,
    ) -> Result<OAuthTokenResponse, AuthClientError> {
        self.get_oauth_token_with_correlation(user_id, provider, "none").await
    }
    
    /// Get OAuth token with correlation ID for tracing
    pub async fn get_oauth_token_with_correlation(
        &self,
        user_id: Uuid,
        provider: &str,
        correlation_id: &str,
    ) -> Result<OAuthTokenResponse, AuthClientError> {
        let url = format!(
            "{}/internal/oauth/{}/token?user_id={}",
            self.base_url, provider, user_id
        );

        info!(
            "[AuthClient][{}] 🔑 Fetching {} token: user_id={}, url={}",
            correlation_id, provider, user_id, url
        );

        let response = self.client
            .get(&url)
            .header("x-correlation-id", correlation_id)
            .send()
            .await
            .map_err(|e| {
                error!(
                    "[AuthClient][{}] ❌ HTTP request failed: provider={}, user_id={}, error={}",
                    correlation_id, provider, user_id, e
                );
                AuthClientError::RequestFailed(e.to_string())
            })?;

        let status = response.status();
        
        info!(
            "[AuthClient][{}] 📡 Auth service response: status={}",
            correlation_id, status
        );
        
        if status.is_success() {
            let token_response: OAuthTokenResponse = response
                .json()
                .await
                .map_err(|e| {
                    error!(
                        "[AuthClient][{}] ❌ Failed to parse token response: error={}",
                        correlation_id, e
                    );
                    AuthClientError::ParseError(e.to_string())
                })?;
            
            // Generate safe token debug
            let token_len = token_response.access_token.len();
            let token_prefix = if token_len >= 6 { &token_response.access_token[..6] } else { &token_response.access_token };
            
            info!(
                "[AuthClient][{}] ✅ Got token from auth service: provider={}, user_id={}, token_len={}, token_prefix={}..., expires_at={:?}",
                correlation_id, provider, user_id, token_len, token_prefix, token_response.expires_at
            );
            Ok(token_response)
        } else if status.as_u16() == 404 {
            // Parse the error response to check for code field
            let error_text = response.text().await.unwrap_or_else(|_| "{}".to_string());
            
            info!(
                "[AuthClient][{}] 📋 Auth service 404 response: body={}",
                correlation_id, error_text
            );
            
            let error_code = serde_json::from_str::<serde_json::Value>(&error_text)
                .ok()
                .and_then(|v| v.get("code").and_then(|c| c.as_str()).map(|s| s.to_string()));
            
            match error_code.as_deref() {
                Some("token_expired") => {
                    warn!(
                        "[AuthClient][{}] ⚠️ Token expired: provider={}, user_id={}",
                        correlation_id, provider, user_id
                    );
                    Err(AuthClientError::TokenExpired(provider.to_string()))
                }
                Some("connection_inactive") => {
                    warn!(
                        "[AuthClient][{}] ⚠️ Connection inactive: provider={}, user_id={}",
                        correlation_id, provider, user_id
                    );
                    Err(AuthClientError::NoConnection(provider.to_string()))
                }
                _ => {
                    warn!(
                        "[AuthClient][{}] ⚠️ No connection found: provider={}, user_id={}, code={:?}",
                        correlation_id, provider, user_id, error_code
                    );
                    Err(AuthClientError::NoConnection(provider.to_string()))
                }
            }
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                "[AuthClient][{}] ❌ Auth service error: status={}, body={}",
                correlation_id, status, error_text
            );
            Err(AuthClientError::ServiceError(status.as_u16(), error_text))
        }
    }

    /// Check OAuth connection status for a user and provider
    pub async fn check_oauth_status(
        &self,
        user_id: Uuid,
        provider: &str,
    ) -> Result<OAuthStatusResponse, AuthClientError> {
        let url = format!(
            "{}/internal/oauth/{}/status?user_id={}",
            self.base_url, provider, user_id
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| AuthClientError::RequestFailed(e.to_string()))?;

        let status = response.status();
        
        if status.is_success() {
            let status_response: OAuthStatusResponse = response
                .json()
                .await
                .map_err(|e| AuthClientError::ParseError(e.to_string()))?;
            Ok(status_response)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(AuthClientError::ServiceError(status.as_u16(), error_text))
        }
    }

    /// Check if auth service is healthy
    pub async fn health_check(&self) -> Result<bool, AuthClientError> {
        let url = format!("{}/health", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

/// Errors that can occur when communicating with auth service
#[derive(Debug, thiserror::Error)]
pub enum AuthClientError {
    #[error("Request to auth service failed: {0}")]
    RequestFailed(String),

    #[error("No {0} connection found for user")]
    NoConnection(String),

    #[error("{0} token has expired")]
    TokenExpired(String),

    #[error("Auth service returned error {0}: {1}")]
    ServiceError(u16, String),

    #[error("Failed to parse auth service response: {0}")]
    ParseError(String),
}
