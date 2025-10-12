use sqlx::PgPool;
use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};

use crate::models::auth::{User, RegisterRequest, LoginRequest};
use crate::services::auth::users::UserService;

pub struct LocalAuthService {
    user_service: UserService,
}

impl LocalAuthService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            user_service: UserService::new(pool),
        }
    }

    /// Register a new user with email and password
    pub async fn register(&self, request: &RegisterRequest) -> Result<User> {
        self.user_service.create_user(request).await
    }

    /// Authenticate user with email and password
    pub async fn login(&self, request: &LoginRequest) -> Result<User> {
        self.user_service.verify_password(&request.email, &request.password).await
    }

    /// Verify password hash
    pub fn verify_password_hash(password: &str, hash: &str) -> Result<bool> {
        verify(password, hash).map_err(|e| anyhow::anyhow!("Password verification failed: {}", e))
    }

    /// Hash password
    pub fn hash_password(password: &str) -> Result<String> {
        hash(password, DEFAULT_COST).map_err(|e| anyhow::anyhow!("Password hashing failed: {}", e))
    }
}
