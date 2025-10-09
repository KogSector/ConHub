use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use validator::Validate;

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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    User,
    Moderator,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum SubscriptionTier {
    Free,
    Personal,
    Team,
    Enterprise,
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
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters"))]
    pub name: String,
    pub avatar_url: Option<String>,
    pub organization: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters"))]
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub organization: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    #[validate(length(min = 8, message = "New password must be at least 8 characters long"))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ForgotPasswordRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    pub token: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: UserProfile,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,      // Subject (user ID)
    pub email: String,    // User email
    pub roles: Vec<String>, // User roles
    pub exp: usize,       // Expiration time
    pub iat: usize,       // Issued at
    pub iss: String,      // Issuer
    pub aud: String,      // Audience
    pub session_id: String, // Session identifier
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
            created_at: user.created_at,
            last_login_at: user.last_login_at,
        }
    }
}

#[allow(dead_code)]
impl UserRole {
    pub fn can_access_admin(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::Moderator)
    }
}

#[allow(dead_code)]
impl SubscriptionTier {
    pub fn can_use_github_apps(&self) -> bool {
        matches!(self, SubscriptionTier::Team | SubscriptionTier::Enterprise)
    }
    
    pub fn max_repositories(&self) -> Option<u32> {
        match self {
            SubscriptionTier::Free => Some(3),
            SubscriptionTier::Personal => Some(20),
            SubscriptionTier::Team => Some(100),
            SubscriptionTier::Enterprise => None, // unlimited
        }
    }
}