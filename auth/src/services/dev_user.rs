//! Development user service for Auth-disabled mode
//!
//! When the Auth feature toggle is false, this module provides helpers to:
//! 1. Ensure a dev user exists in the database (if DB is available)
//! 2. Return dev user profile for /me and /profile endpoints
//!
//! The dev user identity is defined in shared/models/src/auth.rs and is
//! consistent across all services and restarts.

use anyhow::{anyhow, Result};
use chrono::Utc;
use sqlx::PgPool;
use tracing;

use conhub_models::auth::{
    default_dev_user_id, default_dev_user_profile, User, UserProfile, UserRole, SubscriptionTier,
    DEFAULT_DEV_EMAIL, DEFAULT_DEV_NAME, DEFAULT_DEV_ORG,
};

/// Auth0-like subject identifier for the dev user
/// This mimics what Auth0 would provide for a real user
pub const DEV_AUTH0_SUB: &str = "dev|conhub-local-dev-user";

/// Ensure the development user exists in the database.
/// 
/// This function is idempotent - it will create the dev user if it doesn't exist,
/// or do nothing if it already exists.
/// 
/// Returns Ok(true) if user was created, Ok(false) if already existed.
pub async fn ensure_dev_user_exists(pool: &PgPool) -> Result<bool> {
    let dev_id = default_dev_user_id();
    
    // Check if dev user already exists by ID or email
    let exists = sqlx::query_scalar::<_, i32>(
        r#"SELECT 1 FROM users WHERE id = $1 OR email = $2 LIMIT 1"#
    )
    .bind(dev_id)
    .bind(DEFAULT_DEV_EMAIL)
    .fetch_optional(pool)
    .await
    .map(|opt| opt.is_some())
    .map_err(|e| anyhow!("Failed to check for dev user: {}", e))?;

    if exists {
        tracing::debug!("Dev user already exists (id={}, email={})", dev_id, DEFAULT_DEV_EMAIL);
        return Ok(false);
    }

    // Create the dev user
    let now = Utc::now();
    
    sqlx::query(
        r#"
        INSERT INTO users (
            id, email, password_hash, name, avatar_url, organization,
            role, subscription_tier, is_verified, is_active, auth0_sub,
            created_at, updated_at, email_verified_at
        ) VALUES (
            $1, $2, '', $3, NULL, $4,
            'user'::user_role, 'free'::subscription_tier, true, true, $5,
            $6, $7, $8
        )
        ON CONFLICT (id) DO NOTHING
        "#
    )
    .bind(dev_id)
    .bind(DEFAULT_DEV_EMAIL)
    .bind(DEFAULT_DEV_NAME)
    .bind(DEFAULT_DEV_ORG)
    .bind(DEV_AUTH0_SUB)
    .bind(now)
    .bind(now)
    .bind(now) // email_verified_at
    .execute(pool)
    .await
    .map_err(|e| anyhow!("Failed to create dev user: {}", e))?;

    tracing::info!(
        "✅ Created development user: id={}, email={}, auth0_sub={}",
        dev_id, DEFAULT_DEV_EMAIL, DEV_AUTH0_SUB
    );

    Ok(true)
}

/// Get the dev user from the database if it exists, otherwise return the
/// in-memory default profile.
pub async fn get_dev_user(pool: Option<&PgPool>) -> UserProfile {
    if let Some(pool) = pool {
        match fetch_dev_user_from_db(pool).await {
            Ok(Some(user)) => return user.into(),
            Ok(None) => {
                tracing::debug!("Dev user not found in DB, using in-memory profile");
            }
            Err(e) => {
                tracing::warn!("Failed to fetch dev user from DB: {}, using in-memory profile", e);
            }
        }
    }
    
    // Fallback to in-memory profile
    default_dev_user_profile()
}

/// Fetch the dev user from the database
async fn fetch_dev_user_from_db(pool: &PgPool) -> Result<Option<User>> {
    let dev_id = default_dev_user_id();
    
    let row = sqlx::query(
        r#"
        SELECT id, email, password_hash, name, avatar_url, organization,
               role::text as role, subscription_tier::text as subscription_tier,
               is_verified, is_active, is_locked, failed_login_attempts, locked_until,
               password_changed_at, email_verified_at, two_factor_enabled,
               two_factor_secret, backup_codes, created_at, updated_at,
               last_login_at, last_login_ip::text as last_login_ip, last_password_reset
        FROM users
        WHERE id = $1 AND is_active = true
        "#
    )
    .bind(dev_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| anyhow!("Database error: {}", e))?;

    match row {
        Some(row) => {
            use sqlx::Row;
            
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dev_user_constants() {
        assert_eq!(DEFAULT_DEV_EMAIL, "dev@conhub.local");
        assert_eq!(DEFAULT_DEV_NAME, "Development User");
        assert_eq!(DEFAULT_DEV_ORG, "ConHub Dev");
        assert_eq!(DEV_AUTH0_SUB, "dev|conhub-local-dev-user");
    }

    #[test]
    fn test_dev_user_id_is_stable() {
        let id1 = default_dev_user_id();
        let id2 = default_dev_user_id();
        assert_eq!(id1, id2);
        assert_eq!(id1.to_string(), "8f565516-5c3e-4d63-bc6f-1e049d4152ac");
    }

    #[test]
    fn test_default_dev_user_profile() {
        let profile = default_dev_user_profile();
        assert_eq!(profile.id, default_dev_user_id());
        assert_eq!(profile.email, DEFAULT_DEV_EMAIL);
        assert_eq!(profile.name, DEFAULT_DEV_NAME);
        assert_eq!(profile.organization, Some(DEFAULT_DEV_ORG.to_string()));
        assert!(profile.is_verified);
        assert!(profile.is_active);
    }
}
