use async_trait::async_trait;
use reqwest::{Request, Response, Client};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::collections::HashMap;

/// Enhanced connector trait providing advanced features for data source connectors
/// Includes pagination, rate limiting, caching, and webhook support
#[async_trait]
pub trait EnhancedConnector: Send + Sync {
    /// Fetch paginated results from an API endpoint
    /// Automatically handles pagination and returns all results
    async fn fetch_paginated<T>(
        &self,
        url: &str,
        page_size: usize,
    ) -> Result<Vec<T>, ConnectorError>
    where
        T: for<'de> Deserialize<'de> + Send;

    /// Execute a rate-limited HTTP request
    /// Respects provider-specific rate limits and retries on 429
    async fn rate_limited_request(
        &self,
        request: Request,
    ) -> Result<Response, ConnectorError>;

    /// Get cached value by key
    async fn get_cached<T>(&self, key: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>;

    /// Set cache value with TTL
    async fn set_cache<T>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), ConnectorError>
    where
        T: Serialize;

    /// Register a webhook for this data source
    async fn register_webhook(
        &self,
        callback_url: &str,
    ) -> Result<String, ConnectorError>;

    /// Verify webhook signature
    async fn verify_webhook_signature(
        &self,
        payload: &[u8],
        signature: &str,
    ) -> bool;

    /// Clear cache for this connector
    async fn clear_cache(&self) -> Result<(), ConnectorError>;
}

/// Errors that can occur in enhanced connectors
#[derive(Debug, Clone)]
pub enum ConnectorError {
    /// Network or HTTP error
    NetworkError(String),

    /// Rate limit exceeded
    RateLimitExceeded { retry_after: Option<Duration> },

    /// Authentication failed
    AuthenticationError(String),

    /// API returned an error
    ApiError { code: u16, message: String },

    /// Cache operation failed
    CacheError(String),

    /// Webhook operation failed
    WebhookError(String),

    /// Serialization/deserialization error
    SerializationError(String),

    /// General error
    Other(String),
}

impl std::fmt::Display for ConnectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectorError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ConnectorError::RateLimitExceeded { retry_after } => {
                write!(f, "Rate limit exceeded")?;
                if let Some(duration) = retry_after {
                    write!(f, ", retry after {:?}", duration)?;
                }
                Ok(())
            }
            ConnectorError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            ConnectorError::ApiError { code, message } => {
                write!(f, "API error {}: {}", code, message)
            }
            ConnectorError::CacheError(msg) => write!(f, "Cache error: {}", msg),
            ConnectorError::WebhookError(msg) => write!(f, "Webhook error: {}", msg),
            ConnectorError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            ConnectorError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ConnectorError {}

/// Pagination information extracted from API responses
#[derive(Debug, Clone)]
pub struct PaginationInfo {
    pub has_more: bool,
    pub next_cursor: Option<String>,
    pub next_page: Option<u32>,
    pub next_link: Option<String>,
}

/// Rate limit information
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    pub reset_at: chrono::DateTime<chrono::Utc>,
}

impl RateLimitInfo {
    /// Calculate duration until rate limit resets
    pub fn time_until_reset(&self) -> Duration {
        let now = chrono::Utc::now();
        let duration = self.reset_at.signed_duration_since(now);

        if duration.num_seconds() > 0 {
            Duration::from_secs(duration.num_seconds() as u64)
        } else {
            Duration::from_secs(0)
        }
    }

    /// Check if we're approaching the rate limit
    pub fn is_near_limit(&self, threshold: f32) -> bool {
        let usage_ratio = 1.0 - (self.remaining as f32 / self.limit as f32);
        usage_ratio > threshold
    }
}

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub id: String,
    pub callback_url: String,
    pub events: Vec<String>,
    pub secret: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Delta sync cursor for incremental updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaSyncCursor {
    pub cursor: String,
    pub last_sync: chrono::DateTime<chrono::Utc>,
    pub data_source_id: String,
}

/// Helper struct for building HTTP requests with common patterns
pub struct RequestBuilder {
    client: Client,
    base_url: String,
    auth_headers: HashMap<String, String>,
}

impl RequestBuilder {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            auth_headers: HashMap::new(),
        }
    }

    pub fn with_bearer_token(mut self, token: String) -> Self {
        self.auth_headers
            .insert("Authorization".to_string(), format!("Bearer {}", token));
        self
    }

    pub fn with_api_key(mut self, key: String, header_name: &str) -> Self {
        self.auth_headers
            .insert(header_name.to_string(), key);
        self
    }

    pub fn with_basic_auth(mut self, username: String, password: String) -> Self {
        let credentials = base64::encode(format!("{}:{}", username, password));
        self.auth_headers
            .insert("Authorization".to_string(), format!("Basic {}", credentials));
        self
    }

    pub async fn get(&self, path: &str) -> Result<Response, ConnectorError> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), path.trim_start_matches('/'));

        let mut request = self.client.get(&url);
        for (key, value) in &self.auth_headers {
            request = request.header(key, value);
        }

        request
            .send()
            .await
            .map_err(|e| ConnectorError::NetworkError(e.to_string()))
    }

    pub async fn post<T: Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<Response, ConnectorError> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), path.trim_start_matches('/'));

        let mut request = self.client.post(&url).json(body);
        for (key, value) in &self.auth_headers {
            request = request.header(key, value);
        }

        request
            .send()
            .await
            .map_err(|e| ConnectorError::NetworkError(e.to_string()))
    }
}

/// Helper function to extract rate limit info from response headers
pub fn extract_rate_limit_info(response: &Response) -> Option<RateLimitInfo> {
    let headers = response.headers();

    // Try different common rate limit header patterns
    let limit = headers
        .get("x-ratelimit-limit")
        .or_else(|| headers.get("ratelimit-limit"))
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u32>().ok())?;

    let remaining = headers
        .get("x-ratelimit-remaining")
        .or_else(|| headers.get("ratelimit-remaining"))
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u32>().ok())?;

    let reset = headers
        .get("x-ratelimit-reset")
        .or_else(|| headers.get("ratelimit-reset"))
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok())
        .and_then(|timestamp| chrono::DateTime::from_timestamp(timestamp, 0))?;

    Some(RateLimitInfo {
        limit,
        remaining,
        reset_at: reset,
    })
}

/// Helper function to parse Link header for pagination
pub fn parse_link_header(link_header: &str) -> HashMap<String, String> {
    let mut links = HashMap::new();

    for part in link_header.split(',') {
        let parts: Vec<&str> = part.split(';').collect();
        if parts.len() != 2 {
            continue;
        }

        let url = parts[0].trim().trim_start_matches('<').trim_end_matches('>');
        let rel = parts[1]
            .trim()
            .trim_start_matches("rel=\"")
            .trim_end_matches('"');

        links.insert(rel.to_string(), url.to_string());
    }

    links
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_link_header() {
        let header = r#"<https://api.github.com/repos?page=2>; rel="next", <https://api.github.com/repos?page=5>; rel="last""#;
        let links = parse_link_header(header);

        assert_eq!(links.len(), 2);
        assert_eq!(
            links.get("next"),
            Some(&"https://api.github.com/repos?page=2".to_string())
        );
        assert_eq!(
            links.get("last"),
            Some(&"https://api.github.com/repos?page=5".to_string())
        );
    }

    #[test]
    fn test_rate_limit_near_limit() {
        let rate_limit = RateLimitInfo {
            limit: 100,
            remaining: 10,
            reset_at: chrono::Utc::now() + chrono::Duration::hours(1),
        };

        assert!(rate_limit.is_near_limit(0.8)); // 90% used
        assert!(!rate_limit.is_near_limit(0.95)); // Only 90% used, not 95%
    }
}
