use redis::{Client, Connection, RedisError, RedisResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use uuid::Uuid;
use dashmap::DashMap;



pub struct AdvancedCacheService {
    
    l1_cache: Arc<DashMap<String, CacheEntry>>,
    
    redis_client: Option<Client>,
    
    stats: Arc<RwLock<CacheStats>>,
    
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
    WriteThrough,  
    WriteBack,     
    WriteAround,   
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
            l1_ttl_seconds: 300, 
            l2_ttl_seconds: 3600, 
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

    
    pub async fn get<T>(&self, key: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        
        if let Some(entry) = self.get_from_l1(key).await {
            let mut stats = self.stats.write().await;
            stats.l1_hits += 1;
            
            if let Ok(value) = self.deserialize_entry::<T>(&entry) {
                return Some(value);
            }
        }

        
        if let Some(entry) = self.get_from_l2(key).await {
            let mut stats = self.stats.write().await;
            stats.l2_hits += 1;
            
            
            self.set_to_l1(key, &entry).await;
            
            if let Ok(value) = self.deserialize_entry::<T>(&entry) {
                return Some(value);
            }
        }

        
        let mut stats = self.stats.write().await;
        stats.l2_misses += 1;
        None
    }

    
    pub async fn set<T>(&self, key: &str, value: &T, ttl_seconds: Option<u64>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        T: Serialize,
    {
        let ttl = ttl_seconds.unwrap_or(self.config.l1_ttl_seconds);
        let entry = self.create_cache_entry(value, ttl)?;

        match self.config.write_strategy {
            WriteStrategy::WriteThrough => {
                
                self.set_to_l1(key, &entry).await;
                self.set_to_l2(key, &entry).await;
            }
            WriteStrategy::WriteBack => {
                
                self.set_to_l1(key, &entry).await;
                
            }
            WriteStrategy::WriteAround => {
                
                self.set_to_l2(key, &entry).await;
            }
        }

        Ok(())
    }

    
    pub async fn invalidate(&self, key: &str) {
        self.l1_cache.remove(key);
        if let Some(redis) = &self.redis_client {
            if let Ok(mut conn) = redis.get_connection() {
                let _: RedisResult<()> = redis::cmd("DEL").arg(key).query(&mut conn);
            }
        }
    }

    
    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    
    pub async fn clear(&self) {
        self.l1_cache.clear();
        if let Some(redis) = &self.redis_client {
            if let Ok(mut conn) = redis.get_connection() {
                let _: RedisResult<()> = redis::cmd("FLUSHDB").query(&mut conn);
            }
        }
    }

    

    async fn get_from_l1(&self, key: &str) -> Option<CacheEntry> {
        if let Some(mut entry) = self.l1_cache.get_mut(key) {
            
            let now = current_timestamp();
            if now - entry.created_at > entry.ttl_seconds {
                drop(entry);
                self.l1_cache.remove(key);
                return None;
            }

            
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

        
        if self.config.enable_compression && data.len() > 1024 {
            data = compress_data(&data)?;
            compressed = true;
            
            let mut stats = futures::executor::block_on(self.stats.write());
            stats.compressions += 1;
        }

        
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

        
        if entry.encrypted {
            data = decrypt_data(&data)?;
        }

        
        if entry.compressed {
            data = decompress_data(&data)?;
            
            let mut stats = futures::executor::block_on(self.stats.write());
            stats.decompressions += 1;
        }

        Ok(bincode::deserialize(&data)?)
    }

    async fn evict_lru(&self) {
        
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



fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn compress_data(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    
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
    
    Ok(data.to_vec())
}

fn decrypt_data(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    
    Ok(data.to_vec())
}


pub struct QueryCache {
    cache: AdvancedCacheService,
}

impl QueryCache {
    pub fn new(redis_url: Option<&str>) -> Self {
        let config = CacheConfig {
            l1_max_size: 500,
            l1_ttl_seconds: 60, 
            l2_ttl_seconds: 600, 
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
        
        if let Some(cached) = self.cache.get::<T>(key).await {
            return Ok(cached);
        }

        
        let computed = compute_fn().await?;
        
        
        self.cache.set(key, &computed, Some(300)).await?; 
        
        Ok(computed)
    }
}