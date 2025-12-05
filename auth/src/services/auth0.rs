use anyhow::{Result, anyhow};
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

/// Auth0 configuration
#[derive(Debug, Clone)]
pub struct Auth0Config {
    pub domain: String,
    pub audience: String,
}

impl Auth0Config {
    pub fn from_env() -> Result<Self> {
        let domain = std::env::var("AUTH0_DOMAIN")
            .map_err(|_| anyhow!("AUTH0_DOMAIN environment variable not set"))?;
        let audience = std::env::var("AUTH0_AUDIENCE")
            .map_err(|_| anyhow!("AUTH0_AUDIENCE environment variable not set"))?;
        
        Ok(Self { domain, audience })
    }

    pub fn jwks_uri(&self) -> String {
        format!("https://{}/.well-known/jwks.json", self.domain)
    }

    pub fn issuer(&self) -> String {
        format!("https://{}/", self.domain)
    }
}

/// JWKS (JSON Web Key Set) response from Auth0
#[derive(Debug, Deserialize)]
struct JwksResponse {
    keys: Vec<Jwk>,
}

/// JSON Web Key
#[derive(Debug, Clone, Deserialize, Serialize)]
struct Jwk {
    kid: String,
    kty: String,
    #[serde(rename = "use")]
    key_use: Option<String>,
    n: String,
    e: String,
    alg: Option<String>,
}

/// Auth0 token claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Auth0Claims {
    pub sub: String,
    pub aud: serde_json::Value, // Can be string or array
    pub iss: String,
    pub exp: i64,
    pub iat: i64,
    pub email: Option<String>,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub email_verified: Option<bool>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// JWKS cache entry
#[derive(Clone)]
struct JwksCache {
    keys: Vec<Jwk>,
    fetched_at: DateTime<Utc>,
}

/// Auth0 service for token verification
pub struct Auth0Service {
    config: Auth0Config,
    client: Client,
    jwks_cache: Arc<RwLock<Option<JwksCache>>>,
    cache_ttl: Duration,
}

impl Auth0Service {
    pub fn new(config: Auth0Config) -> Self {
        Self {
            config,
            client: Client::new(),
            jwks_cache: Arc::new(RwLock::new(None)),
            cache_ttl: Duration::minutes(60), // Cache JWKS for 1 hour
        }
    }

    /// Fetch JWKS from Auth0
    async fn fetch_jwks(&self) -> Result<Vec<Jwk>> {
        tracing::info!("Fetching JWKS from Auth0: {}", self.config.jwks_uri());
        
        let response = self.client
            .get(&self.config.jwks_uri())
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch JWKS: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("JWKS fetch failed with status {}: {}", status, text));
        }

        let jwks: JwksResponse = response.json().await
            .map_err(|e| anyhow!("Failed to parse JWKS response: {}", e))?;

        tracing::info!("Successfully fetched {} keys from JWKS", jwks.keys.len());
        Ok(jwks.keys)
    }

    /// Get JWKS with caching
    async fn get_jwks(&self) -> Result<Vec<Jwk>> {
        // Check cache first
        {
            let cache = self.jwks_cache.read().await;
            if let Some(cached) = cache.as_ref() {
                let age = Utc::now() - cached.fetched_at;
                if age < self.cache_ttl {
                    tracing::debug!("Using cached JWKS (age: {}s)", age.num_seconds());
                    return Ok(cached.keys.clone());
                }
            }
        }

        // Cache miss or expired, fetch new keys
        let keys = self.fetch_jwks().await?;
        
        // Update cache
        {
            let mut cache = self.jwks_cache.write().await;
            *cache = Some(JwksCache {
                keys: keys.clone(),
                fetched_at: Utc::now(),
            });
        }

        Ok(keys)
    }

    /// Find JWK by key ID
    async fn find_jwk(&self, kid: &str) -> Result<Jwk> {
        let keys = self.get_jwks().await?;
        
        keys.into_iter()
            .find(|k| k.kid == kid)
            .ok_or_else(|| anyhow!("JWK with kid '{}' not found", kid))
    }

    /// Verify and decode Auth0 access token
    pub async fn verify_token(&self, token: &str) -> Result<Auth0Claims> {
        // Decode header to get kid
        let header = decode_header(token)
            .map_err(|e| anyhow!("Failed to decode token header: {}", e))?;

        let kid = header.kid
            .ok_or_else(|| anyhow!("Token missing 'kid' in header"))?;

        // Find the corresponding JWK
        let jwk = self.find_jwk(&kid).await?;

        // Convert JWK to DecodingKey
        let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
            .map_err(|e| anyhow!("Failed to create decoding key from JWK: {}", e))?;

        // Set up validation
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&self.config.issuer()]);
        
        // Audience can be string or array, handle both
        validation.set_audience(&[&self.config.audience]);
        validation.validate_exp = true;

        // Decode and validate token
        let token_data = decode::<Auth0Claims>(token, &decoding_key, &validation)
            .map_err(|e| anyhow!("Token validation failed: {}", e))?;

        tracing::info!("Successfully verified Auth0 token for sub: {}", token_data.claims.sub);
        Ok(token_data.claims)
    }

    /// Extract user info from Auth0 claims
    pub fn extract_user_info(&self, claims: &Auth0Claims) -> (String, Option<String>, Option<String>, Option<String>) {
        let sub = claims.sub.clone();
        let email = claims.email.clone().or_else(|| {
            Self::get_namespaced_claim(&claims.extra, "email")
        });
        let name = claims
            .name
            .clone()
            .or_else(|| Self::get_namespaced_claim(&claims.extra, "name"))
            .or_else(|| {
                // Fallback to email username if name not provided
                email
                    .as_ref()
                    .and_then(|e| e.split('@').next().map(|s| s.to_string()))
            });
        let picture = claims
            .picture
            .clone()
            .or_else(|| Self::get_namespaced_claim(&claims.extra, "picture"));

        (sub, email, name, picture)
    }

    fn get_namespaced_claim(
        extra: &HashMap<String, serde_json::Value>,
        suffix: &str,
    ) -> Option<String> {
        extra
            .iter()
            .find(|(k, _)| k.ends_with(suffix))
            .and_then(|(_, v)| v.as_str().map(|s| s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth0_config_from_env() {
        std::env::set_var("AUTH0_DOMAIN", "test.auth0.com");
        std::env::set_var("AUTH0_AUDIENCE", "https://api.test.com");

        let config = Auth0Config::from_env().unwrap();
        assert_eq!(config.domain, "test.auth0.com");
        assert_eq!(config.audience, "https://api.test.com");
        assert_eq!(config.issuer(), "https://test.auth0.com/");
        assert_eq!(config.jwks_uri(), "https://test.auth0.com/.well-known/jwks.json");
    }

    #[test]
    fn test_extract_user_info() {
        let config = Auth0Config {
            domain: "test.auth0.com".to_string(),
            audience: "https://api.test.com".to_string(),
        };
        let service = Auth0Service::new(config);

        let claims = Auth0Claims {
            sub: "auth0|123".to_string(),
            aud: serde_json::Value::String("https://api.test.com".to_string()),
            iss: "https://test.auth0.com/".to_string(),
            exp: 9999999999,
            iat: 1234567890,
            email: Some("test@example.com".to_string()),
            name: Some("Test User".to_string()),
            picture: Some("https://example.com/pic.jpg".to_string()),
            email_verified: Some(true),
            extra: HashMap::new(),
        };

        let (sub, email, name, picture) = service.extract_user_info(&claims);
        assert_eq!(sub, "auth0|123");
        assert_eq!(email, Some("test@example.com".to_string()));
        assert_eq!(name, Some("Test User".to_string()));
        assert_eq!(picture, Some("https://example.com/pic.jpg".to_string()));
    }
}
