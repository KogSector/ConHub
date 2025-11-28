use std::sync::Arc;
use dashmap::DashMap;
use parking_lot::RwLock;
use sqlx::{PgPool, postgres::{PgPoolOptions, PgConnectOptions}};
use std::str::FromStr;
use redis::Client as RedisClient;
use std::time::{Duration, Instant};

/// Optimized connection pool manager with intelligent resource management
pub struct ConnectionPoolManager {
    pg_pools: Arc<DashMap<String, PoolEntry<PgPool>>>,
    redis_clients: Arc<DashMap<String, RedisClient>>,
    config: PoolConfig,
}

#[derive(Clone)]
pub struct PoolConfig {
    pub max_connections: u32,
    pub min_idle: u32,
    pub max_lifetime: Duration,
    pub idle_timeout: Duration,
    pub acquire_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 20,
            min_idle: 5,
            max_lifetime: Duration::from_secs(3600),
            idle_timeout: Duration::from_secs(600),
            acquire_timeout: Duration::from_secs(30),
        }
    }
}

struct PoolEntry<T> {
    pool: T,
    created_at: Instant,
    last_used: Arc<RwLock<Instant>>,
}

impl ConnectionPoolManager {
    pub fn new(config: PoolConfig) -> Self {
        Self {
            pg_pools: Arc::new(DashMap::new()),
            redis_clients: Arc::new(DashMap::new()),
            config,
        }
    }

    /// Get or create PostgreSQL connection pool with intelligent caching
    pub async fn get_pg_pool(&self, database_url: &str) -> Result<PgPool, sqlx::Error> {
        // Check cache first
        if let Some(entry) = self.pg_pools.get(database_url) {
            entry.last_used.write().clone_from(&Instant::now());
            return Ok(entry.pool.clone());
        }

        // Create new pool
        // Disable server-side prepared statements to be compatible with pgbouncer/transaction pooling
        let connect_options = PgConnectOptions::from_str(database_url)?
            .statement_cache_capacity(0);

        let pool = PgPoolOptions::new()
            .max_connections(self.config.max_connections)
            .min_connections(self.config.min_idle)
            .max_lifetime(self.config.max_lifetime)
            .idle_timeout(self.config.idle_timeout)
            .acquire_timeout(self.config.acquire_timeout)
            .test_before_acquire(true)
            .connect_with(connect_options)
            .await?;

        let entry = PoolEntry {
            pool: pool.clone(),
            created_at: Instant::now(),
            last_used: Arc::new(RwLock::new(Instant::now())),
        };

        self.pg_pools.insert(database_url.to_string(), entry);
        Ok(pool)
    }

    /// Get or create Redis client with caching
    pub fn get_redis_client(&self, redis_url: &str) -> Result<RedisClient, redis::RedisError> {
        // Check cache first
        if let Some(client) = self.redis_clients.get(redis_url) {
            return Ok(client.clone());
        }

        // Create new client
        let client = RedisClient::open(redis_url)?;
        self.redis_clients.insert(redis_url.to_string(), client.clone());
        Ok(client)
    }

    /// Cleanup stale connections (call periodically)
    pub fn cleanup_stale(&self) {
        let now = Instant::now();
        
        self.pg_pools.retain(|_, entry| {
            let last_used = *entry.last_used.read();
            let age = now.duration_since(last_used);
            age < self.config.idle_timeout
        });
    }

    /// Get pool statistics for monitoring
    pub fn get_stats(&self) -> PoolStats {
        PoolStats {
            pg_pools: self.pg_pools.len(),
            redis_clients: self.redis_clients.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub pg_pools: usize,
    pub redis_clients: usize,
}

/// Global connection pool manager instance
lazy_static::lazy_static! {
    pub static ref GLOBAL_POOL_MANAGER: ConnectionPoolManager = {
        ConnectionPoolManager::new(PoolConfig::default())
    };
}

/// Helper function to get the global pool manager
pub fn get_pool_manager() -> &'static ConnectionPoolManager {
    &GLOBAL_POOL_MANAGER
}
