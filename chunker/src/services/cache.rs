use anyhow::{Result, Context};
use redis::{Client, AsyncCommands, aio::ConnectionManager};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::time::Duration;

use conhub_models::chunking::Chunk;

/// Redis-based caching service for chunking operations
pub struct ChunkCache {
    client: Client,
    conn: Option<ConnectionManager>,
    enabled: bool,
    ttl_seconds: u64,
}

impl ChunkCache {
    /// Create a new cache instance
    pub async fn new(redis_url: Option<String>) -> Self {
        let redis_url = redis_url.unwrap_or_else(|| {
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string())
        });

        match Client::open(redis_url.as_str()) {
            Ok(client) => {
                match ConnectionManager::new(client.clone()).await {
                    Ok(conn) => {
                        tracing::info!("✅ Redis cache connected");
                        Self {
                            client,
                            conn: Some(conn),
                            enabled: true,
                            ttl_seconds: 3600, // 1 hour default
                        }
                    }
                    Err(e) => {
                        tracing::warn!("⚠️  Redis connection failed: {}, cache disabled", e);
                        Self {
                            client,
                            conn: None,
                            enabled: false,
                            ttl_seconds: 3600,
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("⚠️  Redis client creation failed: {}, cache disabled", e);
                Self {
                    client: Client::open("redis://localhost").unwrap(),
                    conn: None,
                    enabled: false,
                    ttl_seconds: 3600,
                }
            }
        }
    }

    /// Get cached chunks for a given content hash
    pub async fn get_chunks(&mut self, content: &str, strategy: &str) -> Result<Option<Vec<Chunk>>> {
        if !self.enabled || self.conn.is_none() {
            return Ok(None);
        }

        let key = Self::generate_cache_key(content, strategy);
        
        if let Some(ref mut conn) = self.conn {
            match conn.get::<_, Option<String>>(&key).await {
                Ok(Some(cached_json)) => {
                    match serde_json::from_str::<Vec<Chunk>>(&cached_json) {
                        Ok(chunks) => {
                            tracing::debug!("🎯 Cache hit for key: {}", key);
                            Ok(Some(chunks))
                        }
                        Err(e) => {
                            tracing::warn!("⚠️  Failed to deserialize cached chunks: {}", e);
                            Ok(None)
                        }
                    }
                }
                Ok(None) => {
                    tracing::debug!("❌ Cache miss for key: {}", key);
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

    /// Cache chunks for a given content hash
    pub async fn set_chunks(&mut self, content: &str, strategy: &str, chunks: &[Chunk]) -> Result<()> {
        if !self.enabled || self.conn.is_none() {
            return Ok(());
        }

        let key = Self::generate_cache_key(content, strategy);
        let chunks_json = serde_json::to_string(chunks)
            .context("Failed to serialize chunks")?;

        if let Some(ref mut conn) = self.conn {
            match conn.set_ex::<_, _, ()>(&key, chunks_json, self.ttl_seconds as usize).await {
                Ok(_) => {
                    tracing::debug!("💾 Cached chunks for key: {}", key);
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

    /// Generate a stable cache key from content hash and strategy
    fn generate_cache_key(content: &str, strategy: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let content_hash = format!("{:x}", hasher.finalize());
        
        format!("chunk:{}:{}", strategy, &content_hash[..16])
    }

    /// Clear all chunking caches
    pub async fn clear_all(&mut self) -> Result<()> {
        if !self.enabled || self.conn.is_none() {
            return Ok(());
        }

        if let Some(ref mut conn) = self.conn {
            // Use SCAN to find all chunk: keys and delete them
            let pattern = "chunk:*";
            let mut cursor: u64 = 0;
            let mut deleted_count = 0;

            loop {
                let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                    .arg(cursor)
                    .arg("MATCH")
                    .arg(pattern)
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

            tracing::info!("🗑️  Cleared {} cached chunk entries", deleted_count);
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

    /// Get cache statistics
    pub async fn get_stats(&mut self) -> Result<CacheStats> {
        if !self.enabled || self.conn.is_none() {
            return Ok(CacheStats::default());
        }

        if let Some(ref mut conn) = self.conn {
            // Count chunk keys
            let pattern = "chunk:*";
            let mut cursor: u64 = 0;
            let mut total_keys = 0;

            loop {
                let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                    .arg(cursor)
                    .arg("MATCH")
                    .arg(pattern)
                    .arg("COUNT")
                    .arg(100)
                    .query_async(conn)
                    .await?;

                total_keys += keys.len();
                cursor = new_cursor;
                if cursor == 0 {
                    break;
                }
            }

            // Get memory info
            let info: String = redis::cmd("INFO")
                .arg("memory")
                .query_async(conn)
                .await?;

            let used_memory = info
                .lines()
                .find(|line| line.starts_with("used_memory:"))
                .and_then(|line| line.split(':').nth(1))
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);

            Ok(CacheStats {
                enabled: true,
                total_keys,
                used_memory_bytes: used_memory,
                ttl_seconds: self.ttl_seconds,
            })
        } else {
            Ok(CacheStats::default())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheStats {
    pub enabled: bool,
    pub total_keys: usize,
    pub used_memory_bytes: u64,
    pub ttl_seconds: u64,
}
