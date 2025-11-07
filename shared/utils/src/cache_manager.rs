use std::sync::Arc;
use std::time::{Duration, Instant};
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Serialize, de::DeserializeOwned};

/// Multi-level cache manager with TTL and size limits
pub struct CacheManager {
    // L1: In-memory cache
    l1_cache: Arc<DashMap<String, CacheEntry<Vec<u8>>>>,
    // Configuration
    config: CacheConfig,
    // Statistics
    stats: Arc<RwLock<CacheStats>>,
}

#[derive(Clone)]
pub struct CacheConfig {
    pub max_entries: usize,
    pub default_ttl: Duration,
    pub max_entry_size: usize,
    pub enable_compression: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10_000,
            default_ttl: Duration::from_secs(300), // 5 minutes
            max_entry_size: 1024 * 1024, // 1 MB
            enable_compression: true,
        }
    }
}

struct CacheEntry<T> {
    data: T,
    created_at: Instant,
    ttl: Duration,
    hits: Arc<RwLock<u64>>,
    size: usize,
}

#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub total_entries: usize,
    pub total_size: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

impl CacheManager {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            l1_cache: Arc::new(DashMap::new()),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Get value from cache
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let entry = self.l1_cache.get(key)?;
        
        // Check TTL
        let age = entry.created_at.elapsed();
        if age > entry.ttl {
            drop(entry);
            self.l1_cache.remove(key);
            self.stats.write().misses += 1;
            return None;
        }

        // Update hit count
        *entry.hits.write() += 1;
        self.stats.write().hits += 1;

        // Deserialize
        match serde_json::from_slice(&entry.data) {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    }

    /// Set value in cache
    pub fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<(), CacheError> {
        // Serialize
        let data = serde_json::to_vec(value)
            .map_err(|_| CacheError::SerializationError)?;
        
        let size = data.len();
        
        // Check size limit
        if size > self.config.max_entry_size {
            return Err(CacheError::EntryTooLarge);
        }

        // Check capacity and evict if needed
        if self.l1_cache.len() >= self.config.max_entries {
            self.evict_lru()?;
        }

        let entry = CacheEntry {
            data,
            created_at: Instant::now(),
            ttl: ttl.unwrap_or(self.config.default_ttl),
            hits: Arc::new(RwLock::new(0)),
            size,
        };

        self.l1_cache.insert(key.to_string(), entry);
        
        let mut stats = self.stats.write();
        stats.total_entries = self.l1_cache.len();
        stats.total_size += size;

        Ok(())
    }

    /// Remove value from cache
    pub fn remove(&self, key: &str) -> bool {
        if let Some((_, entry)) = self.l1_cache.remove(key) {
            let mut stats = self.stats.write();
            stats.total_size = stats.total_size.saturating_sub(entry.size);
            stats.total_entries = self.l1_cache.len();
            true
        } else {
            false
        }
    }

    /// Clear all cache entries
    pub fn clear(&self) {
        self.l1_cache.clear();
        let mut stats = self.stats.write();
        stats.total_entries = 0;
        stats.total_size = 0;
    }

    /// Evict least recently used entry
    fn evict_lru(&self) -> Result<(), CacheError> {
        let mut lru_key: Option<String> = None;
        let mut lru_hits: u64 = u64::MAX;
        let mut lru_age: Duration = Duration::ZERO;

        // Find LRU entry (least hits, oldest)
        for entry in self.l1_cache.iter() {
            let hits = *entry.value().hits.read();
            let age = entry.value().created_at.elapsed();
            
            if hits < lru_hits || (hits == lru_hits && age > lru_age) {
                lru_key = Some(entry.key().clone());
                lru_hits = hits;
                lru_age = age;
            }
        }

        if let Some(key) = lru_key {
            self.l1_cache.remove(&key);
            self.stats.write().evictions += 1;
            Ok(())
        } else {
            Err(CacheError::EvictionFailed)
        }
    }

    /// Clean up expired entries
    pub fn cleanup_expired(&self) {
        let now = Instant::now();
        let mut to_remove = Vec::new();

        for entry in self.l1_cache.iter() {
            let age = now.duration_since(entry.value().created_at);
            if age > entry.value().ttl {
                to_remove.push(entry.key().clone());
            }
        }

        for key in to_remove {
            self.remove(&key);
        }
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        self.stats.read().clone()
    }

    /// Get or compute value
    pub fn get_or_compute<T, F>(&self, key: &str, compute: F) -> Result<T, CacheError>
    where
        T: Serialize + DeserializeOwned + Clone,
        F: FnOnce() -> Result<T, CacheError>,
    {
        // Try cache first
        if let Some(value) = self.get::<T>(key) {
            return Ok(value);
        }

        // Compute value
        let value = compute()?;
        
        // Store in cache
        self.set(key, &value, None)?;
        
        Ok(value)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Serialization error")]
    SerializationError,
    #[error("Entry too large")]
    EntryTooLarge,
    #[error("Eviction failed")]
    EvictionFailed,
    #[error("Computation error: {0}")]
    ComputationError(String),
}

/// Global cache manager instance
lazy_static::lazy_static! {
    pub static ref GLOBAL_CACHE: CacheManager = {
        CacheManager::new(CacheConfig::default())
    };
}

/// Helper function to get the global cache manager
pub fn get_cache() -> &'static CacheManager {
    &GLOBAL_CACHE
}

/// Embedding cache with specialized configuration
pub fn get_embedding_cache() -> CacheManager {
    CacheManager::new(CacheConfig {
        max_entries: 5_000,
        default_ttl: Duration::from_secs(3600), // 1 hour
        max_entry_size: 5 * 1024 * 1024, // 5 MB
        enable_compression: true,
    })
}

/// Search results cache with TTL
pub fn get_search_cache() -> CacheManager {
    CacheManager::new(CacheConfig {
        max_entries: 10_000,
        default_ttl: Duration::from_secs(600), // 10 minutes
        max_entry_size: 2 * 1024 * 1024, // 2 MB
        enable_compression: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic_operations() {
        let cache = CacheManager::new(CacheConfig::default());
        
        // Set and get
        cache.set("key1", &"value1", None).unwrap();
        assert_eq!(cache.get::<String>("key1"), Some("value1".to_string()));
        
        // Miss
        assert_eq!(cache.get::<String>("nonexistent"), None);
        
        // Remove
        cache.remove("key1");
        assert_eq!(cache.get::<String>("key1"), None);
    }

    #[test]
    fn test_cache_ttl() {
        let cache = CacheManager::new(CacheConfig::default());
        
        cache.set("key1", &"value1", Some(Duration::from_millis(100))).unwrap();
        assert_eq!(cache.get::<String>("key1"), Some("value1".to_string()));
        
        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(cache.get::<String>("key1"), None);
    }

    #[test]
    fn test_cache_stats() {
        let cache = CacheManager::new(CacheConfig::default());
        
        cache.set("key1", &"value1", None).unwrap();
        cache.get::<String>("key1");
        cache.get::<String>("key1");
        cache.get::<String>("nonexistent");
        
        let stats = cache.get_stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate(), 2.0 / 3.0);
    }
}
