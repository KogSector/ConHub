// Database ORM Layer for ConHub
// This provides a type-safe, repository-pattern based interface for all database operations

pub mod models;
pub mod repositories;
pub mod config;
pub mod cache;
pub mod utils;
pub mod graph;

// Re-export commonly used items
pub use sqlx;
pub use uuid;
pub use chrono;
pub use config::DatabaseConfig;
pub use cache::RedisCache;

use sqlx::{PgPool, postgres::PgPoolOptions};
use anyhow::{Result, Context};

/// Database connection manager
#[derive(Clone)]
pub struct Database {
    pool: PgPool,
    cache: Option<RedisCache>,
}

impl Database {
    /// Create a new database instance from configuration
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&config.database_url)
            .await
            .context("Failed to connect to database")?;

        let cache = if let Some(ref redis_url) = config.redis_url {
            match RedisCache::new(redis_url).await {
                Ok(c) => Some(c),
                Err(e) => {
                    tracing::warn!("Redis connection disabled due to error: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(Self { pool, cache })
    }

    /// Get the underlying connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get the Redis cache if available
    pub fn cache(&self) -> Option<&RedisCache> {
        self.cache.as_ref()
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .context("Failed to run migrations")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connection() {
        let config = DatabaseConfig::from_env();
        let db = Database::new(&config).await;
        assert!(db.is_ok());
    }
}
