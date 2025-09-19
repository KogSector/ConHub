use crate::models::auth::*;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Password hash error: {0}")]
    PasswordHashError(#[from] bcrypt::BcryptError),
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

pub type AuthResult<T> = Result<T, AuthError>;

pub struct AuthService {
    db: Pool<Postgres>,
    jwt_secret: String,
    jwt_expiry_hours: i64,
}

impl AuthService {
    pub fn new(db: Pool<Postgres>, jwt_secret: String) -> Self {
        Self {
            db,
            jwt_secret,
            jwt_expiry_hours: 24 * 7, // 7 days
        }
    }

    /// Initialize the database tables
    #[allow(dead_code)] // This method is kept for potential future use
    pub async fn init_database(&self) -> AuthResult<()> {
        sqlx::query(
            r#"
            -- Create enums if they don't exist
            DO $$ BEGIN
                CREATE TYPE user_role AS ENUM ('admin', 'user', 'moderator');
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
            
            DO $$ BEGIN
                CREATE TYPE subscription_tier AS ENUM ('free', 'personal', 'team', 'enterprise');
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
            
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                email VARCHAR(255) UNIQUE NOT NULL,
                password_hash VARCHAR(255) NOT NULL,
                name VARCHAR(255) NOT NULL,
                avatar_url TEXT,
                organization VARCHAR(255),
                role user_role NOT NULL DEFAULT 'user',
                subscription_tier subscription_tier NOT NULL DEFAULT 'free',
                is_verified BOOLEAN NOT NULL DEFAULT FALSE,
                is_active BOOLEAN NOT NULL DEFAULT TRUE,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                last_login_at TIMESTAMP WITH TIME ZONE
            );
            
            CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
            CREATE INDEX IF NOT EXISTS idx_users_active ON users(is_active);
            "#,
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }

    /// Register a new user
    pub async fn register(&self, request: RegisterRequest) -> AuthResult<AuthResponse> {
        // Check if user already exists
        let existing_user = sqlx::query("SELECT id FROM users WHERE email = $1")
            .bind(&request.email)
            .fetch_optional(&self.db)
            .await?;

        if existing_user.is_some() {
            return Err(AuthError::UserAlreadyExists);
        }

        // Hash the password
        let password_hash = hash(&request.password, DEFAULT_COST)?;
        
        // Create new user
        let user_id = Uuid::new_v4();
        let now = Utc::now();
        
        let user = User {
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
            created_at: now,
            updated_at: now,
            last_login_at: Some(now),
        };

        // Insert user into database
        sqlx::query(
            r#"
            INSERT INTO users (
                id, email, password_hash, name, avatar_url, organization, 
                role, subscription_tier, is_verified, is_active, 
                created_at, updated_at, last_login_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
        )
        .bind(user.id)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.name)
        .bind(&user.avatar_url)
        .bind(&user.organization)
        .bind(&user.role)
        .bind(&user.subscription_tier)
        .bind(user.is_verified)
        .bind(user.is_active)
        .bind(user.created_at)
        .bind(user.updated_at)
        .bind(user.last_login_at)
        .execute(&self.db)
        .await?;

        // Generate JWT token
        let token = self.generate_token(&user)?;
        let expires_at = now + Duration::hours(self.jwt_expiry_hours);

        Ok(AuthResponse {
            user: user.into(),
            token,
            expires_at,
        })
    }

    /// Authenticate user with email and password
    pub async fn login(&self, request: LoginRequest) -> AuthResult<AuthResponse> {
        // Find user by email
        let row = sqlx::query("SELECT * FROM users WHERE email = $1 AND is_active = TRUE")
            .bind(&request.email)
            .fetch_optional(&self.db)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        let user = self.row_to_user(row)?;

        // Verify password
        if !verify(&request.password, &user.password_hash)? {
            return Err(AuthError::InvalidCredentials);
        }

        // Update last login
        let now = Utc::now();
        sqlx::query("UPDATE users SET last_login_at = $1, updated_at = $2 WHERE id = $3")
            .bind(now)
            .bind(now)
            .bind(user.id)
            .execute(&self.db)
            .await?;

        // Generate JWT token
        let token = self.generate_token(&user)?;
        let expires_at = now + Duration::hours(self.jwt_expiry_hours);

        let mut updated_user = user;
        updated_user.last_login_at = Some(now);

        Ok(AuthResponse {
            user: updated_user.into(),
            token,
            expires_at,
        })
    }

    /// Verify JWT token and return user claims
    pub fn verify_token(&self, token: &str) -> AuthResult<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }

    /// Get user by ID
    pub async fn get_user_by_id(&self, user_id: Uuid) -> AuthResult<User> {
        let row = sqlx::query("SELECT * FROM users WHERE id = $1 AND is_active = TRUE")
            .bind(user_id)
            .fetch_optional(&self.db)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        self.row_to_user(row)
    }

    /// Update user profile
    pub async fn update_profile(&self, user_id: Uuid, request: UpdateProfileRequest) -> AuthResult<UserProfile> {
        let now = Utc::now();
        
        // Build update query with PostgreSQL syntax
        let mut query = "UPDATE users SET updated_at = $1".to_string();
        let mut param_count = 1;
        
        // We'll use a more straightforward approach for PostgreSQL
        if request.name.is_some() || request.avatar_url.is_some() || request.organization.is_some() {
            if request.name.is_some() {
                param_count += 1;
                query.push_str(&format!(", name = ${}", param_count));
            }
            if request.avatar_url.is_some() {
                param_count += 1;
                query.push_str(&format!(", avatar_url = ${}", param_count));
            }
            if request.organization.is_some() {
                param_count += 1;
                query.push_str(&format!(", organization = ${}", param_count));
            }
        }
        
        param_count += 1;
        query.push_str(&format!(" WHERE id = ${}", param_count));
        
        let mut query_builder = sqlx::query(&query).bind(now);
        
        if let Some(name) = &request.name {
            query_builder = query_builder.bind(name);
        }
        if let Some(avatar_url) = &request.avatar_url {
            query_builder = query_builder.bind(avatar_url);
        }
        if let Some(organization) = &request.organization {
            query_builder = query_builder.bind(organization);
        }
        
        query_builder = query_builder.bind(user_id);
        
        query_builder.execute(&self.db).await?;

        self.get_user_by_id(user_id).await.map(|u| u.into())
    }

    /// Change user password
    pub async fn change_password(&self, user_id: Uuid, request: ChangePasswordRequest) -> AuthResult<()> {
        let user = self.get_user_by_id(user_id).await?;

        // Verify current password
        if !verify(&request.current_password, &user.password_hash)? {
            return Err(AuthError::InvalidCredentials);
        }

        // Hash new password
        let new_password_hash = hash(&request.new_password, DEFAULT_COST)?;
        let now = Utc::now();

        // Update password
        sqlx::query("UPDATE users SET password_hash = $1, updated_at = $2 WHERE id = $3")
            .bind(&new_password_hash)
            .bind(now)
            .bind(user_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Generate JWT token for user
    fn generate_token(&self, user: &User) -> AuthResult<String> {
        let now = Utc::now();
        let exp = (now + Duration::hours(self.jwt_expiry_hours)).timestamp() as usize;
        let iat = now.timestamp() as usize;

        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            name: user.name.clone(),
            role: user.role.clone(),
            subscription_tier: user.subscription_tier.clone(),
            exp,
            iat,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;

        Ok(token)
    }

    /// Convert database row to User struct  
    fn row_to_user(&self, row: sqlx::postgres::PgRow) -> AuthResult<User> {
        let role_str: String = row.try_get("role")?;
        let tier_str: String = row.try_get("subscription_tier")?;
        let created_at: chrono::DateTime<Utc> = row.try_get("created_at")?;
        let updated_at: chrono::DateTime<Utc> = row.try_get("updated_at")?;
        let last_login_at: Option<chrono::DateTime<Utc>> = row.try_get("last_login_at")?;

        let role = match role_str.as_str() {
            "admin" => UserRole::Admin,
            "moderator" => UserRole::Moderator,
            _ => UserRole::User,
        };

        let subscription_tier = match tier_str.as_str() {
            "personal" => SubscriptionTier::Personal,
            "team" => SubscriptionTier::Team,
            "enterprise" => SubscriptionTier::Enterprise,
            _ => SubscriptionTier::Free,
        };

        Ok(User {
            id: row.try_get("id")?,
            email: row.try_get("email")?,
            password_hash: row.try_get("password_hash")?,
            name: row.try_get("name")?,
            avatar_url: row.try_get("avatar_url")?,
            organization: row.try_get("organization")?,
            role,
            subscription_tier,
            is_verified: row.try_get("is_verified")?,
            is_active: row.try_get("is_active")?,
            created_at,
            updated_at,
            last_login_at,
        })
    }

    /// Initiate password reset for a user
    /// For security reasons, this doesn't reveal if the email exists or not
    pub async fn initiate_password_reset(&self, email: &str) -> AuthResult<()> {
        // Check if user exists
        let user = sqlx::query("SELECT id, email, name FROM users WHERE email = $1 AND is_active = true")
            .bind(email)
            .fetch_optional(&self.db)
            .await?;

        if let Some(user_row) = user {
            let user_id: Uuid = user_row.try_get("id")?;
            let user_email: String = user_row.try_get("email")?;
            let _user_name: String = user_row.try_get("name")?; // Prefixed with _ to suppress warning

            // Generate a password reset token (valid for 1 hour)
            let reset_token = Uuid::new_v4().to_string();
            let expires_at = Utc::now() + Duration::hours(1);

            // Store the reset token in the database
            sqlx::query(
                "INSERT INTO password_reset_tokens (user_id, token, expires_at) 
                 VALUES ($1, $2, $3)
                 ON CONFLICT (user_id) 
                 DO UPDATE SET token = EXCLUDED.token, expires_at = EXCLUDED.expires_at, created_at = NOW()"
            )
            .bind(user_id)
            .bind(&reset_token)
            .bind(expires_at)
            .execute(&self.db)
            .await?;

            // TODO: Send email with reset link
            // For now, we'll just log it (in production, integrate with email service)
            log::info!(
                "Password reset token generated for user {}: {}. Reset link: http://localhost:3000/auth/reset-password?token={}",
                user_email, reset_token, reset_token
            );

            println!(
                "🔑 Password Reset Link for {}: http://localhost:3000/auth/reset-password?token={}",
                user_email, reset_token
            );
        }

        Ok(())
    }

    /// Reset password using token
    pub async fn reset_password(&self, token: &str, new_password: &str) -> AuthResult<()> {
        // Find valid token
        let result = sqlx::query(
            "SELECT user_id FROM password_reset_tokens 
             WHERE token = $1 AND expires_at > NOW()"
        )
        .bind(token)
        .fetch_optional(&self.db)
        .await?;

        let user_id = match result {
            Some(row) => row.try_get::<Uuid, _>("user_id")?,
            None => return Err(AuthError::InvalidCredentials), // Invalid or expired token
        };

        // Get current password hash to check if new password is different
        let current_hash = sqlx::query_scalar::<_, String>(
            "SELECT password_hash FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        // Check if new password is the same as current password
        if verify(new_password, &current_hash).unwrap_or(false) {
            return Err(AuthError::ValidationError("New password cannot be the same as your current password".to_string()));
        }

        // Hash the new password
        let new_password_hash = hash(new_password, DEFAULT_COST)?;

        // Update password
        sqlx::query(
            "UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(&new_password_hash)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        // Delete used token
        sqlx::query("DELETE FROM password_reset_tokens WHERE token = $1")
            .bind(token)
            .execute(&self.db)
            .await?;

        Ok(())
    }
}