use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use futures::stream::{StreamExt, FuturesUnordered};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::sources::core::{DataSource, DataSourceConnector, DataSourceFactory};
use crate::sources::enhanced_connector::{EnhancedConnector, ConnectorError};

/// Advanced data service with connection pooling, caching, and performance optimizations
pub struct AdvancedDataService {
    /// Connection pool for data sources
    connection_pool: Arc<RwLock<HashMap<String, Arc<dyn DataSourceConnector>>>>,
    
    /// Cache for frequently accessed data
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    
    /// Metrics for performance monitoring
    metrics: Arc<RwLock<DataServiceMetrics>>,
    
    /// Semaphore for controlling concurrent operations
    operation_semaphore: Arc<Semaphore>,
    
    /// Configuration for the service
    config: DataServiceConfig,
    
    /// Active data sources
    data_sources: Arc<RwLock<HashMap<String, DataSource>>>,
    
    /// Connection health status
    health_status: Arc<RwLock<HashMap<String, ConnectionHealth>>>,
}

/// Configuration for the advanced data service
#[derive(Debug, Clone)]
pub struct DataServiceConfig {
    /// Maximum number of concurrent operations
    pub max_concurrent_operations: usize,
    
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    
    /// Maximum cache size (number of entries)
    pub max_cache_size: usize,
    
    /// Connection timeout in seconds
    pub connection_timeout_seconds: u64,
    
    /// Retry attempts for failed operations
    pub max_retry_attempts: u32,
    
    /// Health check interval in seconds
    pub health_check_interval_seconds: u64,
    
    /// Enable automatic cache cleanup
    pub enable_cache_cleanup: bool,
    
    /// Batch size for bulk operations
    pub batch_size: usize,
}

impl Default for DataServiceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_operations: 50,
            cache_ttl_seconds: 300, // 5 minutes
            max_cache_size: 10000,
            connection_timeout_seconds: 30,
            max_retry_attempts: 3,
            health_check_interval_seconds: 60,
            enable_cache_cleanup: true,
            batch_size: 100,
        }
    }
}

/// Cache entry with TTL and metadata
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub ttl: Duration,
    pub access_count: u64,
    pub last_accessed: DateTime<Utc>,
}

impl CacheEntry {
    pub fn new(data: serde_json::Value, ttl: Duration) -> Self {
        let now = Utc::now();
        Self {
            data,
            created_at: now,
            ttl,
            access_count: 0,
            last_accessed: now,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.created_at + chrono::Duration::from_std(self.ttl).unwrap_or_default()
    }
    
    pub fn access(&mut self) -> &serde_json::Value {
        self.access_count += 1;
        self.last_accessed = Utc::now();
        &self.data
    }
}

/// Connection health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionHealth {
    pub is_healthy: bool,
    pub last_check: DateTime<Utc>,
    pub response_time_ms: u64,
    pub error_count: u32,
    pub success_count: u32,
    pub last_error: Option<String>,
}

impl ConnectionHealth {
    pub fn new() -> Self {
        Self {
            is_healthy: true,
            last_check: Utc::now(),
            response_time_ms: 0,
            error_count: 0,
            success_count: 0,
            last_error: None,
        }
    }
    
    pub fn record_success(&mut self, response_time_ms: u64) {
        self.is_healthy = true;
        self.last_check = Utc::now();
        self.response_time_ms = response_time_ms;
        self.success_count += 1;
        self.last_error = None;
    }
    
    pub fn record_error(&mut self, error: String) {
        self.is_healthy = false;
        self.last_check = Utc::now();
        self.error_count += 1;
        self.last_error = Some(error);
    }
    
    pub fn health_score(&self) -> f64 {
        if self.success_count + self.error_count == 0 {
            return 1.0;
        }
        
        let success_rate = self.success_count as f64 / (self.success_count + self.error_count) as f64;
        let response_penalty = if self.response_time_ms > 5000 { 0.1 } else { 0.0 };
        
        (success_rate - response_penalty).max(0.0)
    }
}

/// Metrics for monitoring data service performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataServiceMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub average_response_time_ms: f64,
    pub active_connections: u32,
    pub total_data_transferred_bytes: u64,
    pub operations_per_source: HashMap<String, u64>,
    pub error_rates_per_source: HashMap<String, f64>,
    pub last_reset: DateTime<Utc>,
}

impl DataServiceMetrics {
    pub fn new() -> Self {
        Self {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            cache_hits: 0,
            cache_misses: 0,
            average_response_time_ms: 0.0,
            active_connections: 0,
            total_data_transferred_bytes: 0,
            operations_per_source: HashMap::new(),
            error_rates_per_source: HashMap::new(),
            last_reset: Utc::now(),
        }
    }
    
    pub fn record_operation(&mut self, source_id: &str, success: bool, response_time_ms: u64, data_size_bytes: u64) {
        self.total_operations += 1;
        
        if success {
            self.successful_operations += 1;
        } else {
            self.failed_operations += 1;
        }
        
        // Update average response time
        let total_time = self.average_response_time_ms * (self.total_operations - 1) as f64;
        self.average_response_time_ms = (total_time + response_time_ms as f64) / self.total_operations as f64;
        
        self.total_data_transferred_bytes += data_size_bytes;
        
        // Update per-source metrics
        *self.operations_per_source.entry(source_id.to_string()).or_insert(0) += 1;
        
        let source_ops = self.operations_per_source.get(source_id).unwrap_or(&0);
        let current_error_rate = self.error_rates_per_source.get(source_id).unwrap_or(&0.0);
        
        if !success {
            let new_error_rate = (current_error_rate * (*source_ops - 1) as f64 + 1.0) / *source_ops as f64;
            self.error_rates_per_source.insert(source_id.to_string(), new_error_rate);
        } else {
            let new_error_rate = (current_error_rate * (*source_ops - 1) as f64) / *source_ops as f64;
            self.error_rates_per_source.insert(source_id.to_string(), new_error_rate);
        }
    }
    
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }
    
    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }
    
    pub fn cache_hit_rate(&self) -> f64 {
        if self.cache_hits + self.cache_misses == 0 {
            return 0.0;
        }
        self.cache_hits as f64 / (self.cache_hits + self.cache_misses) as f64
    }
    
    pub fn success_rate(&self) -> f64 {
        if self.total_operations == 0 {
            return 1.0;
        }
        self.successful_operations as f64 / self.total_operations as f64
    }
}

impl AdvancedDataService {
    /// Create a new advanced data service
    pub fn new(config: DataServiceConfig) -> Self {
        Self {
            connection_pool: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(DataServiceMetrics::new())),
            operation_semaphore: Arc::new(Semaphore::new(config.max_concurrent_operations)),
            data_sources: Arc::new(RwLock::new(HashMap::new())),
            health_status: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(DataServiceConfig::default())
    }
    
    /// Add a data source to the service
    pub async fn add_data_source(&self, data_source: DataSource) -> Result<(), ConnectorError> {
        let connector = DataSourceFactory::create_connector(&data_source.source_type)
            .map_err(|e| ConnectorError::Other(e.to_string()))?;
        
        let source_id = data_source.id.clone();
        
        // Store the data source
        self.data_sources.write().await.insert(source_id.clone(), data_source);
        
        // Store the connector in the pool
        self.connection_pool.write().await.insert(source_id.clone(), Arc::from(connector));
        
        // Initialize health status
        self.health_status.write().await.insert(source_id, ConnectionHealth::new());
        
        Ok(())
    }
    
    /// Remove a data source from the service
    pub async fn remove_data_source(&self, source_id: &str) -> Result<(), ConnectorError> {
        self.data_sources.write().await.remove(source_id);
        self.connection_pool.write().await.remove(source_id);
        self.health_status.write().await.remove(source_id);
        
        // Clear related cache entries
        self.clear_cache_for_source(source_id).await?;
        
        Ok(())
    }
    
    /// Get data from cache or fetch from source
    pub async fn get_data(&self, source_id: &str, key: &str) -> Result<serde_json::Value, ConnectorError> {
        let cache_key = format!("{}:{}", source_id, key);
        
        // Try cache first
        if let Some(cached_data) = self.get_from_cache(&cache_key).await {
            self.metrics.write().await.record_cache_hit();
            return Ok(cached_data);
        }
        
        self.metrics.write().await.record_cache_miss();
        
        // Fetch from source
        let start_time = Instant::now();
        let _permit = self.operation_semaphore.acquire().await.unwrap();
        
        let result = self.fetch_from_source(source_id, key).await;
        let response_time = start_time.elapsed().as_millis() as u64;
        
        match &result {
            Ok(data) => {
                // Cache the result
                self.set_cache(&cache_key, data.clone()).await?;
                
                // Update metrics
                let data_size = serde_json::to_vec(data).unwrap_or_default().len() as u64;
                self.metrics.write().await.record_operation(source_id, true, response_time, data_size);
                
                // Update health status
                if let Some(health) = self.health_status.write().await.get_mut(source_id) {
                    health.record_success(response_time);
                }
            }
            Err(e) => {
                // Update metrics
                self.metrics.write().await.record_operation(source_id, false, response_time, 0);
                
                // Update health status
                if let Some(health) = self.health_status.write().await.get_mut(source_id) {
                    health.record_error(e.to_string());
                }
            }
        }
        
        result
    }
    
    /// Fetch data from multiple sources in parallel
    pub async fn get_data_batch(&self, requests: Vec<(String, String)>) -> Vec<Result<serde_json::Value, ConnectorError>> {
        let futures: FuturesUnordered<_> = requests
            .into_iter()
            .map(|(source_id, key)| {
                let source_id = source_id.clone();
                let key = key.clone();
                async move { self.get_data(&source_id, &key).await }
            })
            .collect();
        
        futures.collect().await
    }
    
    /// Get cached data
    async fn get_from_cache(&self, key: &str) -> Option<serde_json::Value> {
        let mut cache = self.cache.write().await;
        
        if let Some(entry) = cache.get_mut(key) {
            if !entry.is_expired() {
                return Some(entry.access().clone());
            } else {
                cache.remove(key);
            }
        }
        
        None
    }
    
    /// Set data in cache
    async fn set_cache(&self, key: &str, data: serde_json::Value) -> Result<(), ConnectorError> {
        let mut cache = self.cache.write().await;
        
        // Check cache size limit
        if cache.len() >= self.config.max_cache_size {
            self.evict_cache_entries(&mut cache).await;
        }
        
        let ttl = Duration::from_secs(self.config.cache_ttl_seconds);
        cache.insert(key.to_string(), CacheEntry::new(data, ttl));
        
        Ok(())
    }
    
    /// Evict old cache entries using LRU strategy
    async fn evict_cache_entries(&self, cache: &mut HashMap<String, CacheEntry>) {
        let evict_count = cache.len() / 4; // Evict 25% of entries
        
        let mut entries: Vec<_> = cache.iter().map(|(k, v)| (k.clone(), v.last_accessed)).collect();
        entries.sort_by(|a, b| a.1.cmp(&b.1));
        
        let keys_to_remove: Vec<String> = entries.iter().take(evict_count).map(|(k, _)| k.clone()).collect();
        
        for key in keys_to_remove {
            cache.remove(&key);
        }
    }
    
    /// Clear cache for a specific source
    async fn clear_cache_for_source(&self, source_id: &str) -> Result<(), ConnectorError> {
        let mut cache = self.cache.write().await;
        let prefix = format!("{}:", source_id);
        
        cache.retain(|key, _| !key.starts_with(&prefix));
        
        Ok(())
    }
    
    /// Fetch data from source (placeholder implementation)
    async fn fetch_from_source(&self, source_id: &str, key: &str) -> Result<serde_json::Value, ConnectorError> {
        // This would be implemented based on the specific data source
        // For now, return a placeholder
        Ok(serde_json::json!({
            "source_id": source_id,
            "key": key,
            "data": "placeholder_data",
            "timestamp": Utc::now()
        }))
    }
    
    /// Get service metrics
    pub async fn get_metrics(&self) -> DataServiceMetrics {
        self.metrics.read().await.clone()
    }
    
    /// Reset service metrics
    pub async fn reset_metrics(&self) {
        *self.metrics.write().await = DataServiceMetrics::new();
    }
    
    /// Get health status for all connections
    pub async fn get_health_status(&self) -> HashMap<String, ConnectionHealth> {
        self.health_status.read().await.clone()
    }
    
    /// Perform health check on all connections
    pub async fn health_check(&self) -> Result<HashMap<String, bool>, ConnectorError> {
        let data_sources = self.data_sources.read().await.clone();
        let mut results = HashMap::new();
        
        for (source_id, _) in data_sources {
            let is_healthy = self.check_source_health(&source_id).await.unwrap_or(false);
            results.insert(source_id, is_healthy);
        }
        
        Ok(results)
    }
    
    /// Check health of a specific source
    async fn check_source_health(&self, source_id: &str) -> Result<bool, ConnectorError> {
        let start_time = Instant::now();
        
        // Perform a lightweight operation to test connectivity
        let result = self.fetch_from_source(source_id, "health_check").await;
        let response_time = start_time.elapsed().as_millis() as u64;
        
        let is_healthy = result.is_ok();
        
        // Update health status
        if let Some(health) = self.health_status.write().await.get_mut(source_id) {
            if is_healthy {
                health.record_success(response_time);
            } else {
                health.record_error(result.err().unwrap().to_string());
            }
        }
        
        Ok(is_healthy)
    }
    
    /// Clean up expired cache entries
    pub async fn cleanup_cache(&self) -> Result<usize, ConnectorError> {
        let mut cache = self.cache.write().await;
        let initial_size = cache.len();
        
        cache.retain(|_, entry| !entry.is_expired());
        
        let cleaned_count = initial_size - cache.len();
        Ok(cleaned_count)
    }
    
    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let metrics = self.metrics.read().await;
        
        CacheStats {
            total_entries: cache.len(),
            hit_rate: metrics.cache_hit_rate(),
            total_hits: metrics.cache_hits,
            total_misses: metrics.cache_misses,
            memory_usage_estimate: cache.len() * 1024, // Rough estimate
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub hit_rate: f64,
    pub total_hits: u64,
    pub total_misses: u64,
    pub memory_usage_estimate: usize,
}
