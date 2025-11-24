use anyhow::Result;
use redis::{Client, AsyncCommands, aio::ConnectionManager};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

/// Redis-based connector response cache
/// 
/// Caches API responses from external connectors (GitHub, Bitbucket, Slack, etc.)
/// to reduce API calls and avoid rate limits.
pub struct ConnectorCache {
    client: Client,
    conn: Option<ConnectionManager>,
    enabled: bool,
    ttl_seconds: u64,
}

impl ConnectorCache {
    pub async fn new(redis_url: Option<String>) -> Self {
        let redis_url = redis_url.unwrap_or_else(|| {
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string())
        });

        match Client::open(redis_url.as_str()) {
            Ok(client) => {
                match ConnectionManager::new(client.clone()).await {
                    Ok(conn) => {
                        tracing::info!("✅ Connector cache connected to Redis");
                        Self {
                            client,
                            conn: Some(conn),
                            enabled: true,
                            ttl_seconds: 1800, // 30 minutes default for connector responses
                        }
                    }
                    Err(e) => {
                        tracing::warn!("⚠️  Redis connection failed: {}, connector cache disabled", e);
                        Self {
                            client,
                            conn: None,
                            enabled: false,
                            ttl_seconds: 1800,
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("⚠️  Redis client creation failed: {}, connector cache disabled", e);
                Self {
                    client: Client::open("redis://localhost").unwrap(),
                    conn: None,
                    enabled: false,
                    ttl_seconds: 1800,
                }
            }
        }
    }

    /// Get cached connector response
    pub async fn get<T>(&mut self, connector_type: &str, endpoint: &str, params: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        if !self.enabled || self.conn.is_none() {
            return Ok(None);
        }

        let key = Self::generate_cache_key(connector_type, endpoint, params);
        
        if let Some(ref mut conn) = self.conn {
            match conn.get::<_, Option<String>>(&key).await {
                Ok(Some(cached_json)) => {
                    match serde_json::from_str::<T>(&cached_json) {
                        Ok(data) => {
                            tracing::debug!("🎯 Connector cache HIT: {}", key);
                            Ok(Some(data))
                        }
                        Err(e) => {
                            tracing::warn!("⚠️  Failed to deserialize cached response: {}", e);
                            Ok(None)
                        }
                    }
                }
                Ok(None) => {
                    tracing::debug!("❌ Connector cache MISS: {}", key);
                    Ok(None)
                }
                Err(e) => {
                    tracing::warn!("⚠️  Redis GET error: {}", e);
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Set connector response in cache
    pub async fn set<T>(&mut self, connector_type: &str, endpoint: &str, params: &str, data: &T) -> Result<()>
    where
        T: Serialize,
    {
        if !self.enabled || self.conn.is_none() {
            return Ok(());
        }

        let key = Self::generate_cache_key(connector_type, endpoint, params);
        let data_json = serde_json::to_string(data)?;

        if let Some(ref mut conn) = self.conn {
            match conn.set_ex::<_, _, ()>(&key, data_json, self.ttl_seconds as usize).await {
                Ok(_) => {
                    tracing::debug!("💾 Cached connector response: {}", key);
                    Ok(())
                }
                Err(e) => {
                    tracing::warn!("⚠️  Redis SET error: {}", e);
                    Ok(()) // Don't fail if cache write fails
                }
            }
        } else {
            Ok(())
        }
    }

    /// Generate cache key from connector type, endpoint, and parameters
    fn generate_cache_key(connector_type: &str, endpoint: &str, params: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(connector_type.as_bytes());
        hasher.update(endpoint.as_bytes());
        hasher.update(params.as_bytes());
        
        let hash = format!("{:x}", hasher.finalize());
        format!("connector:{}:{}", connector_type, &hash[..16])
    }

    /// Clear all cached responses for a connector
    pub async fn clear_connector(&mut self, connector_type: &str) -> Result<()> {
        if !self.enabled || self.conn.is_none() {
            return Ok(());
        }

        if let Some(ref mut conn) = self.conn {
            let pattern = format!("connector:{}:*", connector_type);
            let mut cursor: u64 = 0;
            let mut deleted_count = 0;

            loop {
                let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                    .arg(cursor)
                    .arg("MATCH")
                    .arg(&pattern)
                    .arg("COUNT")
                    .arg(100)
                    .query_async(conn)
                    .await?;

                if !keys.is_empty() {
                    let _: () = conn.del(&keys).await?;
                    deleted_count += keys.len();
                }

                cursor = new_cursor;
                if cursor == 0 {
                    break;
                }
            }

            tracing::info!("🗑️  Cleared {} cached responses for {}", deleted_count, connector_type);
        }

        Ok(())
    }

    /// Invalidate a specific cache entry
    pub async fn invalidate(&mut self, connector_type: &str, endpoint: &str, params: &str) -> Result<()> {
        if !self.enabled || self.conn.is_none() {
            return Ok(());
        }

        let key = Self::generate_cache_key(connector_type, endpoint, params);
        
        if let Some(ref mut conn) = self.conn {
            let _: () = conn.del(&key).await?;
            tracing::debug!("🗑️  Invalidated cache key: {}", key);
        }

        Ok(())
    }

    /// Health check
    pub async fn health_check(&mut self) -> bool {
        if !self.enabled || self.conn.is_none() {
            return false;
        }

        if let Some(ref mut conn) = self.conn {
            redis::cmd("PING")
                .query_async::<_, String>(conn)
                .await
                .is_ok()
        } else {
            false
        }
    }

    /// Set custom TTL
    pub fn set_ttl(&mut self, ttl_seconds: u64) {
        self.ttl_seconds = ttl_seconds;
    }
}
