use async_trait::async_trait;
use anyhow::{Result, Context};
use sqlx::{PgPool, query_as, query};
use uuid::Uuid;

use crate::models::{SecurityEvent, RateLimitEntry, EncryptedSecret, CreateSecurityEventInput};

pub struct SecurityRepository {
    pool: PgPool,
}

impl SecurityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn log_security_event(&self, input: &CreateSecurityEventInput) -> Result<SecurityEvent> {
        query_as!(SecurityEvent,
            "INSERT INTO security_events (user_id, event_type, severity, ip_address, user_agent, details) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
            input.user_id, input.event_type, input.severity, input.ip_address, input.user_agent, input.details)
            .fetch_one(&self.pool).await.context("Failed to log security event")
    }

    pub async fn get_security_events(&self, user_id: &Uuid, limit: i32) -> Result<Vec<SecurityEvent>> {
        query_as!(SecurityEvent, "SELECT * FROM security_events WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2", user_id, limit)
            .fetch_all(&self.pool).await.context("Failed to get security events")
    }

    pub async fn check_rate_limit(&self, identifier: &str, endpoint: &str, max_requests: i32, window_seconds: i32) -> Result<bool> {
        let count: i64 = query!(
            "SELECT COUNT(*) as count FROM rate_limits WHERE identifier = $1 AND endpoint = $2 AND window_start > CURRENT_TIMESTAMP - INTERVAL '1 second' * $3",
            identifier, endpoint, window_seconds)
            .fetch_one(&self.pool).await?.count.unwrap_or(0);
        Ok(count < max_requests as i64)
    }

    pub async fn store_encrypted_secret(&self, user_id: &Uuid, key_name: &str, encrypted_value: &[u8], encryption_version: &str) -> Result<EncryptedSecret> {
        query_as!(EncryptedSecret,
            "INSERT INTO encrypted_secrets (user_id, key_name, encrypted_value, encryption_version) VALUES ($1, $2, $3, $4) ON CONFLICT (user_id, key_name) DO UPDATE SET encrypted_value = EXCLUDED.encrypted_value RETURNING *",
            user_id, key_name, encrypted_value, encryption_version)
            .fetch_one(&self.pool).await.context("Failed to store secret")
    }

    pub async fn get_encrypted_secret(&self, user_id: &Uuid, key_name: &str) -> Result<Option<EncryptedSecret>> {
        query_as!(EncryptedSecret, "SELECT * FROM encrypted_secrets WHERE user_id = $1 AND key_name = $2", user_id, key_name)
            .fetch_optional(&self.pool).await.context("Failed to get secret")
    }
}
