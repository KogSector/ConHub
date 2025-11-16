use async_trait::async_trait;
use anyhow::{Result, Context};
use sqlx::{PgPool, query_as, query};
use uuid::Uuid;

use crate::models::{User, CreateUserInput, UpdateUserInput, UserSession, ApiToken, Model, Pagination, PaginatedResult};
use super::Repository;

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, input: &CreateUserInput) -> Result<User> {
        let user = query_as!(
            User,
            r#"
            INSERT INTO users (email, name, password_hash, organization, role, subscription_tier)
            VALUES ($1, $2, $3, $4, 'user', 'free')
            RETURNING *
            "#,
            input.email,
            input.name,
            input.password,
            input.organization
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to create user")?;

        Ok(user)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = query_as!(
            User,
            "SELECT * FROM users WHERE email = $1",
            email
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find user by email")?;

        Ok(user)
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<User>> {
        let user = query_as!(
            User,
            "SELECT * FROM users WHERE name = $1",
            name
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find user by name")?;

        Ok(user)
    }

    pub async fn update_user(&self, id: &Uuid, input: &UpdateUserInput) -> Result<User> {
        let user = query_as!(
            User,
            r#"
            UPDATE users
            SET name = COALESCE($1, name),
                avatar_url = COALESCE($2, avatar_url),
                organization = COALESCE($3, organization),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $4
            RETURNING *
            "#,
            input.name,
            input.avatar_url,
            input.organization,
            id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to update user")?;

        Ok(user)
    }

    pub async fn update_last_login(&self, id: &Uuid) -> Result<()> {
        query!(
            "UPDATE users SET last_login_at = CURRENT_TIMESTAMP WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await
        .context("Failed to update last login")?;

        Ok(())
    }

    pub async fn increment_failed_login(&self, id: &Uuid) -> Result<i32> {
        let result = query!(
            r#"
            UPDATE users 
            SET failed_login_attempts = failed_login_attempts + 1,
                locked_until = CASE 
                    WHEN failed_login_attempts + 1 >= 5 THEN CURRENT_TIMESTAMP + INTERVAL '15 minutes'
                    ELSE locked_until
                END
            WHERE id = $1
            RETURNING failed_login_attempts
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to increment failed login attempts")?;

        Ok(result.failed_login_attempts)
    }

    pub async fn reset_failed_login(&self, id: &Uuid) -> Result<()> {
        query!(
            "UPDATE users SET failed_login_attempts = 0, locked_until = NULL WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await
        .context("Failed to reset failed login attempts")?;

        Ok(())
    }

    pub async fn create_session(&self, user_id: &Uuid, refresh_token: &str, ip_address: Option<String>, user_agent: Option<String>) -> Result<UserSession> {
        let session = query_as!(
            UserSession,
            r#"
            INSERT INTO user_sessions (user_id, refresh_token, ip_address, user_agent, expires_at)
            VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP + INTERVAL '7 days')
            RETURNING *
            "#,
            user_id,
            refresh_token,
            ip_address,
            user_agent
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to create session")?;

        Ok(session)
    }

    pub async fn find_session_by_token(&self, refresh_token: &str) -> Result<Option<UserSession>> {
        let session = query_as!(
            UserSession,
            "SELECT * FROM user_sessions WHERE refresh_token = $1 AND expires_at > CURRENT_TIMESTAMP",
            refresh_token
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find session")?;

        Ok(session)
    }

    pub async fn delete_session(&self, refresh_token: &str) -> Result<()> {
        query!(
            "DELETE FROM user_sessions WHERE refresh_token = $1",
            refresh_token
        )
        .execute(&self.pool)
        .await
        .context("Failed to delete session")?;

        Ok(())
    }

    pub async fn create_api_token(&self, user_id: &Uuid, name: &str, token_hash: &str, scopes: Vec<String>) -> Result<ApiToken> {
        let token = query_as!(
            ApiToken,
            r#"
            INSERT INTO api_tokens (user_id, name, token_hash, scopes)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
            user_id,
            name,
            token_hash,
            &scopes
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to create API token")?;

        Ok(token)
    }

    pub async fn list_api_tokens(&self, user_id: &Uuid) -> Result<Vec<ApiToken>> {
        let tokens = query_as!(
            ApiToken,
            "SELECT * FROM api_tokens WHERE user_id = $1 AND is_active = TRUE ORDER BY created_at DESC",
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to list API tokens")?;

        Ok(tokens)
    }

    pub async fn revoke_api_token(&self, id: &Uuid) -> Result<()> {
        query!(
            "UPDATE api_tokens SET is_active = FALSE WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await
        .context("Failed to revoke API token")?;

        Ok(())
    }
}

#[async_trait]
impl Repository<User> for UserRepository {
    async fn create(&self, entity: &User) -> Result<User> {
        let user = query_as!(
            User,
            r#"
            INSERT INTO users (id, email, username, password_hash, full_name, avatar_url, email_verified, is_active, is_admin)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
            entity.id,
            entity.email,
            entity.username,
            entity.password_hash,
            entity.full_name,
            entity.avatar_url,
            entity.email_verified,
            entity.is_active,
            entity.is_admin
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to create user")?;

        Ok(user)
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>> {
        let user = query_as!(
            User,
            "SELECT * FROM users WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find user")?;

        Ok(user)
    }

    async fn update(&self, id: &Uuid, entity: &User) -> Result<User> {
        let user = query_as!(
            User,
            r#"
            UPDATE users
            SET email = $1, username = $2, full_name = $3, avatar_url = $4,
                email_verified = $5, is_active = $6, is_admin = $7, updated_at = CURRENT_TIMESTAMP
            WHERE id = $8
            RETURNING *
            "#,
            entity.email,
            entity.username,
            entity.full_name,
            entity.avatar_url,
            entity.email_verified,
            entity.is_active,
            entity.is_admin,
            id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to update user")?;

        Ok(user)
    }

    async fn delete(&self, id: &Uuid) -> Result<bool> {
        let result = query!(
            "DELETE FROM users WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await
        .context("Failed to delete user")?;

        Ok(result.rows_affected() > 0)
    }

    async fn list(&self, pagination: &Pagination) -> Result<PaginatedResult<User>> {
        let total: i64 = query!("SELECT COUNT(*) as count FROM users")
            .fetch_one(&self.pool)
            .await
            .context("Failed to count users")?
            .count
            .unwrap_or(0);

        let users = query_as!(
            User,
            "SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            pagination.limit,
            pagination.offset
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to list users")?;

        Ok(PaginatedResult::new(users, total, pagination))
    }
}
