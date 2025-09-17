use crate::models::auth::*;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use sqlx::{Pool, Sqlite, Row};
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
    db: Pool<Sqlite>,
    jwt_secret: String,
    jwt_expiry_hours: i64,
}

impl AuthService {
    pub fn new(db: Pool<Sqlite>, jwt_secret: String) -> Self {
        Self {
            db,
            jwt_secret,
            jwt_expiry_hours: 24 * 7, // 7 days
        }
    }

    /// Initialize the database tables
    pub async fn init_database(&self) -> AuthResult<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                name TEXT NOT NULL,
                avatar_url TEXT,
                organization TEXT,
                role TEXT NOT NULL DEFAULT 'user',
                subscription_tier TEXT NOT NULL DEFAULT 'free',
                is_verified BOOLEAN NOT NULL DEFAULT FALSE,
                is_active BOOLEAN NOT NULL DEFAULT TRUE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_login_at TEXT
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
        let existing_user = sqlx::query("SELECT id FROM users WHERE email = ?")
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
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(user.id.to_string())
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.name)
        .bind(&user.avatar_url)
        .bind(&user.organization)
        .bind("user")
        .bind("free")
        .bind(user.is_verified)
        .bind(user.is_active)
        .bind(user.created_at.to_rfc3339())
        .bind(user.updated_at.to_rfc3339())
        .bind(user.last_login_at.map(|dt| dt.to_rfc3339()))
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
        let row = sqlx::query("SELECT * FROM users WHERE email = ? AND is_active = TRUE")
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
        sqlx::query("UPDATE users SET last_login_at = ?, updated_at = ? WHERE id = ?")
            .bind(now.to_rfc3339())
            .bind(now.to_rfc3339())
            .bind(user.id.to_string())
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
        let row = sqlx::query("SELECT * FROM users WHERE id = ? AND is_active = TRUE")
            .bind(user_id.to_string())
            .fetch_optional(&self.db)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        self.row_to_user(row)
    }

    /// Update user profile
    pub async fn update_profile(&self, user_id: Uuid, request: UpdateProfileRequest) -> AuthResult<UserProfile> {
        let mut query_parts = Vec::new();
        let mut values: Vec<String> = Vec::new();

        if let Some(name) = &request.name {
            query_parts.push("name = ?");
            values.push(name.clone());
        }
        if let Some(avatar_url) = &request.avatar_url {
            query_parts.push("avatar_url = ?");
            values.push(avatar_url.clone());
        }
        if let Some(organization) = &request.organization {
            query_parts.push("organization = ?");
            values.push(organization.clone());
        }

        if query_parts.is_empty() {
            return self.get_user_by_id(user_id).await.map(|u| u.into());
        }

        let now = Utc::now();
        query_parts.push("updated_at = ?");
        values.push(now.to_rfc3339());

        let query = format!("UPDATE users SET {} WHERE id = ?", query_parts.join(", "));
        values.push(user_id.to_string());

        let mut query_builder = sqlx::query(&query);
        for value in &values {
            query_builder = query_builder.bind(value);
        }

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
        sqlx::query("UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ?")
            .bind(&new_password_hash)
            .bind(now.to_rfc3339())
            .bind(user_id.to_string())
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
    fn row_to_user(&self, row: sqlx::sqlite::SqliteRow) -> AuthResult<User> {
        let role_str: String = row.try_get("role")?;
        let tier_str: String = row.try_get("subscription_tier")?;
        let created_at_str: String = row.try_get("created_at")?;
        let updated_at_str: String = row.try_get("updated_at")?;
        let last_login_str: Option<String> = row.try_get("last_login_at")?;

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
            id: Uuid::parse_str(&row.try_get::<String, _>("id")?)
                .map_err(|_| AuthError::DatabaseError(sqlx::Error::ColumnDecode { 
                    index: "id".to_string(), 
                    source: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UUID")) 
                }))?,
            email: row.try_get("email")?,
            password_hash: row.try_get("password_hash")?,
            name: row.try_get("name")?,
            avatar_url: row.try_get("avatar_url")?,
            organization: row.try_get("organization")?,
            role,
            subscription_tier,
            is_verified: row.try_get("is_verified")?,
            is_active: row.try_get("is_active")?,
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|_| AuthError::DatabaseError(sqlx::Error::ColumnDecode { 
                    index: "created_at".to_string(), 
                    source: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid datetime")) 
                }))?
                .with_timezone(&Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|_| AuthError::DatabaseError(sqlx::Error::ColumnDecode { 
                    index: "updated_at".to_string(), 
                    source: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid datetime")) 
                }))?
                .with_timezone(&Utc),
            last_login_at: last_login_str.map(|s| 
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now())
            ),
        })
    }
}