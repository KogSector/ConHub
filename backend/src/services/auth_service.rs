use sqlx::PgPool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct AuthResult {
    pub token: String,
    pub user: UserData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserData {
    pub id: String,
    pub email: String,
    pub name: String,
}

#[derive(Debug)]
pub enum AuthError {
    InvalidCredentials,
    UserAlreadyExists,
    DatabaseError(String),
    JwtError(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidCredentials => write!(f, "Invalid credentials"),
            AuthError::UserAlreadyExists => write!(f, "User already exists"),
            AuthError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AuthError::JwtError(msg) => write!(f, "JWT error: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

pub struct AuthService {
    db_pool: Option<PgPool>,
    redis_client: Option<redis::Client>,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(db_pool: Option<PgPool>, redis_client: Option<redis::Client>, jwt_secret: String) -> Self {
        Self {
            db_pool,
            redis_client,
            jwt_secret,
        }
    }

    pub async fn authenticate_user(&self, email: &str, password: &str) -> Result<AuthResult, AuthError> {
        let base = std::env::var("AUTH_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3010".to_string());
        let url = format!("{}/api/auth/login", base);
        #[derive(Serialize)]
        struct LoginBody<'a> { email: &'a str, password: &'a str }
        #[derive(Deserialize)]
        struct UserDto { id: String, email: String, name: String }
        #[derive(Deserialize)]
        struct RespDto { token: String, user: UserDto }

        let client = reqwest::Client::new();
        let resp = client
            .post(url)
            .json(&LoginBody { email, password })
            .send()
            .await
            .map_err(|e| AuthError::DatabaseError(format!("Network error: {}", e)))?;

        let status = resp.status();
        if !status.is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            if status == reqwest::StatusCode::UNAUTHORIZED {
                return Err(AuthError::InvalidCredentials);
            }
            return Err(AuthError::DatabaseError(err_text));
        }

        let parsed: RespDto = resp.json().await.map_err(|e| AuthError::DatabaseError(format!("Parse error: {}", e)))?;
        Ok(AuthResult {
            token: parsed.token,
            user: UserData { id: parsed.user.id, email: parsed.user.email, name: parsed.user.name },
        })
    }

    pub async fn register_user(&self, email: &str, password: &str, name: &str) -> Result<AuthResult, AuthError> {
        let base = std::env::var("AUTH_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3010".to_string());
        let url = format!("{}/api/auth/register", base);
        #[derive(Serialize)]
        struct RegisterBody<'a> { email: &'a str, password: &'a str, name: &'a str }
        #[derive(Deserialize)]
        struct UserDto { id: String, email: String, name: String }
        #[derive(Deserialize)]
        struct RespDto { token: String, user: UserDto }

        let client = reqwest::Client::new();
        let resp = client
            .post(url)
            .json(&RegisterBody { email, password, name })
            .send()
            .await
            .map_err(|e| AuthError::DatabaseError(format!("Network error: {}", e)))?;
        let status = resp.status();
        if !status.is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            if status == reqwest::StatusCode::BAD_REQUEST {
                if err_text.to_lowercase().contains("already") { return Err(AuthError::UserAlreadyExists); }
            }
            return Err(AuthError::DatabaseError(err_text));
        }

        let parsed: RespDto = resp.json().await.map_err(|e| AuthError::DatabaseError(format!("Parse error: {}", e)))?;
        Ok(AuthResult {
            token: parsed.token,
            user: UserData { id: parsed.user.id, email: parsed.user.email, name: parsed.user.name },
        })
    }

    pub async fn validate_token(&self, token: &str) -> Result<UserData, AuthError> {
        // TODO: Call conhub-auth module when it's created
        log::info!("Validating token");

        Err(AuthError::JwtError("Not implemented".to_string()))
    }

    pub async fn logout_user(&self, token: &str) -> Result<(), AuthError> {
        // TODO: Call conhub-auth module when it's created
        log::info!("Logging out user");

        Ok(())
    }
}
