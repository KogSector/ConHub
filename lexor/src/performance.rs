use std::sync::Arc;

use tokio::sync::RwLock;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub max_memory_mb: usize,
    pub ttl_seconds: u64,
    pub max_entries: usize,
}

#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub data: T,
    pub created_at: std::time::Instant,
    pub access_count: u32,
    pub size_bytes: usize,
}

pub struct IntelligentCache<T: Clone> {
    entries: Arc<DashMap<String, CacheEntry<T>>>,
    config: CacheConfig,
    total_size: Arc<RwLock<usize>>,
}

impl<T: Clone> IntelligentCache<T> {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            entries: Arc::new(DashMap::new()),
            config,
            total_size: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn get(&self, key: &str) -> Option<T> {
        if let Some(mut entry) = self.entries.get_mut(key) {
            // Check TTL
            if entry.created_at.elapsed().as_secs() > self.config.ttl_seconds {
                drop(entry);
                self.entries.remove(key);
                return None;
            }
            
            entry.access_count += 1;
            Some(entry.data.clone())
        } else {
            None
        }
    }

    pub async fn put(&self, key: String, data: T, size_bytes: usize) {
        // Check if we need to evict
        while self.should_evict(size_bytes).await {
            self.evict_lru().await;
        }

        let entry = CacheEntry {
            data,
            created_at: std::time::Instant::now(),
            access_count: 1,
            size_bytes,
        };

        self.entries.insert(key, entry);
        
        let mut total_size = self.total_size.write().await;
        *total_size += size_bytes;
    }

    async fn should_evict(&self, new_size: usize) -> bool {
        let total_size = *self.total_size.read().await;
        let max_size = self.config.max_memory_mb * 1024 * 1024;
        
        total_size + new_size > max_size || self.entries.len() >= self.config.max_entries
    }

    async fn evict_lru(&self) {
        let mut oldest_key = None;
        let _oldest_time = std::time::Instant::now();
        let mut lowest_score = f64::MAX;

        for entry in self.entries.iter() {
            let age_seconds = entry.created_at.elapsed().as_secs_f64();
            let access_frequency = entry.access_count as f64;
            
            // LFU + LRU hybrid score (lower is worse)
            let score = access_frequency / (1.0 + age_seconds);
            
            if score < lowest_score {
                lowest_score = score;
                oldest_key = Some(entry.key().clone());
            }
        }

        if let Some(key) = oldest_key {
            if let Some((_, entry)) = self.entries.remove(&key) {
                let mut total_size = self.total_size.write().await;
                *total_size = total_size.saturating_sub(entry.size_bytes);
            }
        }
    }
}

pub struct PerformanceOptimizer {
    search_cache: IntelligentCache<crate::types::SearchResult>,
    #[allow(dead_code)]
    symbol_cache: IntelligentCache<Vec<crate::types::Symbol>>,
    metrics: Arc<RwLock<PerformanceMetrics>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub avg_search_time_ms: f64,
    pub total_searches: u64,
    pub memory_usage_mb: f64,
}

impl PerformanceOptimizer {
    pub fn new() -> Self {
        let cache_config = CacheConfig {
            max_memory_mb: 512,
            ttl_seconds: 3600, // 1 hour
            max_entries: 10000,
        };

        Self {
            search_cache: IntelligentCache::new(cache_config.clone()),
            symbol_cache: IntelligentCache::new(cache_config),
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
        }
    }

    pub async fn get_cached_search(&self, query_hash: &str) -> Option<crate::types::SearchResult> {
        let result = self.search_cache.get(query_hash).await;
        
        let mut metrics = self.metrics.write().await;
        if result.is_some() {
            metrics.cache_hits += 1;
        } else {
            metrics.cache_misses += 1;
        }
        
        result
    }

    pub async fn cache_search_result(&self, query_hash: String, result: crate::types::SearchResult) {
        let size_estimate = std::mem::size_of_val(&result) + 
            result.results.len() * std::mem::size_of::<crate::types::SearchHit>();
        
        self.search_cache.put(query_hash, result, size_estimate).await;
    }

    pub async fn record_search_time(&self, duration_ms: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.total_searches += 1;
        
        // Exponential moving average
        let alpha = 0.1;
        metrics.avg_search_time_ms = alpha * duration_ms + (1.0 - alpha) * metrics.avg_search_time_ms;
    }

    pub async fn get_metrics(&self) -> PerformanceMetrics {
        (*self.metrics.read().await).clone()
    }
}

#[allow(dead_code)]
pub fn create_query_hash(query: &crate::types::SearchQuery) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    query.query.hash(&mut hasher);
    query.query_type.hash(&mut hasher);
    query.case_sensitive.hash(&mut hasher);
    query.regex.hash(&mut hasher);
    
    format!("{:x}", hasher.finish())
}