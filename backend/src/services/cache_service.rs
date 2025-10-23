use redis::{Client, Connection, Commands, RedisError};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::sync::{Arc, Mutex};

/// Cache service providing Redis-backed caching for API responses and data source results
pub struct CacheService {
    client: Client,
    connection: Arc<Mutex<Connection>>,
    default_ttl: Duration,
}

#[derive(Debug)]
pub enum CacheError {
    ConnectionError(String),
    SerializationError(String),
    RedisError(String),
    InvalidKey(String),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::ConnectionError(msg) => write!(f, "Cache connection error: {}", msg),
            CacheError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            CacheError::RedisError(msg) => write!(f, "Redis error: {}", msg),
            CacheError::InvalidKey(msg) => write!(f, "Invalid cache key: {}", msg),
        }
    }
}

impl std::error::Error for CacheError {}

impl From<RedisError> for CacheError {
    fn from(err: RedisError) -> Self {
        CacheError::RedisError(err.to_string())
    }
}

impl CacheService {
    /// Create a new cache service connected to Redis
    pub fn new(redis_url: &str) -> Result<Self, CacheError> {
        let client = Client::open(redis_url)
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

        let connection = client
            .get_connection()
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

        Ok(Self {
            client,
            connection: Arc::new(Mutex::new(connection)),
            default_ttl: Duration::from_secs(300), // 5 minutes default
        })
    }

    /// Create cache service from environment variable
    pub fn from_env() -> Result<Self, CacheError> {
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        Self::new(&redis_url)
    }

    /// Get a value from cache
    pub fn get<T>(&self, key: &str) -> Result<Option<T>, CacheError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let full_key = self.build_key(key);

        let mut conn = self.connection
            .lock()
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

        let value: Option<String> = conn.get(&full_key)?;

        match value {
            Some(json_str) => {
                let deserialized = serde_json::from_str(&json_str)
                    .map_err(|e| CacheError::SerializationError(e.to_string()))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    /// Set a value in cache with default TTL
    pub fn set<T>(&self, key: &str, value: &T) -> Result<(), CacheError>
    where
        T: Serialize,
    {
        self.set_with_ttl(key, value, self.default_ttl)
    }

    /// Set a value in cache with custom TTL
    pub fn set_with_ttl<T>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), CacheError>
    where
        T: Serialize,
    {
        let full_key = self.build_key(key);

        let json_str = serde_json::to_string(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        let mut conn = self.connection
            .lock()
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

        conn.set_ex(&full_key, json_str, ttl.as_secs() as usize)?;

        Ok(())
    }

    /// Delete a value from cache
    pub fn delete(&self, key: &str) -> Result<(), CacheError> {
        let full_key = self.build_key(key);

        let mut conn = self.connection
            .lock()
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

        conn.del(&full_key)?;

        Ok(())
    }

    /// Delete all keys matching a pattern
    pub fn delete_pattern(&self, pattern: &str) -> Result<u64, CacheError> {
        let full_pattern = self.build_key(pattern);

        let mut conn = self.connection
            .lock()
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

        // Get all matching keys
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&full_pattern)
            .query(&mut *conn)?;

        if keys.is_empty() {
            return Ok(0);
        }

        // Delete all matching keys
        let count: u64 = conn.del(&keys)?;

        Ok(count)
    }

    /// Check if a key exists in cache
    pub fn exists(&self, key: &str) -> Result<bool, CacheError> {
        let full_key = self.build_key(key);

        let mut conn = self.connection
            .lock()
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

        let exists: bool = conn.exists(&full_key)?;

        Ok(exists)
    }

    /// Get remaining TTL for a key
    pub fn ttl(&self, key: &str) -> Result<Option<Duration>, CacheError> {
        let full_key = self.build_key(key);

        let mut conn = self.connection
            .lock()
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

        let ttl_secs: i64 = conn.ttl(&full_key)?;

        if ttl_secs < 0 {
            Ok(None) // Key doesn't exist or has no expiry
        } else {
            Ok(Some(Duration::from_secs(ttl_secs as u64)))
        }
    }

    /// Extend TTL for an existing key
    pub fn extend_ttl(&self, key: &str, additional_ttl: Duration) -> Result<(), CacheError> {
        let full_key = self.build_key(key);

        let mut conn = self.connection
            .lock()
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

        conn.expire(&full_key, additional_ttl.as_secs() as usize)?;

        Ok(())
    }

    /// Clear all cache entries
    pub fn clear_all(&self) -> Result<(), CacheError> {
        let mut conn = self.connection
            .lock()
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

        redis::cmd("FLUSHDB").query(&mut *conn)?;

        Ok(())
    }

    /// Build full cache key with namespace prefix
    fn build_key(&self, key: &str) -> String {
        format!("conhub:cache:{}", key)
    }

    /// Helper to create a data source cache key
    pub fn datasource_key(source_type: &str, source_id: &str, endpoint: &str) -> String {
        format!("datasource:{}:{}:{}", source_type, source_id, endpoint)
    }

    /// Helper to create a MCP resource cache key
    pub fn mcp_resource_key(server_id: &str, uri: &str) -> String {
        format!("mcp:resource:{}:{}", server_id, uri)
    }

    /// Helper to create a user-specific cache key
    pub fn user_key(user_id: &str, key: &str) -> String {
        format!("user:{}:{}", user_id, key)
    }
}

/// TTL configurations for different types of cached data
pub struct CacheTtl;

impl CacheTtl {
    /// Project/repository metadata: 10 minutes
    pub const PROJECT_METADATA: Duration = Duration::from_secs(600);

    /// Branch list: 5 minutes
    pub const BRANCH_LIST: Duration = Duration::from_secs(300);

    /// File contents: 30 minutes
    pub const FILE_CONTENT: Duration = Duration::from_secs(1800);

    /// Folder/directory listings: 5 minutes
    pub const FOLDER_LISTING: Duration = Duration::from_secs(300);

    /// User profile data: 1 hour
    pub const USER_PROFILE: Duration = Duration::from_secs(3600);

    /// Search results: 15 minutes
    pub const SEARCH_RESULTS: Duration = Duration::from_secs(900);

    /// API rate limit tracking: Until reset time (variable)
    pub const RATE_LIMIT: Duration = Duration::from_secs(3600);

    /// Webhook configurations: 1 hour
    pub const WEBHOOK_CONFIG: Duration = Duration::from_secs(3600);
}

/// Helper trait for cache-aware data structures
pub trait Cacheable: Serialize + for<'de> Deserialize<'de> {
    /// Get cache key for this item
    fn cache_key(&self) -> String;

    /// Get TTL for this item type
    fn cache_ttl() -> Duration;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestData {
        id: String,
        value: String,
    }

    #[test]
    fn test_build_key() {
        let cache = CacheService::new("redis://localhost:6379").unwrap();
        assert_eq!(cache.build_key("test"), "conhub:cache:test");
    }

    #[test]
    fn test_datasource_key() {
        let key = CacheService::datasource_key("github", "123", "repos");
        assert_eq!(key, "datasource:github:123:repos");
    }

    #[test]
    fn test_mcp_resource_key() {
        let key = CacheService::mcp_resource_key("server-1", "file://test.txt");
        assert_eq!(key, "mcp:resource:server-1:file://test.txt");
    }

    #[test]
    fn test_user_key() {
        let key = CacheService::user_key("user-123", "preferences");
        assert_eq!(key, "user:user-123:preferences");
    }

    // Note: Integration tests requiring Redis connection should be in a separate test suite
}
