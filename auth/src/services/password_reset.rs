use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use sqlx::{PgPool, Row};
use anyhow::{anyhow, Result};
use log;

#[derive(Debug, Clone)]
pub struct PasswordResetToken {
    pub token: String,
    pub email: String,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
}

pub struct PasswordResetService {
    pool: PgPool,
}

impl PasswordResetService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn generate_reset_token(&self, email: &str) -> Result<String> {
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::hours(1);
        let now = Utc::now();

        // Clean up any existing tokens for this email
        sqlx::query("DELETE FROM password_reset_tokens WHERE email = $1")
            .bind(email)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to clean up existing tokens: {}", e))?;

        // Insert new token
        sqlx::query(
            r#"
            INSERT INTO password_reset_tokens (token, email, expires_at, used, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(&token)
        .bind(email)
        .bind(expires_at)
        .bind(false)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to create password reset token: {}", e))?;

        log::info!("Generated password reset token for email: {}", email);
        Ok(token)
    }

    pub async fn validate_token(&self, token: &str) -> Result<String> {
        let now = Utc::now();

        // First, clean up expired tokens
        sqlx::query("DELETE FROM password_reset_tokens WHERE expires_at < $1")
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to clean up expired tokens: {}", e))?;

        // Find and validate the token
        let row = sqlx::query(
            "SELECT email, expires_at, used FROM password_reset_tokens WHERE token = $1"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch token: {}", e))?;

        match row {
            Some(row) => {
                let email: String = row.get("email");
                let expires_at: DateTime<Utc> = row.get("expires_at");
                let used: bool = row.get("used");

                if used {
                    return Err(anyhow!("Token has already been used"));
                }

                if expires_at < now {
                    // Delete expired token
                    sqlx::query("DELETE FROM password_reset_tokens WHERE token = $1")
                        .bind(token)
                        .execute(&self.pool)
                        .await
                        .map_err(|e| anyhow!("Failed to delete expired token: {}", e))?;
                    return Err(anyhow!("Token has expired"));
                }

                // Mark token as used
                sqlx::query("UPDATE password_reset_tokens SET used = true, updated_at = $1 WHERE token = $2")
                    .bind(now)
                    .bind(token)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| anyhow!("Failed to mark token as used: {}", e))?;

                Ok(email)
            }
            None => Err(anyhow!("Invalid token")),
        }
    }

    pub async fn cleanup_expired_tokens(&self) -> Result<()> {
        let now = Utc::now();
        let result = sqlx::query("DELETE FROM password_reset_tokens WHERE expires_at < $1")
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to cleanup expired tokens: {}", e))?;

        if result.rows_affected() > 0 {
            log::info!("Cleaned up {} expired password reset tokens", result.rows_affected());
        }

        Ok(())
    }
}