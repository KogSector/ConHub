use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::{Result, anyhow};
use bcrypt::{hash, verify, DEFAULT_COST};

use conhub_models::auth::{User, RegisterRequest};

pub struct UserService {
    pool: PgPool,
}

impl UserService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, request: &RegisterRequest) -> Result<User> {
        
        if self.find_by_email(&request.email).await.is_ok() {
            return Err(anyhow!("User with this email already exists"));
        }

        
        let password_hash = hash(&request.password, DEFAULT_COST)
            .map_err(|e| anyhow!("Failed to hash password: {}", e))?;

        let user_id = Uuid::new_v4();
        let now = Utc::now();
        
        
        let result = sqlx::query!(
            r#"
            INSERT INTO users (
                id, email, password_hash, name, avatar_url, organization,
                role, subscription_tier, is_verified, is_active, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                'user'::user_role, 'free'::subscription_tier, $7, $8, $9, $10
            )
            "#,
            user_id, &request.email, password_hash, &request.name,
            request.avatar_url.as_ref(), request.organization.as_ref(),
            false, true, now, now
        )
        .execute(&self.pool)
        .await;

        if let Err(e) = result {
            log::error!("Database error creating user: {:?}", e);
            return Err(anyhow!("Failed to create user in database: {}", e));
        }

        
        let row = sqlx::query!(
            r#"
            SELECT id, email, password_hash, name, avatar_url, organization,
                   role::text as role, subscription_tier::text as subscription_tier,
                   is_verified, is_active, created_at, updated_at, last_login_at
            FROM users
            WHERE id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch created user {}: {:?}", user_id, e);
            anyhow!("Failed to fetch created user: {}", e)
        })?;

        log::info!("Successfully created user: {} ({})", row.email, row.id);

        Ok(User {
            id: row.id,
            email: row.email,
            password_hash: row.password_hash,
            name: row.name,
            avatar_url: row.avatar_url,
            organization: row.organization,
            role: row.role.unwrap_or_else(|| "user".to_string()),
            subscription_tier: row.subscription_tier.unwrap_or_else(|| "free".to_string()),
            is_verified: row.is_verified,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
            last_login_at: row.last_login_at,
        })
    }

    
    pub async fn find_by_email(&self, email: &str) -> Result<User> {
        let row = sqlx::query!(
            r#"
            SELECT id, email, password_hash, name, avatar_url, organization,
                   role::text as role, subscription_tier::text as subscription_tier,
                   is_verified, is_active, created_at, updated_at, last_login_at
            FROM users
            WHERE email = $1 AND is_active = true
            "#,
            email
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow!("User not found: {}", e))?;

        Ok(User {
            id: row.id,
            email: row.email,
            password_hash: row.password_hash,
            name: row.name,
            avatar_url: row.avatar_url,
            organization: row.organization,
            role: row.role.unwrap_or_else(|| "user".to_string()),
            subscription_tier: row.subscription_tier.unwrap_or_else(|| "free".to_string()),
            is_verified: row.is_verified,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
            last_login_at: row.last_login_at,
        })
    }

    
    pub async fn find_by_id(&self, user_id: Uuid) -> Result<User> {
        let row = sqlx::query!(
            r#"
            SELECT id, email, password_hash, name, avatar_url, organization,
                   role::text as role, subscription_tier::text as subscription_tier,
                   is_verified, is_active, created_at, updated_at, last_login_at
            FROM users
            WHERE id = $1 AND is_active = true
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow!("User not found: {}", e))?;

        Ok(User {
            id: row.id,
            email: row.email,
            password_hash: row.password_hash,
            name: row.name,
            avatar_url: row.avatar_url,
            organization: row.organization,
            role: row.role.unwrap_or_else(|| "user".to_string()),
            subscription_tier: row.subscription_tier.unwrap_or_else(|| "free".to_string()),
            is_verified: row.is_verified,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
            last_login_at: row.last_login_at,
        })
    }

    
    pub async fn verify_password(&self, email: &str, password: &str) -> Result<User> {
        let user = self.find_by_email(email).await?;
        
        if !verify(password, &user.password_hash)
            .map_err(|e| anyhow!("Password verification failed: {}", e))? {
            return Err(anyhow!("Invalid password"));
        }

        Ok(user)
    }

    
    pub async fn update_last_login(&self, user_id: Uuid) -> Result<()> {
        sqlx::query!(
            "UPDATE users SET last_login_at = $1, updated_at = $1 WHERE id = $2",
            Utc::now(),
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to update last login: {}", e))?;

        Ok(())
    }

    
    pub async fn verify_user(&self, user_id: Uuid) -> Result<()> {
        sqlx::query!(
            "UPDATE users SET is_verified = true, updated_at = $1 WHERE id = $2",
            Utc::now(),
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to verify user: {}", e))?;

        Ok(())
    }

    
    pub async fn update_password(&self, user_id: Uuid, new_password: &str) -> Result<()> {
        let password_hash = hash(new_password, DEFAULT_COST)
            .map_err(|e| anyhow!("Failed to hash password: {}", e))?;

        sqlx::query!(
            "UPDATE users SET password_hash = $1, updated_at = $2 WHERE id = $3",
            password_hash,
            Utc::now(),
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to update password: {}", e))?;

        Ok(())
    }

    
    pub async fn update_profile(&self, user_id: Uuid, name: Option<String>, avatar_url: Option<String>, organization: Option<String>) -> Result<User> {
        let row = sqlx::query!(
            r#"
            UPDATE users 
            SET name = COALESCE($2, name),
                avatar_url = COALESCE($3, avatar_url),
                organization = COALESCE($4, organization),
                updated_at = $5
            WHERE id = $1
            RETURNING id, email, password_hash, name, avatar_url, organization,
                      role::text as role, subscription_tier::text as subscription_tier,
                      is_verified, is_active, created_at, updated_at, last_login_at
            "#,
            user_id,
            name,
            avatar_url,
            organization,
            Utc::now()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to update profile: {}", e))?;

        Ok(User {
            id: row.id,
            email: row.email,
            password_hash: row.password_hash,
            name: row.name,
            avatar_url: row.avatar_url,
            organization: row.organization,
            role: row.role.unwrap_or_else(|| "user".to_string()),
            subscription_tier: row.subscription_tier.unwrap_or_else(|| "free".to_string()),
            is_verified: row.is_verified,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
            last_login_at: row.last_login_at,
        })
    }

    
    pub async fn deactivate_user(&self, user_id: Uuid) -> Result<()> {
        sqlx::query!(
            "UPDATE users SET is_active = false, updated_at = $1 WHERE id = $2",
            Utc::now(),
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to deactivate user: {}", e))?;

        Ok(())
    }

    
    pub async fn get_user_count(&self) -> Result<i64> {
        let row = sqlx::query!("SELECT COUNT(*) as count FROM users WHERE is_active = true")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to get user count: {}", e))?;

        Ok(row.count.unwrap_or(0))
    }

    
    pub async fn list_users(&self, limit: i64, offset: i64) -> Result<Vec<User>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, email, password_hash, name, avatar_url, organization,
                   role::text as role, subscription_tier::text as subscription_tier,
                   is_verified, is_active, created_at, updated_at, last_login_at
            FROM users
            WHERE is_active = true
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to list users: {}", e))?;

        let users = rows.into_iter().map(|row| User {
            id: row.id,
            email: row.email,
            password_hash: row.password_hash,
            name: row.name,
            avatar_url: row.avatar_url,
            organization: row.organization,
            role: row.role.unwrap_or_else(|| "user".to_string()),
            subscription_tier: row.subscription_tier.unwrap_or_else(|| "free".to_string()),
            is_verified: row.is_verified,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
            last_login_at: row.last_login_at,
        }).collect();

        Ok(users)
    }
}
