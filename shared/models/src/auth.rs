use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use validator::Validate;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    User,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "subscription_tier", rename_all = "lowercase")]
pub enum SubscriptionTier {
    Free,
    Personal,
    Team,
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "session_status", rename_all = "lowercase")]
pub enum SessionStatus {
    Active,
    Expired,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "audit_event_type", rename_all = "snake_case")]
pub enum AuditEventType {
    LoginSuccess,
    LoginFailed,
    Logout,
    Register,
    PasswordChange,
    PasswordReset,
    ProfileUpdate,
    AccountLocked,
    AccountUnlocked,
    TokenRefresh,
    OauthLogin,
    OauthDisconnect,
    TwoFactorEnabled,
    TwoFactorDisabled,
    SuspiciousActivity,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub organization: Option<String>,
    pub role: UserRole,
    pub subscription_tier: SubscriptionTier,
    pub is_verified: bool,
    pub is_active: bool,
    pub is_locked: bool,
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub password_changed_at: DateTime<Utc>,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub two_factor_enabled: bool,
    pub two_factor_secret: Option<String>,
    pub backup_codes: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub last_login_ip: Option<String>,
    pub last_password_reset: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub session_token: String,
    pub refresh_token: String,
    pub device_info: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub location: Option<serde_json::Value>,
    pub status: SessionStatus,
    pub expires_at: DateTime<Utc>,
    pub refresh_expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SecurityAuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub event_type: AuditEventType,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<serde_json::Value>,
    pub risk_score: Option<i32>,
    pub location: Option<serde_json::Value>,
    pub session_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RateLimit {
    pub id: Uuid,
    pub identifier: String,
    pub action: String,
    pub attempts: i32,
    pub window_start: DateTime<Utc>,
    pub blocked_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub organization: Option<String>,
    pub role: UserRole,
    pub subscription_tier: SubscriptionTier,
    pub is_verified: bool,
    pub is_active: bool,
    pub two_factor_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 8, max = 128, message = "Password must be between 8 and 128 characters"))]
    #[validate(custom(function = "validate_password_strength", message = "Password must contain at least one uppercase letter, one lowercase letter, one number, and one special character"))]
    pub password: String,
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters"))]
    pub name: String,
    pub avatar_url: Option<String>,
    pub organization: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
    pub two_factor_code: Option<String>,
    pub remember_me: Option<bool>,
    pub device_info: Option<DeviceInfo>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeviceInfo {
    pub device_type: String,
    pub os: Option<String>,
    pub browser: Option<String>,
    pub device_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters"))]
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub organization: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    #[validate(length(min = 8, max = 128, message = "New password must be between 8 and 128 characters"))]
    #[validate(custom(function = "validate_password_strength", message = "Password must contain at least one uppercase letter, one lowercase letter, one number, and one special character"))]
    pub new_password: String,
    pub two_factor_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ForgotPasswordRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    pub token: String,
    #[validate(length(min = 8, max = 128, message = "Password must be between 8 and 128 characters"))]
    #[validate(custom(function = "validate_password_strength", message = "Password must contain at least one uppercase letter, one lowercase letter, one number, and one special character"))]
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub session_id: Option<Uuid>,
    pub logout_all: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct Enable2FARequest {
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct Verify2FARequest {
    pub code: String,
    pub backup_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Disable2FARequest {
    pub password: String,
    pub two_factor_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user: UserProfile,
    pub token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub session_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorSetupResponse {
    pub secret: String,
    pub qr_code_url: String,
    pub backup_codes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,      
    pub email: String,    
    pub roles: Vec<String>, 
    pub exp: usize,       
    pub iat: usize,       
    pub iss: String,      
    pub aud: String,      
    pub session_id: String,
    pub jti: String,      // JWT ID for token revocation
}

// Default development user helpers used when Auth is disabled
// These values are not stored in any database; they are generated at runtime
// and can be used to populate user-related fields in a dev environment.
pub const DEFAULT_DEV_EMAIL: &str = "dev@conhub.local";
pub const DEFAULT_DEV_NAME: &str = "Development User";
pub const DEFAULT_DEV_ORG: &str = "ConHub Dev";

// Deterministic dev user ID to keep consistency across services and restarts
pub fn default_dev_user_id() -> uuid::Uuid {
    uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, b"dev.conhub.local.user")
}

// Build a default development user profile for UI and service use
pub fn default_dev_user_profile() -> UserProfile {
    let now = chrono::Utc::now();
    UserProfile {
        id: default_dev_user_id(),
        email: DEFAULT_DEV_EMAIL.to_string(),
        name: DEFAULT_DEV_NAME.to_string(),
        avatar_url: None,
        organization: Some(DEFAULT_DEV_ORG.to_string()),
        role: UserRole::User,
        subscription_tier: SubscriptionTier::Free,
        is_verified: true,
        is_active: true,
        two_factor_enabled: false,
        created_at: now,
        last_login_at: Some(now),
    }
}

// Build default JWT-like claims for development mode when Auth is disabled
pub fn default_dev_claims() -> Claims {
    let now = chrono::Utc::now().timestamp() as usize;
    let exp = now + 60 * 60 * 24 * 365; // 1 year
    let sub = default_dev_user_id().to_string();
    let session_id = uuid::Uuid::new_v4().to_string();
    let jti = uuid::Uuid::new_v4().to_string();

    Claims {
        sub,
        email: DEFAULT_DEV_EMAIL.to_string(),
        roles: vec!["user".to_string(), "dev".to_string()],
        exp,
        iat: now,
        iss: "conhub".to_string(),
        aud: "conhub-users".to_string(),
        session_id,
        jti,
    }
}

#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub id: Uuid,
    pub device_info: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub location: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
    pub is_current: bool,
}

#[derive(Debug, Serialize)]
pub struct UserSessionsResponse {
    pub sessions: Vec<SessionInfo>,
}

impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        UserProfile {
            id: user.id,
            email: user.email,
            name: user.name,
            avatar_url: user.avatar_url,
            organization: user.organization,
            role: user.role,
            subscription_tier: user.subscription_tier,
            is_verified: user.is_verified,
            is_active: user.is_active,
            two_factor_enabled: user.two_factor_enabled,
            created_at: user.created_at,
            last_login_at: user.last_login_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct OAuthCallbackRequest {
    #[validate(length(min = 1, message = "Provider is required"))]
    pub provider: String,
    pub provider_user_id: String,
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>,
    pub scope: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OAuthCallbackResponse {
    pub user_id: Uuid,
    pub is_new_user: bool,
    pub connection_id: Uuid,
}

// Password strength validation function
pub fn validate_password_strength(password: &str) -> Result<(), validator::ValidationError> {
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));
    
    if has_uppercase && has_lowercase && has_digit && has_special {
        Ok(())
    } else {
        Err(validator::ValidationError::new("password_strength"))
    }
}
