use sqlx::{PgPool, Row, Postgres, Transaction, Acquire};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{RwLock, Semaphore};
use dashmap::DashMap;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use log::{info, warn, error};
use std::time::{Duration, Instant};
use futures::future::try_join_all;

/// Advanced database optimization service with connection pooling, query batching, and intelligent caching
#[derive(Debug, Clone)]
pub struct DatabaseOptimizations {
    pool: PgPool,
    /// Query result cache with TTL
    query_cache: Arc<DashMap<String, CacheEntry>>,
    /// Performance statistics
    stats: Arc<RwLock<DatabaseStats>>,
    /// Prepared statements cache
    prepared_statements: Arc<DashMap<String, String>>,
    /// Query batch scheduler
    batch_scheduler: Arc<RwLock<QueryBatchScheduler>>,
    /// Connection semaphore for rate limiting
    connection_semaphore: Arc<Semaphore>,
    /// Query performance analyzer
    query_analyzer: Arc<RwLock<QueryAnalyzer>>,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    data: serde_json::Value,
    created_at: Instant,
    ttl: Duration,
    access_count: u64,
    last_accessed: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_queries: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub avg_query_time_ms: f64,
    pub active_connections: u32,
    pub batch_operations: u64,
    pub connection_pool_utilization: f64,
    pub slow_queries: u64,
    pub deadlocks: u64,
}

#[derive(Debug)]
struct QueryBatchScheduler {
    batches: HashMap<String, QueryBatch>,
    max_batch_size: usize,
    batch_timeout: Duration,
}

#[derive(Debug)]
struct QueryBatch {
    queries: Vec<BatchedQuery>,
    created_at: Instant,
    batch_key: String,
}

#[derive(Debug)]
struct BatchedQuery {
    sql: String,
    params: Vec<serde_json::Value>,
    callback: tokio::sync::oneshot::Sender<Result<Vec<sqlx::postgres::PgRow>, sqlx::Error>>,
}

#[derive(Debug)]
struct QueryAnalyzer {
    query_patterns: HashMap<String, QueryPattern>,
    slow_query_threshold: Duration,
}

#[derive(Debug, Clone)]
struct QueryPattern {
    pattern: String,
    execution_count: u64,
    total_duration: Duration,
    avg_duration: Duration,
    max_duration: Duration,
    min_duration: Duration,
    last_execution: Instant,
}

impl DatabaseStats {
    pub fn new() -> Self {
        Self {
            total_queries: 0,
            cache_hits: 0,
            cache_misses: 0,
            avg_query_time_ms: 0.0,
            active_connections: 0,
            batch_operations: 0,
            connection_pool_utilization: 0.0,
            slow_queries: 0,
            deadlocks: 0,
        }
    }
    
    pub fn cache_hit_rate(&self) -> f64 {
        if self.total_queries == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_queries as f64
        }
    }
}

impl QueryBatchScheduler {
    fn new() -> Self {
        Self {
            batches: HashMap::new(),
            max_batch_size: 100,
            batch_timeout: Duration::from_millis(50),
        }
    }
    
    fn add_query(&mut self, batch_key: String, query: BatchedQuery) -> bool {
        let batch = self.batches.entry(batch_key.clone()).or_insert_with(|| QueryBatch {
            queries: Vec::new(),
            created_at: Instant::now(),
            batch_key: batch_key.clone(),
        });
        
        batch.queries.push(query);
        
        // Check if batch is ready for execution
        batch.queries.len() >= self.max_batch_size || 
        batch.created_at.elapsed() >= self.batch_timeout
    }
}

impl QueryAnalyzer {
    fn new() -> Self {
        Self {
            query_patterns: HashMap::new(),
            slow_query_threshold: Duration::from_millis(1000),
        }
    }
    
    fn analyze_query(&mut self, sql: &str, duration: Duration) {
        let pattern = self.extract_pattern(sql);
        let entry = self.query_patterns.entry(pattern.clone()).or_insert_with(|| QueryPattern {
            pattern: pattern.clone(),
            execution_count: 0,
            total_duration: Duration::ZERO,
            avg_duration: Duration::ZERO,
            max_duration: Duration::ZERO,
            min_duration: Duration::MAX,
            last_execution: Instant::now(),
        });
        
        entry.execution_count += 1;
        entry.total_duration += duration;
        entry.avg_duration = entry.total_duration / entry.execution_count as u32;
        entry.max_duration = entry.max_duration.max(duration);
        entry.min_duration = entry.min_duration.min(duration);
        entry.last_execution = Instant::now();
    }
    
    fn extract_pattern(&self, sql: &str) -> String {
        // Normalize SQL to pattern (remove specific values)
        sql.trim()
            .replace(char::is_numeric, "?")
            .replace("'", "")
            .replace("\"", "")
            .to_lowercase()
    }
}

impl DatabaseOptimizations {
    pub fn new(pool: PgPool) -> Self {
        let max_connections = 50; // Configure based on your needs
        
        Self {
            pool,
            query_cache: Arc::new(DashMap::new()),
            stats: Arc::new(RwLock::new(DatabaseStats::new())),
            prepared_statements: Arc::new(DashMap::new()),
            batch_scheduler: Arc::new(RwLock::new(QueryBatchScheduler::new())),
            connection_semaphore: Arc::new(Semaphore::new(max_connections)),
            query_analyzer: Arc::new(RwLock::new(QueryAnalyzer::new())),
        }
    }

    /// Execute a query with intelligent caching and performance monitoring
    pub async fn execute_optimized_query<T>(&self, sql: &str, cache_key: Option<&str>, ttl: Duration) -> Result<T, sqlx::Error>
    where
        T: serde::de::DeserializeOwned + Serialize + Clone,
    {
        let start_time = Instant::now();
        
        // Check cache first for read operations
        if let Some(key) = cache_key {
            if let Some(entry) = self.query_cache.get_mut(key) {
                if entry.created_at.elapsed() < entry.ttl {
                    // Cache hit - update access statistics
                    entry.access_count += 1;
                    entry.last_accessed = Instant::now();
                    
                    let mut stats = self.stats.write().await;
                    stats.total_queries += 1;
                    stats.cache_hits += 1;
                    
                    return serde_json::from_value(entry.data.clone())
                        .map_err(|e| sqlx::Error::Decode(Box::new(e)));
                }
                // Expired cache entry - remove it
                drop(entry);
                self.query_cache.remove(key);
            }
        }

        // Acquire connection with rate limiting
        let _permit = self.connection_semaphore.acquire().await
            .map_err(|_| sqlx::Error::PoolClosed)?;

        // Execute the query
        let rows = sqlx::query(sql).fetch_all(&self.pool).await?;
        
        let duration = start_time.elapsed();
        
        // Analyze query performance
        let mut analyzer = self.query_analyzer.write().await;
        analyzer.analyze_query(sql, duration);
        
        // Convert rows to JSON for caching
        let json_result: serde_json::Value = serde_json::to_value(&rows)
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        // Cache the result if cache_key is provided
        if let Some(key) = cache_key {
            let cache_entry = CacheEntry {
                data: json_result.clone(),
                created_at: Instant::now(),
                ttl,
                access_count: 0,
                last_accessed: Instant::now(),
            };
            self.query_cache.insert(key.to_string(), cache_entry);
        }

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_queries += 1;
        if cache_key.is_some() {
            stats.cache_misses += 1;
        }
        
        // Update average query time
        let new_avg = (stats.avg_query_time_ms * (stats.total_queries - 1) as f64 + duration.as_millis() as f64) / stats.total_queries as f64;
        stats.avg_query_time_ms = new_avg;
        
        if duration.as_millis() > 1000 {
            stats.slow_queries += 1;
        }

        // Deserialize and return result
        serde_json::from_value(json_result)
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))
    }

    /// Execute multiple queries in a transaction with optimized batching
    pub async fn execute_batch_transaction<F, R>(&self, queries: F) -> Result<R, sqlx::Error>
    where
        F: FnOnce(&mut Transaction<'_, Postgres>) -> futures::future::BoxFuture<'_, Result<R, sqlx::Error>> + Send,
        R: Send,
    {
        let _permit = self.connection_semaphore.acquire().await
            .map_err(|_| sqlx::Error::PoolClosed)?;

        let mut tx = self.pool.begin().await?;
        let result = queries(&mut tx).await;
        
        match result {
            Ok(value) => {
                tx.commit().await?;
                
                let mut stats = self.stats.write().await;
                stats.batch_operations += 1;
                
                Ok(value)
            }
            Err(e) => {
                tx.rollback().await?;
                Err(e)
            }
        }
    }

    /// Bulk insert with optimal batch sizing
    pub async fn bulk_insert<T>(&self, table: &str, records: Vec<T>, batch_size: usize) -> Result<u64, sqlx::Error>
    where
        T: Serialize,
    {
        if records.is_empty() {
            return Ok(0);
        }

        let mut total_inserted = 0u64;
        
        for chunk in records.chunks(batch_size) {
            let _permit = self.connection_semaphore.acquire().await
                .map_err(|_| sqlx::Error::PoolClosed)?;

            // Convert to JSON for bulk insert
            let json_data: Vec<serde_json::Value> = chunk.iter()
                .map(|record| serde_json::to_value(record))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

            // Use PostgreSQL's JSON bulk insert capabilities
            let insert_sql = format!(
                "INSERT INTO {} SELECT * FROM json_populate_recordset(NULL::{}, $1)",
                table, table
            );

            let result = sqlx::query(&insert_sql)
                .bind(serde_json::Value::Array(json_data))
                .execute(&self.pool)
                .await?;

            total_inserted += result.rows_affected();
        }

        let mut stats = self.stats.write().await;
        stats.batch_operations += 1;

        Ok(total_inserted)
    }

    /// Get database performance statistics
    pub async fn get_performance_stats(&self) -> DatabaseStats {
        let stats = self.stats.read().await;
        let mut result = stats.clone();
        
        // Calculate current connection pool utilization
        result.connection_pool_utilization = (50 - self.connection_semaphore.available_permits()) as f64 / 50.0;
        
        result
    }

    /// Get slow query analysis
    pub async fn get_slow_queries(&self) -> Vec<QueryPattern> {
        let analyzer = self.query_analyzer.read().await;
        analyzer.query_patterns.values()
            .filter(|pattern| pattern.avg_duration.as_millis() > 1000)
            .cloned()
            .collect()
    }

    /// Clear cache entries that are expired or least recently used
    pub async fn cleanup_cache(&self) {
        let now = Instant::now();
        let mut expired_keys = Vec::new();
        
        for entry in self.query_cache.iter() {
            if entry.value().created_at.elapsed() >= entry.value().ttl {
                expired_keys.push(entry.key().clone());
            }
        }
        
        for key in expired_keys {
            self.query_cache.remove(&key);
        }
        
        // If cache is still too large, remove least recently used entries
        if self.query_cache.len() > 1000 {
            let mut entries: Vec<_> = self.query_cache.iter()
                .map(|entry| (entry.key().clone(), entry.value().last_accessed))
                .collect();
                
            entries.sort_by_key(|(_, last_accessed)| *last_accessed);
            
            let to_remove = entries.len() - 800; // Keep 800 entries
            for (key, _) in entries.iter().take(to_remove) {
                self.query_cache.remove(key);
            }
        }
    }

    /// Create optimized database indexes for common query patterns
    pub async fn optimize_indexes(&self) -> Result<(), sqlx::Error> {
        let indexes = vec![
            // User table optimizations
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_email_active ON users(email) WHERE is_active = true",
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_organization_role ON users(organization, role) WHERE is_active = true",
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_created_at_desc ON users(created_at DESC)",
            
            // Session table optimizations
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_user_sessions_user_id_active ON user_sessions(user_id) WHERE is_active = true",
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_user_sessions_expires_at ON user_sessions(expires_at) WHERE expires_at > NOW()",
            
            // Repository data optimizations
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_repositories_user_id_updated ON repositories(user_id, updated_at DESC)",
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_repositories_status_priority ON repositories(status, priority DESC)",
            
            // Search optimization with GIN indexes
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_repositories_search ON repositories USING gin(to_tsvector('english', name || ' ' || description))",
            
            // Performance monitoring
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_query_logs_created_at ON query_logs(created_at DESC)",
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_query_logs_duration ON query_logs(duration_ms DESC)",
        ];

        for index_sql in indexes {
            match sqlx::query(index_sql).execute(&self.pool).await {
                Ok(_) => info!("Successfully created index: {}", index_sql),
                Err(e) => warn!("Failed to create index: {} - Error: {}", index_sql, e),
            }
        }

        Ok(())
    }

    /// Analyze and suggest query optimizations
    pub async fn analyze_query_performance(&self, sql: &str) -> Result<QueryAnalysisResult, sqlx::Error> {
        let explain_sql = format!("EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON) {}", sql);
        
        let rows = sqlx::query(&explain_sql).fetch_all(&self.pool).await?;
        
        // Parse the EXPLAIN output
        let analysis = if let Some(row) = rows.first() {
            let json_result: serde_json::Value = row.try_get(0)?;
            self.parse_explain_output(json_result)
        } else {
            QueryAnalysisResult::default()
        };

        Ok(analysis)
    }

    fn parse_explain_output(&self, explain_json: serde_json::Value) -> QueryAnalysisResult {
        // Simplified analysis - in production, you'd want more sophisticated parsing
        QueryAnalysisResult {
            estimated_cost: 0.0,
            actual_time: 0.0,
            rows_returned: 0,
            suggestions: vec![
                "Consider adding appropriate indexes".to_string(),
                "Review WHERE clause selectivity".to_string(),
            ],
        }
    }
}

#[derive(Debug, Default)]
pub struct QueryAnalysisResult {
    pub estimated_cost: f64,
    pub actual_time: f64,
    pub rows_returned: u64,
    pub suggestions: Vec<String>,
}

/// Background task for database maintenance
pub async fn database_maintenance_task(db_opt: Arc<DatabaseOptimizations>) {
    let mut interval = tokio::time::interval(Duration::from_secs(1800)); // Every 30 minutes
    
    loop {
        interval.tick().await;
        
        // Clean up cache
        db_opt.cleanup_cache().await;
        
        // Log statistics
        let stats = db_opt.get_performance_stats().await;
        info!("Database stats - Queries: {}, Cache hit rate: {:.2}%, Avg query time: {:.2}ms, Pool utilization: {:.2}%", 
              stats.total_queries, 
              stats.cache_hit_rate() * 100.0, 
              stats.avg_query_time_ms,
              stats.connection_pool_utilization * 100.0);
        
        // Analyze slow queries
        let slow_queries = db_opt.get_slow_queries().await;
        if !slow_queries.is_empty() {
            warn!("Found {} slow query patterns", slow_queries.len());
            for pattern in slow_queries.iter().take(5) { // Log top 5
                warn!("Slow query pattern: {} (avg: {}ms, count: {})", 
                     pattern.query_hash, 
                     pattern.avg_duration.as_millis(),
                     pattern.execution_count);
            }
        }
    }
}