use std::collections::{HashMap, BTreeMap};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::hash::{Hash, Hasher};
use tokio::sync::{RwLock, Mutex};
use tokio::time::{interval, sleep};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rand::seq::SliceRandom;
use rand::thread_rng;
use flate2::Compression;
use flate2::write::{GzEncoder, GzDecoder};
use std::io::Write;

/// Advanced multi-layer cache with intelligent eviction policies
pub struct AdvancedCache {
    /// L1 Cache - In-memory, fastest access
    l1_cache: Arc<RwLock<L1Cache>>,
    
    /// L2 Cache - Compressed in-memory, larger capacity
    l2_cache: Arc<RwLock<L2Cache>>,
    
    /// Cache configuration
    config: CacheConfig,
    
    /// Cache metrics
    metrics: Arc<RwLock<CacheMetrics>>,
    
    /// Eviction policy manager
    eviction_manager: Arc<EvictionManager>,
    
    /// Cache warming manager
    warming_manager: Arc<CacheWarmingManager>,
    
    /// Bloom filter for negative caching
    bloom_filter: Arc<RwLock<BloomFilter>>,
}

/// Configuration for the advanced cache
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// L1 cache maximum size (number of entries)
    pub l1_max_size: usize,
    
    /// L2 cache maximum size (number of entries)
    pub l2_max_size: usize,
    
    /// Default TTL for cache entries
    pub default_ttl_seconds: u64,
    
    /// Maximum TTL for any cache entry
    pub max_ttl_seconds: u64,
    
    /// Eviction policy for L1 cache
    pub l1_eviction_policy: EvictionPolicy,
    
    /// Eviction policy for L2 cache
    pub l2_eviction_policy: EvictionPolicy,
    
    /// Enable compression for L2 cache
    pub enable_l2_compression: bool,
    
    /// Compression threshold (bytes)
    pub compression_threshold_bytes: usize,
    
    /// Enable cache warming
    pub enable_cache_warming: bool,
    
    /// Cache warming batch size
    pub warming_batch_size: usize,
    
    /// Enable negative caching
    pub enable_negative_caching: bool,
    
    /// Negative cache TTL
    pub negative_cache_ttl_seconds: u64,
    
    /// Enable cache statistics
    pub enable_statistics: bool,
    
    /// Statistics collection interval
    pub statistics_interval_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            l1_max_size: 10000,
            l2_max_size: 100000,
            default_ttl_seconds: 300, // 5 minutes
            max_ttl_seconds: 3600,    // 1 hour
            l1_eviction_policy: EvictionPolicy::LRU,
            l2_eviction_policy: EvictionPolicy::LFU,
            enable_l2_compression: true,
            compression_threshold_bytes: 1024, // 1KB
            enable_cache_warming: true,
            warming_batch_size: 100,
            enable_negative_caching: true,
            negative_cache_ttl_seconds: 60, // 1 minute
            enable_statistics: true,
            statistics_interval_seconds: 60,
        }
    }
}

/// Eviction policies for cache management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionPolicy {
    /// Least Recently Used
    LRU,
    /// Least Frequently Used
    LFU,
    /// Time-based expiration
    TTL,
    /// Adaptive Replacement Cache
    ARC,
    /// Random eviction
    Random,
    /// Size-based eviction
    SizeBased,
}

/// L1 Cache - Fast in-memory cache
pub struct L1Cache {
    /// Cache entries
    entries: HashMap<String, L1CacheEntry>,
    
    /// LRU tracking
    lru_tracker: LRUTracker,
    
    /// LFU tracking
    lfu_tracker: LFUTracker,
    
    /// Configuration
    config: CacheConfig,
}

/// L1 Cache entry
#[derive(Debug, Clone)]
pub struct L1CacheEntry {
    /// Cached data
    pub data: serde_json::Value,
    
    /// Entry metadata
    pub metadata: CacheEntryMetadata,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last accessed timestamp
    pub last_accessed: DateTime<Utc>,
    
    /// Access count
    pub access_count: u64,
    
    /// Entry size in bytes
    pub size_bytes: usize,
    
    /// TTL for this entry
    pub ttl: Duration,
}

/// L2 Cache - Compressed in-memory cache
pub struct L2Cache {
    /// Cache entries (potentially compressed)
    entries: HashMap<String, L2CacheEntry>,
    
    /// LRU tracking
    lru_tracker: LRUTracker,
    
    /// LFU tracking
    lfu_tracker: LFUTracker,
    
    /// Configuration
    config: CacheConfig,
}

/// L2 Cache entry
#[derive(Debug, Clone)]
pub struct L2CacheEntry {
    /// Cached data (potentially compressed)
    pub data: Vec<u8>,
    
    /// Whether the data is compressed
    pub is_compressed: bool,
    
    /// Entry metadata
    pub metadata: CacheEntryMetadata,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last accessed timestamp
    pub last_accessed: DateTime<Utc>,
    
    /// Access count
    pub access_count: u64,
    
    /// Original size in bytes
    pub original_size_bytes: usize,
    
    /// Compressed size in bytes
    pub compressed_size_bytes: usize,
    
    /// TTL for this entry
    pub ttl: Duration,
}

/// Metadata for cache entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntryMetadata {
    /// Entry key
    pub key: String,
    
    /// Data source identifier
    pub source_id: String,
    
    /// Entry type/category
    pub entry_type: String,
    
    /// Priority level
    pub priority: CachePriority,
    
    /// Tags for categorization
    pub tags: Vec<String>,
    
    /// Custom metadata
    pub custom: HashMap<String, serde_json::Value>,
}

/// Cache priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum CachePriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// LRU (Least Recently Used) tracker
#[derive(Debug)]
pub struct LRUTracker {
    /// Access order tracking
    access_order: BTreeMap<DateTime<Utc>, String>,
    
    /// Key to timestamp mapping
    key_timestamps: HashMap<String, DateTime<Utc>>,
}

impl LRUTracker {
    pub fn new() -> Self {
        Self {
            access_order: BTreeMap::new(),
            key_timestamps: HashMap::new(),
        }
    }
    
    pub fn access(&mut self, key: &str) {
        // Remove old entry if exists
        if let Some(old_timestamp) = self.key_timestamps.remove(key) {
            self.access_order.remove(&old_timestamp);
        }
        
        // Add new entry
        let timestamp = Utc::now();
        self.access_order.insert(timestamp, key.to_string());
        self.key_timestamps.insert(key.to_string(), timestamp);
    }
    
    pub fn remove(&mut self, key: &str) {
        if let Some(timestamp) = self.key_timestamps.remove(key) {
            self.access_order.remove(&timestamp);
        }
    }
    
    pub fn get_lru_keys(&self, count: usize) -> Vec<String> {
        self.access_order
            .iter()
            .take(count)
            .map(|(_, key)| key.clone())
            .collect()
    }
}

/// LFU (Least Frequently Used) tracker
#[derive(Debug)]
pub struct LFUTracker {
    /// Frequency counts
    frequencies: HashMap<String, u64>,
    
    /// Frequency to keys mapping
    frequency_buckets: BTreeMap<u64, Vec<String>>,
}

impl LFUTracker {
    pub fn new() -> Self {
        Self {
            frequencies: HashMap::new(),
            frequency_buckets: BTreeMap::new(),
        }
    }
    
    pub fn access(&mut self, key: &str) {
        let old_freq = self.frequencies.get(key).copied().unwrap_or(0);
        let new_freq = old_freq + 1;
        
        // Remove from old frequency bucket
        if old_freq > 0 {
            if let Some(bucket) = self.frequency_buckets.get_mut(&old_freq) {
                bucket.retain(|k| k != key);
                if bucket.is_empty() {
                    self.frequency_buckets.remove(&old_freq);
                }
            }
        }
        
        // Add to new frequency bucket
        self.frequency_buckets
            .entry(new_freq)
            .or_insert_with(Vec::new)
            .push(key.to_string());
        
        self.frequencies.insert(key.to_string(), new_freq);
    }
    
    pub fn remove(&mut self, key: &str) {
        if let Some(freq) = self.frequencies.remove(key) {
            if let Some(bucket) = self.frequency_buckets.get_mut(&freq) {
                bucket.retain(|k| k != key);
                if bucket.is_empty() {
                    self.frequency_buckets.remove(&freq);
                }
            }
        }
    }
    
    pub fn get_lfu_keys(&self, count: usize) -> Vec<String> {
        let mut result = Vec::new();
        
        for (_, keys) in self.frequency_buckets.iter() {
            for key in keys {
                if result.len() >= count {
                    break;
                }
                result.push(key.clone());
            }
            if result.len() >= count {
                break;
            }
        }
        
        result
    }
}

/// Eviction manager for handling cache eviction policies
pub struct EvictionManager {
    /// Configuration
    config: CacheConfig,
}

impl EvictionManager {
    pub fn new(config: CacheConfig) -> Self {
        Self { config }
    }
    
    /// Get keys to evict from L1 cache
    pub fn get_l1_eviction_candidates(
        &self,
        cache: &L1Cache,
        count: usize,
    ) -> Vec<String> {
        match self.config.l1_eviction_policy {
            EvictionPolicy::LRU => cache.lru_tracker.get_lru_keys(count),
            EvictionPolicy::LFU => cache.lfu_tracker.get_lfu_keys(count),
            EvictionPolicy::TTL => self.get_expired_keys_l1(cache),
            EvictionPolicy::Random => self.get_random_keys_l1(cache, count),
            EvictionPolicy::SizeBased => self.get_largest_keys_l1(cache, count),
            EvictionPolicy::ARC => self.get_arc_eviction_candidates_l1(cache, count),
        }
    }
    
    /// Get keys to evict from L2 cache
    pub fn get_l2_eviction_candidates(
        &self,
        cache: &L2Cache,
        count: usize,
    ) -> Vec<String> {
        match self.config.l2_eviction_policy {
            EvictionPolicy::LRU => cache.lru_tracker.get_lru_keys(count),
            EvictionPolicy::LFU => cache.lfu_tracker.get_lfu_keys(count),
            EvictionPolicy::TTL => self.get_expired_keys_l2(cache),
            EvictionPolicy::Random => self.get_random_keys_l2(cache, count),
            EvictionPolicy::SizeBased => self.get_largest_keys_l2(cache, count),
            EvictionPolicy::ARC => self.get_arc_eviction_candidates_l2(cache, count),
        }
    }
    
    fn get_expired_keys_l1(&self, cache: &L1Cache) -> Vec<String> {
        let now = Utc::now();
        cache
            .entries
            .iter()
            .filter(|(_, entry)| {
                now > entry.created_at + chrono::Duration::from_std(entry.ttl).unwrap_or_default()
            })
            .map(|(key, _)| key.clone())
            .collect()
    }
    
    fn get_expired_keys_l2(&self, cache: &L2Cache) -> Vec<String> {
        let now = Utc::now();
        cache
            .entries
            .iter()
            .filter(|(_, entry)| {
                now > entry.created_at + chrono::Duration::from_std(entry.ttl).unwrap_or_default()
            })
            .map(|(key, _)| key.clone())
            .collect()
    }
    
    fn get_random_keys_l1(&self, cache: &L1Cache, count: usize) -> Vec<String> {
        let mut keys: Vec<_> = cache.entries.keys().cloned().collect();
        keys.shuffle(&mut thread_rng());
        keys.into_iter().take(count).collect()
    }
    
    fn get_random_keys_l2(&self, cache: &L2Cache, count: usize) -> Vec<String> {
        let mut keys: Vec<_> = cache.entries.keys().cloned().collect();
        keys.shuffle(&mut thread_rng());
        keys.into_iter().take(count).collect()
    }
    
    fn get_largest_keys_l1(&self, cache: &L1Cache, count: usize) -> Vec<String> {
        let mut entries: Vec<_> = cache.entries.iter().collect();
        entries.sort_by(|a, b| b.1.size_bytes.cmp(&a.1.size_bytes));
        entries.into_iter().take(count).map(|(key, _)| key.clone()).collect()
    }
    
    fn get_largest_keys_l2(&self, cache: &L2Cache, count: usize) -> Vec<String> {
        let mut entries: Vec<_> = cache.entries.iter().collect();
        entries.sort_by(|a, b| b.1.compressed_size_bytes.cmp(&a.1.compressed_size_bytes));
        entries.into_iter().take(count).map(|(key, _)| key.clone()).collect()
    }
    
    fn get_arc_eviction_candidates_l1(&self, cache: &L1Cache, count: usize) -> Vec<String> {
        // Simplified ARC implementation - combine LRU and LFU
        let lru_candidates = cache.lru_tracker.get_lru_keys(count / 2);
        let lfu_candidates = cache.lfu_tracker.get_lfu_keys(count / 2);
        
        let mut candidates = lru_candidates;
        candidates.extend(lfu_candidates);
        candidates.into_iter().take(count).collect()
    }
    
    fn get_arc_eviction_candidates_l2(&self, cache: &L2Cache, count: usize) -> Vec<String> {
        // Simplified ARC implementation - combine LRU and LFU
        let lru_candidates = cache.lru_tracker.get_lru_keys(count / 2);
        let lfu_candidates = cache.lfu_tracker.get_lfu_keys(count / 2);
        
        let mut candidates = lru_candidates;
        candidates.extend(lfu_candidates);
        candidates.into_iter().take(count).collect()
    }
}

/// Cache warming manager for preloading frequently accessed data
pub struct CacheWarmingManager {
    /// Warming strategies
    strategies: Vec<WarmingStrategy>,
    
    /// Configuration
    config: CacheConfig,
}

/// Cache warming strategies
#[derive(Debug, Clone)]
pub enum WarmingStrategy {
    /// Warm based on access patterns
    AccessPattern,
    /// Warm based on time patterns
    TimePattern,
    /// Warm based on data relationships
    DataRelationship,
    /// Warm based on user behavior
    UserBehavior,
}

impl CacheWarmingManager {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            strategies: vec![
                WarmingStrategy::AccessPattern,
                WarmingStrategy::TimePattern,
            ],
            config,
        }
    }
    
    /// Get keys that should be warmed
    pub async fn get_warming_candidates(&self, _metrics: &CacheMetrics) -> Vec<String> {
        // This would be implemented based on historical access patterns
        // For now, return empty list
        Vec::new()
    }
}

/// Bloom filter for negative caching
pub struct BloomFilter {
    /// Bit array
    bits: Vec<bool>,
    
    /// Number of hash functions
    hash_functions: usize,
    
    /// Size of the bit array
    size: usize,
}

impl BloomFilter {
    pub fn new(expected_items: usize, false_positive_rate: f64) -> Self {
        let size = Self::optimal_size(expected_items, false_positive_rate);
        let hash_functions = Self::optimal_hash_functions(size, expected_items);
        
        Self {
            bits: vec![false; size],
            hash_functions,
            size,
        }
    }
    
    fn optimal_size(expected_items: usize, false_positive_rate: f64) -> usize {
        let ln2 = std::f64::consts::LN_2;
        (-(expected_items as f64 * false_positive_rate.ln()) / (ln2 * ln2)).ceil() as usize
    }
    
    fn optimal_hash_functions(size: usize, expected_items: usize) -> usize {
        ((size as f64 / expected_items as f64) * std::f64::consts::LN_2).ceil() as usize
    }
    
    pub fn add(&mut self, key: &str) {
        for i in 0..self.hash_functions {
            let hash = self.hash(key, i);
            self.bits[hash % self.size] = true;
        }
    }
    
    pub fn contains(&self, key: &str) -> bool {
        for i in 0..self.hash_functions {
            let hash = self.hash(key, i);
            if !self.bits[hash % self.size] {
                return false;
            }
        }
        true
    }
    
    fn hash(&self, key: &str, seed: usize) -> usize {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        seed.hash(&mut hasher);
        hasher.finish() as usize
    }
}

/// Cache metrics for monitoring and optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    /// L1 cache metrics
    pub l1_hits: u64,
    pub l1_misses: u64,
    pub l1_evictions: u64,
    pub l1_size: usize,
    pub l1_memory_usage_bytes: usize,
    
    /// L2 cache metrics
    pub l2_hits: u64,
    pub l2_misses: u64,
    pub l2_evictions: u64,
    pub l2_size: usize,
    pub l2_memory_usage_bytes: usize,
    pub l2_compression_ratio: f64,
    
    /// Overall metrics
    pub total_requests: u64,
    pub hit_rate: f64,
    pub average_access_time_ms: f64,
    pub cache_warming_operations: u64,
    pub negative_cache_hits: u64,
    
    /// Performance metrics
    pub eviction_time_ms: f64,
    pub compression_time_ms: f64,
    pub decompression_time_ms: f64,
    
    /// Last reset timestamp
    pub last_reset: DateTime<Utc>,
}

impl CacheMetrics {
    pub fn new() -> Self {
        Self {
            l1_hits: 0,
            l1_misses: 0,
            l1_evictions: 0,
            l1_size: 0,
            l1_memory_usage_bytes: 0,
            l2_hits: 0,
            l2_misses: 0,
            l2_evictions: 0,
            l2_size: 0,
            l2_memory_usage_bytes: 0,
            l2_compression_ratio: 0.0,
            total_requests: 0,
            hit_rate: 0.0,
            average_access_time_ms: 0.0,
            cache_warming_operations: 0,
            negative_cache_hits: 0,
            eviction_time_ms: 0.0,
            compression_time_ms: 0.0,
            decompression_time_ms: 0.0,
            last_reset: Utc::now(),
        }
    }
    
    pub fn record_l1_hit(&mut self) {
        self.l1_hits += 1;
        self.total_requests += 1;
        self.update_hit_rate();
    }
    
    pub fn record_l1_miss(&mut self) {
        self.l1_misses += 1;
        self.total_requests += 1;
        self.update_hit_rate();
    }
    
    pub fn record_l2_hit(&mut self) {
        self.l2_hits += 1;
        self.total_requests += 1;
        self.update_hit_rate();
    }
    
    pub fn record_l2_miss(&mut self) {
        self.l2_misses += 1;
        self.total_requests += 1;
        self.update_hit_rate();
    }
    
    fn update_hit_rate(&mut self) {
        if self.total_requests > 0 {
            let total_hits = self.l1_hits + self.l2_hits;
            self.hit_rate = total_hits as f64 / self.total_requests as f64;
        }
    }
}

impl AdvancedCache {
    /// Create a new advanced cache
    pub fn new(config: CacheConfig) -> Self {
        let eviction_manager = Arc::new(EvictionManager::new(config.clone()));
        let warming_manager = Arc::new(CacheWarmingManager::new(config.clone()));
        let bloom_filter = Arc::new(RwLock::new(BloomFilter::new(10000, 0.01)));
        
        Self {
            l1_cache: Arc::new(RwLock::new(L1Cache {
                entries: HashMap::new(),
                lru_tracker: LRUTracker::new(),
                lfu_tracker: LFUTracker::new(),
                config: config.clone(),
            })),
            l2_cache: Arc::new(RwLock::new(L2Cache {
                entries: HashMap::new(),
                lru_tracker: LRUTracker::new(),
                lfu_tracker: LFUTracker::new(),
                config: config.clone(),
            })),
            config,
            metrics: Arc::new(RwLock::new(CacheMetrics::new())),
            eviction_manager,
            warming_manager,
            bloom_filter,
        }
    }
    
    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(CacheConfig::default())
    }
    
    /// Get value from cache
    pub async fn get<T>(&self, key: &str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        let start_time = Instant::now();
        
        // Check negative cache first
        if self.config.enable_negative_caching {
            let bloom = self.bloom_filter.read().await;
            if !bloom.contains(key) {
                self.metrics.write().await.negative_cache_hits += 1;
                return None;
            }
        }
        
        // Try L1 cache first
        {
            let mut l1 = self.l1_cache.write().await;
            if let Some(entry) = l1.entries.get(key) {
                if !self.is_expired(&entry.created_at, &entry.ttl) {
                    // Clone the data before updating trackers
                    let data = entry.data.clone();
                    
                    // Now update the entry and trackers
                    if let Some(entry) = l1.entries.get_mut(key) {
                        entry.last_accessed = Utc::now();
                        entry.access_count += 1;
                    }
                    l1.lru_tracker.access(key);
                    l1.lfu_tracker.access(key);
                    
                    self.metrics.write().await.record_l1_hit();
                    
                    if let Ok(value) = serde_json::from_value(data) {
                        return Some(value);
                    }
                }
            }
        }
        
        self.metrics.write().await.record_l1_miss();
        
        // Try L2 cache
        {
            let mut l2 = self.l2_cache.write().await;
            if let Some(entry) = l2.entries.get(key) {
                if !self.is_expired(&entry.created_at, &entry.ttl) {
                    // Clone the data before updating trackers
                    let data = if entry.is_compressed {
                        self.decompress_data(&entry.data).unwrap_or_default()
                    } else {
                        entry.data.clone()
                    };
                    
                    // Now update the entry and trackers
                    if let Some(entry) = l2.entries.get_mut(key) {
                        entry.last_accessed = Utc::now();
                        entry.access_count += 1;
                    }
                    l2.lru_tracker.access(key);
                    l2.lfu_tracker.access(key);
                    
                    self.metrics.write().await.record_l2_hit();
                    
                    if let Ok(json_value) = serde_json::from_slice::<serde_json::Value>(&data) {
                        if let Ok(value) = serde_json::from_value(json_value.clone()) {
                            // Promote to L1 cache
                            self.promote_to_l1(key, json_value).await;
                            return Some(value);
                        }
                    }
                }
            }
        }
        
        self.metrics.write().await.record_l2_miss();
        
        // Record access time
        let access_time = start_time.elapsed().as_millis() as f64;
        let mut metrics = self.metrics.write().await;
        let total_requests = metrics.total_requests as f64;
        metrics.average_access_time_ms = 
            (metrics.average_access_time_ms * (total_requests - 1.0) + access_time) / total_requests;
        
        None
    }
    
    /// Set value in cache
    pub async fn set<T>(&self, key: &str, value: &T, metadata: CacheEntryMetadata) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        T: Serialize,
    {
        let json_value = serde_json::to_value(value)?;
        let ttl = Duration::from_secs(self.config.default_ttl_seconds);
        
        // Try to store in L1 cache first
        let mut l1 = self.l1_cache.write().await;
        
        // Check if L1 cache needs eviction
        if l1.entries.len() >= self.config.l1_max_size {
            let eviction_candidates = self.eviction_manager.get_l1_eviction_candidates(&l1, 1);
            for candidate_key in eviction_candidates {
                if let Some(evicted_entry) = l1.entries.remove(&candidate_key) {
                    l1.lru_tracker.remove(&candidate_key);
                    l1.lfu_tracker.remove(&candidate_key);
                    
                    // Move to L2 cache if it's valuable
                    if evicted_entry.access_count > 1 {
                        self.store_in_l2(&candidate_key, evicted_entry.data, evicted_entry.metadata).await?;
                    }
                }
            }
        }
        
        // Store in L1 cache
        let size_bytes = serde_json::to_vec(&json_value)?.len();
        let entry = L1CacheEntry {
            data: json_value,
            metadata,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 0,
            size_bytes,
            ttl,
        };
        
        l1.entries.insert(key.to_string(), entry);
        l1.lru_tracker.access(key);
        l1.lfu_tracker.access(key);
        
        Ok(())
    }
    
    /// Promote entry from L2 to L1 cache
    async fn promote_to_l1(&self, key: &str, value: serde_json::Value) {
        let mut l1 = self.l1_cache.write().await;
        
        // Check if L1 cache needs eviction
        if l1.entries.len() >= self.config.l1_max_size {
            let eviction_candidates = self.eviction_manager.get_l1_eviction_candidates(&l1, 1);
            for candidate_key in eviction_candidates {
                if let Some(_) = l1.entries.remove(&candidate_key) {
                    l1.lru_tracker.remove(&candidate_key);
                    l1.lfu_tracker.remove(&candidate_key);
                }
            }
        }
        
        // Create L1 entry
        let size_bytes = serde_json::to_vec(&value).unwrap_or_default().len();
        let ttl = Duration::from_secs(self.config.default_ttl_seconds);
        
        let entry = L1CacheEntry {
            data: value,
            metadata: CacheEntryMetadata {
                key: key.to_string(),
                source_id: "promoted".to_string(),
                entry_type: "promoted".to_string(),
                priority: CachePriority::Normal,
                tags: vec![],
                custom: HashMap::new(),
            },
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 1,
            size_bytes,
            ttl,
        };
        
        l1.entries.insert(key.to_string(), entry);
        l1.lru_tracker.access(key);
        l1.lfu_tracker.access(key);
    }
    
    /// Store entry in L2 cache
    async fn store_in_l2(
        &self,
        key: &str,
        value: serde_json::Value,
        metadata: CacheEntryMetadata,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut l2 = self.l2_cache.write().await;
        
        // Check if L2 cache needs eviction
        if l2.entries.len() >= self.config.l2_max_size {
            let eviction_candidates = self.eviction_manager.get_l2_eviction_candidates(&l2, 1);
            for candidate_key in eviction_candidates {
                if let Some(_) = l2.entries.remove(&candidate_key) {
                    l2.lru_tracker.remove(&candidate_key);
                    l2.lfu_tracker.remove(&candidate_key);
                }
            }
        }
        
        // Serialize and optionally compress
        let data = serde_json::to_vec(&value)?;
        let original_size = data.len();
        
        let (final_data, is_compressed, compressed_size) = if self.config.enable_l2_compression 
            && original_size > self.config.compression_threshold_bytes {
            let compressed = self.compress_data(&data)?;
            let compressed_size = compressed.len();
            (compressed, true, compressed_size)
        } else {
            (data, false, original_size)
        };
        
        let ttl = Duration::from_secs(self.config.default_ttl_seconds);
        let entry = L2CacheEntry {
            data: final_data,
            is_compressed,
            metadata,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 0,
            original_size_bytes: original_size,
            compressed_size_bytes: compressed_size,
            ttl,
        };
        
        l2.entries.insert(key.to_string(), entry);
        l2.lru_tracker.access(key);
        l2.lfu_tracker.access(key);
        
        Ok(())
    }
    
    /// Check if entry is expired
    fn is_expired(&self, created_at: &DateTime<Utc>, ttl: &Duration) -> bool {
        let elapsed = Utc::now() - *created_at;
        elapsed > chrono::Duration::from_std(*ttl).unwrap_or_default()
    }
    
    /// Compress data
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        use std::io::Write;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }
    
    /// Decompress data
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        use flate2::read::GzDecoder;
        use std::io::Read;
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }
    
    /// Get cache metrics
    pub async fn get_metrics(&self) -> CacheMetrics {
        self.metrics.read().await.clone()
    }
    
    /// Reset cache metrics
    pub async fn reset_metrics(&self) {
        *self.metrics.write().await = CacheMetrics::new();
    }
    
    /// Clear all cache entries
    pub async fn clear(&self) {
        self.l1_cache.write().await.entries.clear();
        self.l2_cache.write().await.entries.clear();
        *self.bloom_filter.write().await = BloomFilter::new(10000, 0.01);
    }
}
