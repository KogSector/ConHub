use anyhow::Result;
use redis::{Client, AsyncCommands, aio::ConnectionManager};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

use crate::models::{ContextQueryRequest, ContextQueryResponse};

/// Redis-based query result cache
pub struct QueryCache {
    client: Client,
    conn: Option<ConnectionManager>,
    enabled: bool,
    ttl_seconds: u64,
}

impl QueryCache {
    pub async fn new(redis_url: Option<String>) -> Self {
        let redis_url = redis_url.unwrap_or_else(|| {
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string())
        });

        match Client::open(redis_url.as_str()) {
            Ok(client) => {
                match ConnectionManager::new(client.clone()).await {
                    Ok(conn) => {
                        tracing::info!("✅ Query cache connected to Redis");
                        Self {
                            client,
                            conn: Some(conn),
                            enabled: true,
                            ttl_seconds: 300, // 5 minutes default
                        }
                    }
                    Err(e) => {
                        tracing::warn!("⚠️  Redis connection failed: {}, query cache disabled", e);
                        Self {
                            client,
                            conn: None,
                            enabled: false,
                            ttl_seconds: 300,
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("⚠️  Redis client creation failed: {}, query cache disabled", e);
                Self {
                    client: Client::open("redis://localhost").unwrap(),
                    conn: None,
                    enabled: false,
                    ttl_seconds: 300,
                }
            }
        }
    }

    pub async fn get(&mut self, request: &ContextQueryRequest) -> Result<Option<ContextQueryResponse>> {
        if !self.enabled || self.conn.is_none() {
            return Ok(None);
        }

        let key = Self::generate_cache_key(request);
        
        if let Some(ref mut conn) = self.conn {
            match conn.get::<_, Option<String>>(&key).await {
                Ok(Some(cached_json)) => {
                    match serde_json::from_str::<ContextQueryResponse>(&cached_json) {
                        Ok(response) => {
                            tracing::debug!("🎯 Query cache hit for key: {}", key);
                            Ok(Some(response))
                        }
                        Err(e) => {
                            tracing::warn!("⚠️  Failed to deserialize cached response: {}", e);
                            Ok(None)
                        }
                    }
                }
                Ok(None) => {
                    tracing::debug!("❌ Query cache miss for key: {}", key);
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

    pub async fn set(&mut self, request: &ContextQueryRequest, response: &ContextQueryResponse) -> Result<()> {
        if !self.enabled || self.conn.is_none() {
            return Ok(());
        }

        let key = Self::generate_cache_key(request);
        let response_json = serde_json::to_string(response)?;

        if let Some(ref mut conn) = self.conn {
            match conn.set_ex::<_, _, ()>(&key, response_json, self.ttl_seconds as usize).await {
                Ok(_) => {
                    tracing::debug!("💾 Cached query result for key: {}", key);
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

    fn generate_cache_key(request: &ContextQueryRequest) -> String {
        // Hash query + filters + strategy
        let mut hasher = Sha256::new();
        hasher.update(request.query.as_bytes());
        hasher.update(format!("{:?}", request.filters).as_bytes());
        hasher.update(format!("{:?}", request.strategy).as_bytes());
        hasher.update(request.top_k.to_string().as_bytes());
        
        let hash = format!("{:x}", hasher.finalize());
        format!("context_query:{}:{}", request.tenant_id, &hash[..16])
    }

    pub async fn clear_tenant(&mut self, tenant_id: uuid::Uuid) -> Result<()> {
        if !self.enabled || self.conn.is_none() {
            return Ok(());
        }

        if let Some(ref mut conn) = self.conn {
            let pattern = format!("context_query:{}:*", tenant_id);
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

            tracing::info!("🗑️  Cleared {} cached queries for tenant {}", deleted_count, tenant_id);
        }

        Ok(())
    }

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
}
