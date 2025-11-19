use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub database_url: String,
    pub redis_url: Option<String>,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .or_else(|_| env::var("DATABASE_URL_NEON"))
                .expect("DATABASE_URL or DATABASE_URL_NEON must be set"),
            redis_url: env::var("REDIS_URL").ok(),
            max_connections: env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(20),
            min_connections: env::var("DB_MIN_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            connect_timeout_seconds: env::var("DB_CONNECT_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            idle_timeout_seconds: env::var("DB_IDLE_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(600),
        }
    }

    pub fn new(database_url: String, redis_url: Option<String>) -> Self {
        Self {
            database_url,
            redis_url,
            max_connections: 20,
            min_connections: 5,
            connect_timeout_seconds: 30,
            idle_timeout_seconds: 600,
        }
    }
}
