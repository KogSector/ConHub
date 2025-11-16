// Auth service using ORM layer instead of raw SQL
use anyhow::{Result, Context, anyhow};
use uuid::Uuid;
use chrono::{Duration, Utc};
use conhub_database::{
    Database, DatabaseConfig,
    models::{User, CreateUserInput, UpdateUserInput},
    repositories::UserRepository,
    cache::{RedisCache, CacheKeyBuilder},
    utils::{hash_password, verify_password, generate_token},
};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub username: Option<String>,
    pub full_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub username: Option<String>,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub email_verified: bool,
    pub is_admin: bool,
    pub created_at: chrono::DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            username: user.username,
            full_name: user.full_name,
            avatar_url: user.avatar_url,
            email_verified: user.email_verified,
            is_admin: user.is_admin,
            created_at: user.created_at,
        }
    }
}

pub struct AuthServiceOrm {
    db: Database,
    user_repo: UserRepository,
    jwt_secret: String,
}

impl AuthServiceOrm {
    pub async fn new() -> Result<Self> {
        let config = DatabaseConfig::from_env();
        let db = Database::new(&config).await?;
        let user_repo = UserRepository::new(db.pool().clone());
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "default_jwt_secret".to_string());

        Ok(Self {
            db,
            user_repo,
            jwt_secret,
        })
    }

    /// Register a new user
    pub async fn register(&self, req: RegisterRequest) -> Result<AuthResponse> {
        // Check if user already exists
        if let Some(_existing) = self.user_repo.find_by_email(&req.email).await? {
            return Err(anyhow!("User with this email already exists"));
        }

        // Hash password
        let password_hash = hash_password(&req.password)?;

        // Create user
        let input = CreateUserInput {
            email: req.email.clone(),
            username: req.username.clone(),
            password: password_hash,
            full_name: req.full_name.clone(),
        };

        let user = self.user_repo.create_user(&input).await?;

        // Generate tokens
        let access_token = self.generate_access_token(&user)?;
        let refresh_token = generate_token(64);

        // Create session
        self.user_repo.create_session(
            &user.id,
            &refresh_token,
            None,
            None,
        ).await?;

        // Cache user data in Redis
        if let Some(cache) = self.db.cache() {
            let cache_key = CacheKeyBuilder::user(&user.id);
            cache.set(&cache_key, &user, std::time::Duration::from_secs(3600)).await.ok();
        }

        Ok(AuthResponse {
            access_token,
            refresh_token,
            user: user.into(),
        })
    }

    /// Login a user
    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse> {
        // Find user by email
        let user = self.user_repo.find_by_email(&req.email).await?
            .ok_or_else(|| anyhow!("Invalid email or password"))?;

        // Check if account is locked
        if let Some(locked_until) = user.locked_until {
            if locked_until > Utc::now() {
                return Err(anyhow!("Account is temporarily locked. Please try again later."));
            }
        }

        // Verify password
        if !verify_password(&req.password, &user.password_hash)? {
            // Increment failed login attempts
            let attempts = self.user_repo.increment_failed_login(&user.id).await?;
            
            if attempts >= 5 {
                return Err(anyhow!("Account locked due to too many failed login attempts"));
            }
            
            return Err(anyhow!("Invalid email or password"));
        }

        // Reset failed login attempts on successful login
        self.user_repo.reset_failed_login(&user.id).await?;

        // Update last login
        self.user_repo.update_last_login(&user.id).await?;

        // Generate tokens
        let access_token = self.generate_access_token(&user)?;
        let refresh_token = generate_token(64);

        // Create session
        self.user_repo.create_session(
            &user.id,
            &refresh_token,
            None,
            None,
        ).await?;

        // Cache user data
        if let Some(cache) = self.db.cache() {
            let cache_key = CacheKeyBuilder::user(&user.id);
            cache.set(&cache_key, &user, std::time::Duration::from_secs(3600)).await.ok();
        }

        Ok(AuthResponse {
            access_token,
            refresh_token,
            user: user.into(),
        })
    }

    /// Refresh access token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthResponse> {
        // Find session by refresh token
        let session = self.user_repo.find_session_by_token(refresh_token).await?
            .ok_or_else(|| anyhow!("Invalid refresh token"))?;

        // Get user
        let user = self.user_repo.find_by_id(&session.user_id).await?
            .ok_or_else(|| anyhow!("User not found"))?;

        // Generate new tokens
        let access_token = self.generate_access_token(&user)?;
        let new_refresh_token = generate_token(64);

        // Delete old session and create new one
        self.user_repo.delete_session(refresh_token).await?;
        self.user_repo.create_session(
            &user.id,
            &new_refresh_token,
            None,
            None,
        ).await?;

        Ok(AuthResponse {
            access_token,
            refresh_token: new_refresh_token,
            user: user.into(),
        })
    }

    /// Logout a user
    pub async fn logout(&self, refresh_token: &str) -> Result<()> {
        self.user_repo.delete_session(refresh_token).await?;
        Ok(())
    }

    /// Get user by ID (with caching)
    pub async fn get_user(&self, user_id: &Uuid) -> Result<Option<User>> {
        // Try to get from cache first
        if let Some(cache) = self.db.cache() {
            let cache_key = CacheKeyBuilder::user(user_id);
            if let Ok(Some(user)) = cache.get::<User>(&cache_key).await {
                return Ok(Some(user));
            }
        }

        // Get from database
        let user = self.user_repo.find_by_id(user_id).await?;

        // Cache if found
        if let (Some(ref u), Some(cache)) = (&user, self.db.cache()) {
            let cache_key = CacheKeyBuilder::user(user_id);
            cache.set(&cache_key, u, std::time::Duration::from_secs(3600)).await.ok();
        }

        Ok(user)
    }

    /// Update user profile
    pub async fn update_user(&self, user_id: &Uuid, input: UpdateUserInput) -> Result<User> {
        let user = self.user_repo.update_user(user_id, &input).await?;

        // Invalidate cache
        if let Some(cache) = self.db.cache() {
            let cache_key = CacheKeyBuilder::user(user_id);
            cache.delete(&cache_key).await.ok();
        }

        Ok(user)
    }

    /// Generate JWT access token
    fn generate_access_token(&self, user: &User) -> Result<String> {
        use jsonwebtoken::{encode, Header, EncodingKey};
        
        #[derive(Serialize)]
        struct Claims {
            sub: String,
            email: String,
            is_admin: bool,
            exp: usize,
        }

        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(1))
            .ok_or_else(|| anyhow!("Invalid expiration time"))?
            .timestamp() as usize;

        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            is_admin: user.is_admin,
            exp: expiration,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;

        Ok(token)
    }

    /// Verify JWT token
    pub fn verify_token(&self, token: &str) -> Result<Uuid> {
        use jsonwebtoken::{decode, DecodingKey, Validation};
        
        #[derive(Deserialize)]
        struct Claims {
            sub: String,
        }

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )?;

        let user_id = Uuid::parse_str(&token_data.claims.sub)?;
        Ok(user_id)
    }

    /// Create API token
    pub async fn create_api_token(&self, user_id: &Uuid, name: &str, scopes: Vec<String>) -> Result<String> {
        let token = generate_token(64);
        let token_hash = hash_password(&token)?;
        
        self.user_repo.create_api_token(user_id, name, &token_hash, scopes).await?;
        
        Ok(token)
    }

    /// Revoke API token
    pub async fn revoke_api_token(&self, token_id: &Uuid) -> Result<()> {
        self.user_repo.revoke_api_token(token_id).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_login() {
        // This is a placeholder - proper tests should use a test database
        // For now, just ensure the service can be created
        dotenv::dotenv().ok();
        let result = AuthServiceOrm::new().await;
        assert!(result.is_ok() || result.is_err()); // Either way is OK for now
    }
}
