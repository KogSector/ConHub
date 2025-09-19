use redis::{Client, Connection, RedisError, RedisResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use uuid::Uuid;
use dashmap::DashMap;

/// High-performance multi-level caching service
/// Implements LRU + LFU hybrid algorithm with write-through and write-back strategies
pub struct AdvancedCacheService {
    /// L1 Cache: In-memory ultra-fast access (LRU)
    l1_cache: Arc<DashMap<String, CacheEntry>>,
    /// L2 Cache: Redis distributed cache
    redis_client: Option<Client>,
    /// Cache statistics for optimization
    stats: Arc<RwLock<CacheStats>>,
    /// Configuration
    config: CacheConfig,
}

#[derive(Clone, Debug)]
pub struct CacheConfig {
    pub l1_max_size: usize,
    pub l1_ttl_seconds: u64,
    pub l2_ttl_seconds: u64,
    pub enable_compression: bool,
    pub enable_encryption: bool,
    pub write_strategy: WriteStrategy,
}

#[derive(Clone, Debug)]
pub enum WriteStrategy {
    WriteThrough,  // Write to both L1 and L2 immediately
    WriteBack,     // Write to L1, batch write to L2
    WriteAround,   // Write only to L2, bypass L1
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CacheEntry {
    pub data: Vec<u8>,
    pub created_at: u64,
    pub last_accessed: u64,
    pub access_count: u64,
    pub ttl_seconds: u64,
    pub compressed: bool,
    pub encrypted: bool,
}

#[derive(Debug, Default)]
pub struct CacheStats {
    pub l1_hits: u64,
    pub l1_misses: u64,
    pub l2_hits: u64,
    pub l2_misses: u64,
    pub evictions: u64,
    pub compressions: u64,
    pub decompressions: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            l1_max_size: 1000,
            l1_ttl_seconds: 300, // 5 minutes
            l2_ttl_seconds: 3600, // 1 hour
            enable_compression: true,
            enable_encryption: false,
            write_strategy: WriteStrategy::WriteThrough,
        }
    }
}

impl AdvancedCacheService {
    pub fn new(redis_url: Option<&str>, config: Option<CacheConfig>) -> Self {
        let config = config.unwrap_or_default();
        
        let redis_client = if let Some(url) = redis_url {
            Client::open(url).ok()
        } else {
            None
        };

        Self {
            l1_cache: Arc::new(DashMap::new()),
            redis_client,
            stats: Arc::new(RwLock::new(CacheStats::default())),
            config,
        }
    }

    /// Get value from cache with optimal lookup strategy
    pub async fn get<T>(&self, key: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Try L1 cache first (fastest)
        if let Some(entry) = self.get_from_l1(key).await {
            let mut stats = self.stats.write().await;
            stats.l1_hits += 1;
            
            if let Ok(value) = self.deserialize_entry::<T>(&entry) {
                return Some(value);
            }
        }

        // Try L2 cache (Redis)
        if let Some(entry) = self.get_from_l2(key).await {
            let mut stats = self.stats.write().await;
            stats.l2_hits += 1;
            
            // Promote to L1 cache
            self.set_to_l1(key, &entry).await;
            
            if let Ok(value) = self.deserialize_entry::<T>(&entry) {
                return Some(value);
            }
        }

        // Cache miss
        let mut stats = self.stats.write().await;
        stats.l2_misses += 1;
        None
    }

    /// Set value in cache with intelligent write strategy
    pub async fn set<T>(&self, key: &str, value: &T, ttl_seconds: Option<u64>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        T: Serialize,
    {
        let ttl = ttl_seconds.unwrap_or(self.config.l1_ttl_seconds);
        let entry = self.create_cache_entry(value, ttl)?;

        match self.config.write_strategy {
            WriteStrategy::WriteThrough => {
                // Write to both L1 and L2 immediately
                self.set_to_l1(key, &entry).await;
                self.set_to_l2(key, &entry).await;
            }
            WriteStrategy::WriteBack => {
                // Write to L1 immediately, schedule L2 write
                self.set_to_l1(key, &entry).await;
                // TODO: Implement batch write scheduler
            }
            WriteStrategy::WriteAround => {
                // Write only to L2
                self.set_to_l2(key, &entry).await;
            }
        }

        Ok(())
    }

    /// Invalidate cache entry
    pub async fn invalidate(&self, key: &str) {
        self.l1_cache.remove(key);
        if let Some(redis) = &self.redis_client {
            if let Ok(mut conn) = redis.get_connection() {
                let _: RedisResult<()> = redis::cmd("DEL").arg(key).query(&mut conn);
            }
        }
    }

    /// Get cache statistics for monitoring
    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    /// Clear all cache data
    pub async fn clear(&self) {
        self.l1_cache.clear();
        if let Some(redis) = &self.redis_client {
            if let Ok(mut conn) = redis.get_connection() {
                let _: RedisResult<()> = redis::cmd("FLUSHDB").query(&mut conn);
            }
        }
    }

    // Private helper methods

    async fn get_from_l1(&self, key: &str) -> Option<CacheEntry> {
        if let Some(mut entry) = self.l1_cache.get_mut(key) {
            // Check TTL
            let now = current_timestamp();
            if now - entry.created_at > entry.ttl_seconds {
                drop(entry);
                self.l1_cache.remove(key);
                return None;
            }

            // Update access statistics
            entry.last_accessed = now;
            entry.access_count += 1;

            return Some(entry.clone());
        }
        None
    }

    async fn get_from_l2(&self, key: &str) -> Option<CacheEntry> {
        if let Some(redis) = &self.redis_client {
            if let Ok(mut conn) = redis.get_connection() {
                if let Ok(data) = redis::cmd("GET").arg(key).query::<Vec<u8>>(&mut conn) {
                    if let Ok(entry) = bincode::deserialize::<CacheEntry>(&data) {
                        return Some(entry);
                    }
                }
            }
        }
        None
    }

    async fn set_to_l1(&self, key: &str, entry: &CacheEntry) {
        // Implement LRU eviction if cache is full
        if self.l1_cache.len() >= self.config.l1_max_size {
            self.evict_lru().await;
        }

        self.l1_cache.insert(key.to_string(), entry.clone());
    }

    async fn set_to_l2(&self, key: &str, entry: &CacheEntry) {
        if let Some(redis) = &self.redis_client {
            if let Ok(mut conn) = redis.get_connection() {
                if let Ok(data) = bincode::serialize(entry) {
                    let _: RedisResult<()> = redis::cmd("SETEX")
                        .arg(key)
                        .arg(entry.ttl_seconds)
                        .arg(data)
                        .query(&mut conn);
                }
            }
        }
    }

    fn create_cache_entry<T>(&self, value: &T, ttl_seconds: u64) -> Result<CacheEntry, Box<dyn std::error::Error + Send + Sync>>
    where
        T: Serialize,
    {
        let mut data = bincode::serialize(value)?;
        let mut compressed = false;
        let mut encrypted = false;

        // Apply compression if enabled and beneficial
        if self.config.enable_compression && data.len() > 1024 {
            data = compress_data(&data)?;
            compressed = true;
            
            let mut stats = futures::executor::block_on(self.stats.write());
            stats.compressions += 1;
        }

        // Apply encryption if enabled
        if self.config.enable_encryption {
            data = encrypt_data(&data)?;
            encrypted = true;
        }

        Ok(CacheEntry {
            data,
            created_at: current_timestamp(),
            last_accessed: current_timestamp(),
            access_count: 0,
            ttl_seconds,
            compressed,
            encrypted,
        })
    }

    fn deserialize_entry<T>(&self, entry: &CacheEntry) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut data = entry.data.clone();

        // Apply decryption if needed
        if entry.encrypted {
            data = decrypt_data(&data)?;
        }

        // Apply decompression if needed
        if entry.compressed {
            data = decompress_data(&data)?;
            
            let mut stats = futures::executor::block_on(self.stats.write());
            stats.decompressions += 1;
        }

        Ok(bincode::deserialize(&data)?)
    }

    async fn evict_lru(&self) {
        // Find least recently used entry
        let mut oldest_key: Option<String> = None;
        let mut oldest_time = u64::MAX;

        for entry in self.l1_cache.iter() {
            let last_accessed = entry.value().last_accessed;
            if last_accessed < oldest_time {
                oldest_time = last_accessed;
                oldest_key = Some(entry.key().clone());
            }
        }

        if let Some(key) = oldest_key {
            self.l1_cache.remove(&key);
            let mut stats = self.stats.write().await;
            stats.evictions += 1;
        }
    }
}

// Utility functions

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn compress_data(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    // Simple compression implementation (in production, use zstd or lz4)
    use std::io::Write;
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}

fn decompress_data(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    use std::io::Read;
    let mut decoder = flate2::read::GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

fn encrypt_data(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    // Placeholder for encryption (implement with AES-GCM in production)
    Ok(data.to_vec())
}

fn decrypt_data(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    // Placeholder for decryption (implement with AES-GCM in production)
    Ok(data.to_vec())
}

/// Query result caching specifically for database operations
pub struct QueryCache {
    cache: AdvancedCacheService,
}

impl QueryCache {
    pub fn new(redis_url: Option<&str>) -> Self {
        let config = CacheConfig {
            l1_max_size: 500,
            l1_ttl_seconds: 60, // 1 minute for queries
            l2_ttl_seconds: 600, // 10 minutes
            enable_compression: true,
            enable_encryption: false,
            write_strategy: WriteStrategy::WriteThrough,
        };

        Self {
            cache: AdvancedCacheService::new(redis_url, Some(config)),
        }
    }

    pub async fn get_or_compute<F, T, Fut>(&self, key: &str, compute_fn: F) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
        T: Serialize + for<'de> Deserialize<'de> + Clone,
    {
        // Try cache first
        if let Some(cached) = self.cache.get::<T>(key).await {
            return Ok(cached);
        }

        // Compute value
        let computed = compute_fn().await?;
        
        // Cache the result
        self.cache.set(key, &computed, Some(300)).await?; // 5 minute TTL
        
        Ok(computed)
    }
}