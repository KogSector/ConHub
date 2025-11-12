use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use sqlx::{PgPool, Row};
use bcrypt::{hash, verify, DEFAULT_COST};
use validator::Validate;
use serde_json::json;
use anyhow::{anyhow, Result};
use log;

use conhub_models::auth::*;
use super::security::SecurityService;

pub struct UserService {
    pool: PgPool,
    security_service: std::sync::Arc<SecurityService>,
}

impl UserService {
    pub async fn new(pool: PgPool) -> Result<Self> {
        let security_service = SecurityService::new(pool.clone()).await
            .map_err(|e| anyhow!("Failed to initialize security service: {}", e))?;
        
        Ok(Self {
            pool,
            security_service: std::sync::Arc::new(security_service),
        })
    }

    pub async fn create_user(&self, request: &RegisterRequest) -> Result<User> {
        
        if self.find_by_email(&request.email).await.is_ok() {
            return Err(anyhow!("User with this email already exists"));
        }

        // Validate password strength using SecurityService
        if let Err(e) = self.security_service.validate_password_strength(&request.password) {
            return Err(anyhow!("Password validation failed: {}", e));
        }

        // Hash password using Argon2 via SecurityService
        let password_hash = self.security_service.hash_password(&request.password)
            .map_err(|e| anyhow!("Failed to hash password: {}", e))?;

        let user_id = Uuid::new_v4();
        let now = Utc::now();
        
        
        let result = sqlx::query(
            r#"
            INSERT INTO users (
                id, email, password_hash, name, avatar_url, organization,
                role, subscription_tier, is_verified, is_active, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                'user'::user_role, 'free'::subscription_tier, $7, $8, $9, $10
            )
            "#
        )
        .bind(user_id)
        .bind(&request.email)
        .bind(&password_hash)
        .bind(&request.name)
        .bind(request.avatar_url.as_ref())
        .bind(request.organization.as_ref())
        .bind(false)
        .bind(true)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await;

        if let Err(e) = result {
            log::error!("Database error creating user: {:?}", e);
            return Err(anyhow!("Failed to create user in database: {}", e));
        }

        
        // Since we just created the user, we can construct it from the request data
        // instead of querying it back, avoiding the column mismatch issue
        log::info!("Successfully created user: {} ({})", request.email, user_id);

        Ok(User {
            id: user_id,
            email: request.email.clone(),
            password_hash,
            name: request.name.clone(),
            avatar_url: request.avatar_url.clone(),
            organization: request.organization.clone(),
            role: UserRole::User,
            subscription_tier: SubscriptionTier::Free,
            is_verified: false,
            is_active: true,
            is_locked: false,
            failed_login_attempts: 0,
            locked_until: None,
            password_changed_at: now,
            email_verified_at: None,
            two_factor_enabled: false,
            two_factor_secret: None,
            backup_codes: None,
            created_at: now,
            updated_at: now,
            last_login_at: None,
            last_login_ip: None,
            last_password_reset: None,
        })
    }

    
    pub async fn find_by_email(&self, email: &str) -> Result<User> {
        let row = sqlx::query(
            r#"
            SELECT id, email, password_hash, name, avatar_url, organization,
                   role::text as role, subscription_tier::text as subscription_tier,
                   is_verified, is_active, is_locked, failed_login_attempts, locked_until,
                   password_changed_at, email_verified_at, two_factor_enabled,
                   two_factor_secret, backup_codes, created_at, updated_at,
                   last_login_at, last_login_ip, last_password_reset
            FROM users
            WHERE email = $1 AND is_active = true
            "#
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            log::error!("User not found by email {}: {:?}", email, e);
            anyhow!("User not found")
        })?;

        let role_str: Option<String> = row.get("role");
        let subscription_tier_str: Option<String> = row.get("subscription_tier");

        Ok(User {
            id: row.get("id"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            name: row.get("name"),
            avatar_url: row.get("avatar_url"),
            organization: row.get("organization"),
            role: match role_str.as_deref() {
                Some("admin") => UserRole::Admin,
                _ => UserRole::User,
            },
            subscription_tier: match subscription_tier_str.as_deref() {
                Some("personal") => SubscriptionTier::Personal,
                Some("team") => SubscriptionTier::Team,
                Some("enterprise") => SubscriptionTier::Enterprise,
                _ => SubscriptionTier::Free,
            },
            is_verified: row.get("is_verified"),
            is_active: row.get("is_active"),
            is_locked: row.get("is_locked"),
            failed_login_attempts: row.get("failed_login_attempts"),
            locked_until: row.get("locked_until"),
            password_changed_at: row.get("password_changed_at"),
            email_verified_at: row.get("email_verified_at"),
            two_factor_enabled: row.get("two_factor_enabled"),
            two_factor_secret: row.get("two_factor_secret"),
            backup_codes: row.get("backup_codes"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            last_login_at: row.get("last_login_at"),
            last_login_ip: row.get("last_login_ip"),
            last_password_reset: row.get("last_password_reset"),
        })
    }

    pub async fn verify_password(&self, email: &str, password: &str) -> Result<User> {
        let user = self.find_by_email(email).await?;
        
        // Check if account is locked
        if user.is_locked {
            return Err(anyhow!("Account is locked"));
        }

        // Check if locked_until has passed
        if let Some(locked_until) = user.locked_until {
            if locked_until > Utc::now() {
                return Err(anyhow!("Account is temporarily locked"));
            }
        }

        // Verify password
        match verify(password, &user.password_hash) {
            Ok(true) => {
                // Reset failed login attempts on successful login
                if user.failed_login_attempts > 0 {
                    sqlx::query("UPDATE users SET failed_login_attempts = 0, locked_until = NULL, updated_at = NOW() WHERE id = $1")
                        .bind(user.id)
                        .execute(&self.pool)
                        .await
                        .map_err(|e| anyhow!("Failed to reset failed login attempts: {}", e))?;
                }
                Ok(user)
            }
            Ok(false) => {
                // Increment failed login attempts
                let new_attempts = user.failed_login_attempts + 1;
                let max_attempts = 5; // This should come from config
                
                if new_attempts >= max_attempts {
                    // Lock the account
                    let lock_duration = Duration::minutes(15); // This should come from config
                    let locked_until = Utc::now() + lock_duration;
                    
                    sqlx::query("UPDATE users SET failed_login_attempts = $1, is_locked = true, locked_until = $2, updated_at = NOW() WHERE id = $3")
                        .bind(new_attempts)
                        .bind(locked_until)
                        .bind(user.id)
                        .execute(&self.pool)
                        .await
                        .map_err(|e| anyhow!("Failed to lock account: {}", e))?;
                    
                    Err(anyhow!("Invalid credentials. Account locked due to too many failed attempts"))
                } else {
                    // Just increment the counter
                    sqlx::query("UPDATE users SET failed_login_attempts = $1, updated_at = NOW() WHERE id = $2")
                        .bind(new_attempts)
                        .bind(user.id)
                        .execute(&self.pool)
                        .await
                        .map_err(|e| anyhow!("Failed to update failed login attempts: {}", e))?;
                    
                    Err(anyhow!("Invalid credentials"))
                }
            }
            Err(e) => Err(anyhow!("Password verification failed: {}", e)),
        }
    }

    pub async fn update_last_login(&self, user_id: Uuid) -> Result<()> {
        let client_ip = "127.0.0.1"; // This should be extracted from the request
        
        sqlx::query(
            r#"
            UPDATE users 
            SET last_login_at = NOW(), last_login_ip = $1::inet, updated_at = NOW() 
            WHERE id = $2
            "#
        )
        .bind(client_ip)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to update last login: {}", e))?;

        Ok(())
    }

    pub async fn update_user(&self, user_id: Uuid, name: Option<String>, avatar_url: Option<String>, organization: Option<String>) -> Result<User> {
        // Simple update implementation
        let row = sqlx::query(
            r#"
            UPDATE users 
            SET name = COALESCE($2, name),
                avatar_url = COALESCE($3, avatar_url),
                organization = COALESCE($4, organization),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, email, password_hash, name, avatar_url, organization,
                      role::text as role, subscription_tier::text as subscription_tier,
                      is_verified, is_active, is_locked, failed_login_attempts, locked_until,
                      password_changed_at, email_verified_at, two_factor_enabled,
                      two_factor_secret, backup_codes, created_at, updated_at,
                      last_login_at, last_login_ip, last_password_reset
            "#
        )
        .bind(user_id)
        .bind(name)
        .bind(avatar_url)
        .bind(organization)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to update profile: {}", e))?;

        let role_str: Option<String> = row.get("role");
        let subscription_tier_str: Option<String> = row.get("subscription_tier");

        Ok(User {
            id: row.get("id"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            name: row.get("name"),
            avatar_url: row.get("avatar_url"),
            organization: row.get("organization"),
            role: match role_str.as_deref() {
                Some("admin") => UserRole::Admin,
                _ => UserRole::User,
            },
            subscription_tier: match subscription_tier_str.as_deref() {
                Some("personal") => SubscriptionTier::Personal,
                Some("team") => SubscriptionTier::Team,
                Some("enterprise") => SubscriptionTier::Enterprise,
                _ => SubscriptionTier::Free,
            },
            is_verified: row.get("is_verified"),
            is_active: row.get("is_active"),
            is_locked: row.get("is_locked"),
            failed_login_attempts: row.get("failed_login_attempts"),
            locked_until: row.get("locked_until"),
            password_changed_at: row.get("password_changed_at"),
            email_verified_at: row.get("email_verified_at"),
            two_factor_enabled: row.get("two_factor_enabled"),
            two_factor_secret: row.get("two_factor_secret"),
            backup_codes: row.get("backup_codes"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            last_login_at: row.get("last_login_at"),
            last_login_ip: row.get("last_login_ip"),
            last_password_reset: row.get("last_password_reset"),
        })
    }

    pub async fn update_password(&self, user_id: Uuid, new_password_hash: &str) -> Result<()> {
        sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
            .bind(new_password_hash)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to update password: {}", e))?;

        Ok(())
    }

    pub async fn find_by_id(&self, user_id: Uuid) -> Result<User> {
        let row = sqlx::query(
            r#"
            SELECT id, email, password_hash, name, avatar_url, organization,
                   role::text as role, subscription_tier::text as subscription_tier,
                   is_verified, is_active, is_locked, failed_login_attempts, locked_until,
                   password_changed_at, email_verified_at, two_factor_enabled,
                   two_factor_secret, backup_codes, created_at, updated_at,
                   last_login_at, last_login_ip, last_password_reset
            FROM users
            WHERE id = $1 AND is_active = true
            "#
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            log::error!("User not found by id {}: {:?}", user_id, e);
            anyhow!("User not found")
        })?;

        let role_str: Option<String> = row.get("role");
        let subscription_tier_str: Option<String> = row.get("subscription_tier");

        Ok(User {
            id: row.get("id"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            name: row.get("name"),
            avatar_url: row.get("avatar_url"),
            organization: row.get("organization"),
            role: match role_str.as_deref() {
                Some("admin") => UserRole::Admin,
                _ => UserRole::User,
            },
            subscription_tier: match subscription_tier_str.as_deref() {
                Some("personal") => SubscriptionTier::Personal,
                Some("team") => SubscriptionTier::Team,
                Some("enterprise") => SubscriptionTier::Enterprise,
                _ => SubscriptionTier::Free,
            },
            is_verified: row.get("is_verified"),
            is_active: row.get("is_active"),
            is_locked: row.get("is_locked"),
            failed_login_attempts: row.get("failed_login_attempts"),
            locked_until: row.get("locked_until"),
            password_changed_at: row.get("password_changed_at"),
            email_verified_at: row.get("email_verified_at"),
            two_factor_enabled: row.get("two_factor_enabled"),
            two_factor_secret: row.get("two_factor_secret"),
            backup_codes: row.get("backup_codes"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            last_login_at: row.get("last_login_at"),
            last_login_ip: row.get("last_login_ip"),
            last_password_reset: row.get("last_password_reset"),
        })
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>> {
        let row = sqlx::query(
            r#"
            SELECT id, email, password_hash, name, avatar_url, organization,
                   role::text as role, subscription_tier::text as subscription_tier,
                   is_verified, is_active, is_locked, failed_login_attempts, locked_until,
                   password_changed_at, email_verified_at, two_factor_enabled,
                   two_factor_secret, backup_codes, created_at, updated_at,
                   last_login_at, last_login_ip, last_password_reset
            FROM users
            WHERE id = $1 AND is_active = true
            "#
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            log::error!("Database error getting user by id {}: {:?}", user_id, e);
            anyhow!("Database error: {}", e)
        })?;

        match row {
            Some(row) => {
                let role_str: Option<String> = row.get("role");
                let subscription_tier_str: Option<String> = row.get("subscription_tier");

                Ok(Some(User {
                    id: row.get("id"),
                    email: row.get("email"),
                    password_hash: row.get("password_hash"),
                    name: row.get("name"),
                    avatar_url: row.get("avatar_url"),
                    organization: row.get("organization"),
                    role: match role_str.as_deref() {
                        Some("admin") => UserRole::Admin,
                        _ => UserRole::User,
                    },
                    subscription_tier: match subscription_tier_str.as_deref() {
                        Some("personal") => SubscriptionTier::Personal,
                        Some("team") => SubscriptionTier::Team,
                        Some("enterprise") => SubscriptionTier::Enterprise,
                        _ => SubscriptionTier::Free,
                    },
                    is_verified: row.get("is_verified"),
                    is_active: row.get("is_active"),
                    is_locked: row.get("is_locked"),
                    failed_login_attempts: row.get("failed_login_attempts"),
                    locked_until: row.get("locked_until"),
                    password_changed_at: row.get("password_changed_at"),
                    email_verified_at: row.get("email_verified_at"),
                    two_factor_enabled: row.get("two_factor_enabled"),
                    two_factor_secret: row.get("two_factor_secret"),
                    backup_codes: row.get("backup_codes"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                    last_login_at: row.get("last_login_at"),
                    last_login_ip: row.get("last_login_ip"),
                    last_password_reset: row.get("last_password_reset"),
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn list_users(&self, limit: i64, offset: i64) -> Result<Vec<User>> {
        let rows = sqlx::query(
            r#"
            SELECT id, email, password_hash, name, avatar_url, organization,
                   role::text as role, subscription_tier::text as subscription_tier,
                   is_verified, is_active, is_locked, failed_login_attempts, locked_until,
                   password_changed_at, email_verified_at, two_factor_enabled,
                   two_factor_secret, backup_codes, created_at, updated_at,
                   last_login_at, last_login_ip, last_password_reset
            FROM users
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to list users: {}", e))?;

        let mut users = Vec::new();
        for row in rows {
            let role_str: Option<String> = row.get("role");
            let subscription_tier_str: Option<String> = row.get("subscription_tier");

            users.push(User {
                id: row.get("id"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                name: row.get("name"),
                avatar_url: row.get("avatar_url"),
                organization: row.get("organization"),
                role: match role_str.as_deref() {
                    Some("admin") => UserRole::Admin,
                    _ => UserRole::User,
                },
                subscription_tier: match subscription_tier_str.as_deref() {
                    Some("personal") => SubscriptionTier::Personal,
                    Some("team") => SubscriptionTier::Team,
                    Some("enterprise") => SubscriptionTier::Enterprise,
                    _ => SubscriptionTier::Free,
                },
                is_verified: row.get("is_verified"),
                is_active: row.get("is_active"),
                is_locked: row.get("is_locked"),
                failed_login_attempts: row.get("failed_login_attempts"),
                locked_until: row.get("locked_until"),
                password_changed_at: row.get("password_changed_at"),
                email_verified_at: row.get("email_verified_at"),
                two_factor_enabled: row.get("two_factor_enabled"),
                two_factor_secret: row.get("two_factor_secret"),
                backup_codes: row.get("backup_codes"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                last_login_at: row.get("last_login_at"),
                last_login_ip: row.get("last_login_ip"),
                last_password_reset: row.get("last_password_reset"),
            });
        }

        Ok(users)
    }

    pub async fn delete_user(&self, user_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE users SET is_active = false, updated_at = NOW() WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to delete user: {}", e))?;

        Ok(())
    }

    pub async fn lock_user(&self, user_id: Uuid, locked_until: Option<DateTime<Utc>>) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE users 
            SET is_locked = true, locked_until = $1, updated_at = NOW() 
            WHERE id = $2
            "#
        )
        .bind(locked_until)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to lock user: {}", e))?;

        Ok(())
    }

    pub async fn unlock_user(&self, user_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE users 
            SET is_locked = false, locked_until = NULL, failed_login_attempts = 0, updated_at = NOW() 
            WHERE id = $1
            "#
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to unlock user: {}", e))?;

        Ok(())
    }
}