use redis::{Client, aio::ConnectionManager, AsyncCommands};
use anyhow::{Result, Context};
use serde::{Serialize, de::DeserializeOwned};
use std::time::Duration;

/// Redis cache wrapper with JSON serialization support
#[derive(Clone)]
pub struct RedisCache {
    manager: ConnectionManager,
}

impl RedisCache {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = Client::open(redis_url)
            .context("Failed to create Redis client")?;
        let manager = ConnectionManager::new(client)
            .await
            .context("Failed to connect to Redis")?;

        Ok(Self { manager })
    }

    /// Get a value from cache
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let mut conn = self.manager.clone();
        let value: Option<String> = conn.get(key).await?;

        match value {
            Some(json_str) => {
                let value = serde_json::from_str(&json_str)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Set a value in cache with TTL
    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Duration) -> Result<()> {
        let mut conn = self.manager.clone();
        let json_str = serde_json::to_string(value)?;
        conn.set_ex(key, json_str, ttl.as_secs()).await?;
        Ok(())
    }

    /// Set a value in cache without TTL
    pub async fn set_permanent<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let mut conn = self.manager.clone();
        let json_str = serde_json::to_string(value)?;
        conn.set(key, json_str).await?;
        Ok(())
    }

    /// Delete a value from cache
    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.manager.clone();
        conn.del(key).await?;
        Ok(())
    }

    /// Check if a key exists
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.manager.clone();
        Ok(conn.exists(key).await?)
    }

    /// Set expiry for a key
    pub async fn expire(&self, key: &str, ttl: Duration) -> Result<()> {
        let mut conn = self.manager.clone();
        conn.expire(key, ttl.as_secs() as i64).await?;
        Ok(())
    }

    /// Increment a counter
    pub async fn incr(&self, key: &str) -> Result<i64> {
        let mut conn = self.manager.clone();
        Ok(conn.incr(key, 1).await?)
    }

    /// Increment a counter with TTL
    pub async fn incr_with_ttl(&self, key: &str, ttl: Duration) -> Result<i64> {
        let mut conn = self.manager.clone();
        let val: i64 = conn.incr(key, 1).await?;
        if val == 1 {
            conn.expire(key, ttl.as_secs() as i64).await?;
        }
        Ok(val)
    }

    /// Get multiple keys at once
    pub async fn mget<T: DeserializeOwned>(&self, keys: &[&str]) -> Result<Vec<Option<T>>> {
        let mut conn = self.manager.clone();
        let values: Vec<Option<String>> = conn.get(keys).await?;

        values
            .into_iter()
            .map(|v| match v {
                Some(json_str) => serde_json::from_str(&json_str)
                    .map(Some)
                    .context("Failed to deserialize value"),
                None => Ok(None),
            })
            .collect()
    }

    /// Set multiple key-value pairs
    pub async fn mset<T: Serialize>(&self, pairs: &[(&str, &T)]) -> Result<()> {
        let mut conn = self.manager.clone();
        let serialized: Result<Vec<(String, String)>> = pairs
            .iter()
            .map(|(k, v)| {
                serde_json::to_string(v)
                    .map(|json_str| (k.to_string(), json_str))
                    .context("Failed to serialize value")
            })
            .collect();

        let serialized = serialized?;
        let refs: Vec<(&str, &str)> = serialized
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        conn.mset(&refs).await?;
        Ok(())
    }

    /// Flush all keys (use with caution!)
    pub async fn flush_all(&self) -> Result<()> {
        let mut conn = self.manager.clone();
        redis::cmd("FLUSHALL").query_async(&mut conn).await?;
        Ok(())
    }

    /// Get keys matching a pattern
    pub async fn keys(&self, pattern: &str) -> Result<Vec<String>> {
        let mut conn = self.manager.clone();
        Ok(conn.keys(pattern).await?)
    }

    /// Publish a message to a channel
    pub async fn publish(&self, channel: &str, message: &str) -> Result<()> {
        let mut conn = self.manager.clone();
        conn.publish(channel, message).await?;
        Ok(())
    }
}

/// Cache key builder for consistent key naming
pub struct CacheKeyBuilder;

impl CacheKeyBuilder {
    pub fn user(user_id: &uuid::Uuid) -> String {
        format!("user:{}", user_id)
    }

    pub fn session(session_id: &str) -> String {
        format!("session:{}", session_id)
    }

    pub fn connected_account(account_id: &uuid::Uuid) -> String {
        format!("connected_account:{}", account_id)
    }

    pub fn document(document_id: &uuid::Uuid) -> String {
        format!("document:{}", document_id)
    }

    pub fn rate_limit(user_id: &uuid::Uuid, action: &str) -> String {
        format!("rate_limit:{}:{}", user_id, action)
    }

    pub fn auth_token(token: &str) -> String {
        format!("auth:token:{}", token)
    }
}
