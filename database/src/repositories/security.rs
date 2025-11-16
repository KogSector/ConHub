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
        // Map to security_audit_log table with risk_score derived from severity
        let risk_score = match input.severity.as_str() {
            "low" => 20,
            "medium" => 50,
            "high" => 80,
            "critical" => 95,
            _ => 50,
        };
        
        sqlx::query_as::<_, SecurityEvent>(
            r#"
            INSERT INTO security_audit_log (
                user_id,
                event_type,
                ip_address,
                user_agent,
                details,
                risk_score
            ) VALUES (
                $1,
                $2::audit_event_type,
                $3::inet,
                $4,
                $5,
                $6
            )
            RETURNING
                id,
                user_id,
                event_type::text AS event_type,
                CASE
                    WHEN risk_score < 33 THEN 'low'
                    WHEN risk_score < 66 THEN 'medium'
                    ELSE 'high'
                END AS severity,
                ip_address::text AS ip_address,
                user_agent,
                details,
                created_at
            "#
        )
        .bind(input.user_id)
        .bind(&input.event_type)
        .bind(input.ip_address.as_deref())
        .bind(&input.user_agent)
        .bind(&input.details)
        .bind(risk_score)
        .fetch_one(&self.pool)
        .await
        .context("Failed to log security event")
    }

    pub async fn get_security_events(&self, user_id: &Uuid, limit: i32) -> Result<Vec<SecurityEvent>> {
        query_as!(
            SecurityEvent,
            r#"
            SELECT
                id,
                user_id,
                event_type::text AS "event_type!",
                CASE
                    WHEN risk_score < 33 THEN 'low'
                    WHEN risk_score < 66 THEN 'medium'
                    ELSE 'high'
                END AS "severity!",
                ip_address::text AS "ip_address",
                user_agent,
                details,
                created_at
            FROM security_audit_log
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            user_id,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to get security events")
    }

    pub async fn check_rate_limit(&self, identifier: &str, action: &str, max_requests: i32, window_seconds: i32) -> Result<bool> {
        let count: i64 = query!(
            r#"
            SELECT COUNT(*) as count
            FROM rate_limits
            WHERE identifier = $1
              AND action = $2
              AND window_start > CURRENT_TIMESTAMP - INTERVAL '1 second' * $3
            "#,
            identifier,
            action,
            window_seconds as f64
        )
        .fetch_one(&self.pool)
        .await?
        .count
        .unwrap_or(0);
        
        Ok(count < max_requests as i64)
    }

    pub async fn store_encrypted_secret(&self, _user_id: &Uuid, _key_name: &str, _encrypted_value: &[u8], _encryption_version: &str) -> Result<EncryptedSecret> {
        // Temporarily disabled until migration 010 is applied
        Err(anyhow::anyhow!("encrypted_secrets table not yet provisioned - run migration 010_security_extensions.sql"))
    }

    pub async fn get_encrypted_secret(&self, _user_id: &Uuid, _key_name: &str) -> Result<Option<EncryptedSecret>> {
        // Temporarily disabled until migration 010 is applied
        Ok(None)
    }
}
