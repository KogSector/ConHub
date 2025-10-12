use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use uuid::Uuid;
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};
use base64::{engine::general_purpose, Engine as _};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub refresh_token_expiration_days: i64,
    pub max_login_attempts: u32,
    pub lockout_duration_minutes: i64,
    pub rate_limit_requests_per_minute: u32,
    pub csrf_token_expiration_minutes: i64,
    pub password_min_length: usize,
    pub require_special_chars: bool,
    pub require_numbers: bool,
    pub require_uppercase: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "super-secure-secret-key-change-in-production".to_string(),
            jwt_expiration_hours: 24,
            refresh_token_expiration_days: 30,
            max_login_attempts: 5,
            lockout_duration_minutes: 30,
            rate_limit_requests_per_minute: 100,
            csrf_token_expiration_minutes: 60,
            password_min_length: 8,
            require_special_chars: true,
            require_numbers: true,
            require_uppercase: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitEntry {
    pub requests: u32,
    pub window_start: Instant,
    pub last_request: Instant,
}

#[derive(Debug, Clone)]
pub struct LoginAttempt {
    pub attempts: u32,
    pub last_attempt: DateTime<Utc>,
    pub locked_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct CsrfToken {
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub user_id: String,
}

#[derive(Debug, Clone)]
pub struct SecureSession {
    pub session_id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub ip_address: String,
    pub user_agent: String,
    pub is_active: bool,
}

pub struct SecurityService {
    pub config: SecurityConfig,
    pub rsa_private_key: RsaPrivateKey,
    pub rsa_public_key: RsaPublicKey,
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
    pub aes_key: Key<Aes256Gcm>,
    pub rate_limits: Arc<DashMap<String, RateLimitEntry>>,
    pub login_attempts: Arc<DashMap<String, LoginAttempt>>,
    pub csrf_tokens: Arc<RwLock<HashMap<String, CsrfToken>>>,
    pub active_sessions: Arc<RwLock<HashMap<String, SecureSession>>>,
    pub argon2: Argon2<'static>,
}

impl SecurityService {
    pub fn new(config: SecurityConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Generate RSA keys for asymmetric encryption
        let mut rng = rand::thread_rng();
        let rsa_private_key = RsaPrivateKey::new(&mut rng, 2048)?;
        let rsa_public_key = RsaPublicKey::from(&rsa_private_key);
        
        // JWT keys
        let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_ref());
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_ref());
        
        // AES key for symmetric encryption
        let aes_key = Aes256Gcm::generate_key(&mut rng);
        
        // Argon2 for password hashing
        let argon2 = Argon2::default();

        Ok(Self {
            config,
            rsa_private_key,
            rsa_public_key,
            encoding_key,
            decoding_key,
            aes_key,
            rate_limits: Arc::new(DashMap::new()),
            login_attempts: Arc::new(DashMap::new()),
            csrf_tokens: Arc::new(RwLock::new(HashMap::new())),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            argon2,
        })
    }

    /// Generate a secure JWT token with RSA signing
    pub async fn generate_jwt_token(&self, user_id: String, email: String, roles: Vec<String>) -> Result<String, jsonwebtoken::errors::Error> {
        let session_id = Uuid::new_v4().to_string();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
        let exp = now + (self.config.jwt_expiration_hours * 3600) as usize;

        let claims = Claims {
            sub: user_id.clone(),
            email: email.clone(),
            roles,
            exp,
            iat: now,
            iss: "ConHub".to_string(),
            aud: "ConHub-Users".to_string(),
            session_id: session_id.clone(),
        };

        // Create secure session
        let session = SecureSession {
            session_id: session_id.clone(),
            user_id,
            created_at: Utc::now(),
            last_activity: Utc::now(),
            ip_address: "127.0.0.1".to_string(), // This should be passed from request
            user_agent: "Unknown".to_string(),   // This should be passed from request
            is_active: true,
        };

        {
            let mut sessions = self.active_sessions.write().await;
            sessions.insert(session_id, session);
        }

        encode(&Header::new(Algorithm::HS256), &claims, &self.encoding_key)
    }

    /// Verify and decode JWT token
    pub async fn verify_jwt_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;
        
        // Check if session is still active
        let sessions = self.active_sessions.read().await;
        if let Some(session) = sessions.get(&token_data.claims.session_id) {
            if !session.is_active {
                return Err(jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken));
            }
        } else {
            return Err(jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken));
        }

        Ok(token_data.claims)
    }

    /// Rate limiting middleware
    pub async fn check_rate_limit(&self, identifier: &str) -> bool {
        let now = Instant::now();
        let window_duration = std::time::Duration::from_secs(60); // 1 minute window
        
        let mut entry = self.rate_limits.entry(identifier.to_string()).or_insert(RateLimitEntry {
            requests: 0,
            window_start: now,
            last_request: now,
        });

        // Reset window if needed
        if now.duration_since(entry.window_start) >= window_duration {
            entry.requests = 0;
            entry.window_start = now;
        }

        entry.requests += 1;
        entry.last_request = now;

        entry.requests <= self.config.rate_limit_requests_per_minute
    }

    /// Check and update login attempts
    pub async fn check_login_attempts(&self, identifier: &str) -> bool {
        let now = Utc::now();
        
        let mut entry = self.login_attempts.entry(identifier.to_string()).or_insert(LoginAttempt {
            attempts: 0,
            last_attempt: now,
            locked_until: None,
        });

        // Check if account is locked
        if let Some(locked_until) = entry.locked_until {
            if now < locked_until {
                return false; // Still locked
            } else {
                // Reset after lockout period
                entry.attempts = 0;
                entry.locked_until = None;
            }
        }

        true // Not locked
    }

    /// Record failed login attempt
    pub async fn record_failed_login(&self, identifier: &str) {
        let now = Utc::now();
        
        let mut entry = self.login_attempts.entry(identifier.to_string()).or_insert(LoginAttempt {
            attempts: 0,
            last_attempt: now,
            locked_until: None,
        });

        entry.attempts += 1;
        entry.last_attempt = now;

        if entry.attempts >= self.config.max_login_attempts {
            entry.locked_until = Some(now + Duration::minutes(self.config.lockout_duration_minutes));
        }
    }

    /// Clear login attempts on successful login
    pub async fn clear_login_attempts(&self, identifier: &str) {
        self.login_attempts.remove(identifier);
    }

    /// Generate CSRF token
    pub async fn generate_csrf_token(&self, user_id: &str) -> String {
        let token = format!("{}-{}", Uuid::new_v4(), user_id);
        let csrf_token = CsrfToken {
            token: token.clone(),
            created_at: Utc::now(),
            user_id: user_id.to_string(),
        };

        let mut tokens = self.csrf_tokens.write().await;
        tokens.insert(token.clone(), csrf_token);
        
        // Clean up old tokens
        let cutoff = Utc::now() - Duration::minutes(self.config.csrf_token_expiration_minutes);
        tokens.retain(|_, token| token.created_at > cutoff);

        token
    }

    /// Verify CSRF token
    pub async fn verify_csrf_token(&self, token: &str, user_id: &str) -> bool {
        let tokens = self.csrf_tokens.read().await;
        if let Some(csrf_token) = tokens.get(token) {
            let not_expired = Utc::now() - csrf_token.created_at < Duration::minutes(self.config.csrf_token_expiration_minutes);
            let correct_user = csrf_token.user_id == user_id;
            not_expired && correct_user
        } else {
            false
        }
    }

    /// Hash password using Argon2
    pub fn hash_password(&self, password: &str) -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self.argon2.hash_password(password.as_bytes(), &salt)?;
        Ok(password_hash.to_string())
    }

    /// Verify password
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
        let parsed_hash = PasswordHash::new(hash)?;
        match self.argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Validate password strength
    pub fn validate_password_strength(&self, password: &str) -> Result<(), String> {
        if password.len() < self.config.password_min_length {
            return Err(format!("Password must be at least {} characters long", self.config.password_min_length));
        }

        if self.config.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err("Password must contain at least one uppercase letter".to_string());
        }

        if self.config.require_numbers && !password.chars().any(|c| c.is_numeric()) {
            return Err("Password must contain at least one number".to_string());
        }

        if self.config.require_special_chars && !password.chars().any(|c| !c.is_alphanumeric()) {
            return Err("Password must contain at least one special character".to_string());
        }

        Ok(())
    }

    /// Encrypt data using AES-256-GCM
    pub fn encrypt_data(&self, data: &str) -> Result<String, Box<dyn std::error::Error>> {
        let cipher = Aes256Gcm::new(&self.aes_key);
        let nonce = Aes256Gcm::generate_nonce(&mut rand::thread_rng());
        let ciphertext = cipher.encrypt(&nonce, data.as_bytes())?;
        
        // Combine nonce and ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        
        Ok(general_purpose::STANDARD.encode(&result))
    }

    /// Decrypt data using AES-256-GCM
    pub fn decrypt_data(&self, encrypted_data: &str) -> Result<String, Box<dyn std::error::Error>> {
        let data = general_purpose::STANDARD.decode(encrypted_data)?;
        
        if data.len() < 12 {
            return Err("Invalid encrypted data format".into());
        }
        
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let cipher = Aes256Gcm::new(&self.aes_key);
        let plaintext = cipher.decrypt(nonce, ciphertext)?;
        
        Ok(String::from_utf8(plaintext)?)
    }

    /// Encrypt data using RSA public key
    pub fn rsa_encrypt(&self, data: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut rng = rand::thread_rng();
        let encrypted = self.rsa_public_key.encrypt(&mut rng, Pkcs1v15Encrypt, data.as_bytes())?;
        Ok(general_purpose::STANDARD.encode(&encrypted))
    }

    /// Decrypt data using RSA private key
    pub fn rsa_decrypt(&self, encrypted_data: &str) -> Result<String, Box<dyn std::error::Error>> {
        let data = general_purpose::STANDARD.decode(encrypted_data)?;
        let decrypted = self.rsa_private_key.decrypt(Pkcs1v15Encrypt, &data)?;
        Ok(String::from_utf8(decrypted)?)
    }

    /// Create secure hash of data
    pub fn create_secure_hash(&self, data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Create HMAC signature
    pub fn create_hmac_signature(&self, data: &str) -> String {
        let mut hasher = Sha512::new();
        hasher.update(self.config.jwt_secret.as_bytes());
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verify HMAC signature
    pub fn verify_hmac_signature(&self, data: &str, signature: &str) -> bool {
        let expected_signature = self.create_hmac_signature(data);
        expected_signature == signature
    }

    /// Invalidate session
    pub async fn invalidate_session(&self, session_id: &str) -> bool {
        let mut sessions = self.active_sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.is_active = false;
            true
        } else {
            false
        }
    }

    /// Update session activity
    pub async fn update_session_activity(&self, session_id: &str) -> bool {
        let mut sessions = self.active_sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.last_activity = Utc::now();
            true
        } else {
            false
        }
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) {
        let cutoff = Utc::now() - Duration::hours(self.config.jwt_expiration_hours);
        let mut sessions = self.active_sessions.write().await;
        sessions.retain(|_, session| session.last_activity > cutoff && session.is_active);
    }

    /// Get session information
    pub async fn get_session_info(&self, session_id: &str) -> Option<SecureSession> {
        let sessions = self.active_sessions.read().await;
        sessions.get(session_id).cloned()
    }

    /// Generate secure random token
    pub fn generate_secure_token(&self, length: usize) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                abcdefghijklmnopqrstuvwxyz\
                                0123456789";
        let mut rng = rand::thread_rng();
        
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
}

/// Security middleware for rate limiting
pub async fn rate_limit_middleware(
    req: HttpRequest,
    security: web::Data<Arc<SecurityService>>,
) -> ActixResult<()> {
    let ip = req.peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    
    if !security.check_rate_limit(&ip).await {
        return Err(actix_web::error::ErrorTooManyRequests("Rate limit exceeded"));
    }
    
    Ok(())
}

/// Security utilities for data encoding/decoding
pub mod encoding {
    use base64::{engine::general_purpose, Engine as _};
    use hex;
    
    pub fn base64_encode(data: &[u8]) -> String {
        general_purpose::STANDARD.encode(data)
    }
    
    pub fn base64_decode(data: &str) -> Result<Vec<u8>, base64::DecodeError> {
        general_purpose::STANDARD.decode(data)
    }
    
    pub fn hex_encode(data: &[u8]) -> String {
        hex::encode(data)
    }
    
    pub fn hex_decode(data: &str) -> Result<Vec<u8>, hex::FromHexError> {
        hex::decode(data)
    }
    
    pub fn url_safe_encode(data: &[u8]) -> String {
        general_purpose::URL_SAFE_NO_PAD.encode(data)
    }
    
    pub fn url_safe_decode(data: &str) -> Result<Vec<u8>, base64::DecodeError> {
        general_purpose::URL_SAFE_NO_PAD.decode(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_service_creation() {
        let config = SecurityConfig::default();
        let security = SecurityService::new(config).unwrap();
        
        // Test password operations
        let password = "TestPassword123!";
        let hash = security.hash_password(password).unwrap();
        assert!(security.verify_password(password, &hash).unwrap());
        
        // Test encryption/decryption
        let data = "sensitive data";
        let encrypted = security.encrypt_data(data).unwrap();
        let decrypted = security.decrypt_data(&encrypted).unwrap();
        assert_eq!(data, decrypted);
    }

    #[tokio::test]
    async fn test_jwt_operations() {
        let config = SecurityConfig::default();
        let security = SecurityService::new(config).unwrap();
        
        let token = security.generate_jwt_token(
            "user123".to_string(),
            "test@example.com".to_string(),
            vec!["user".to_string()],
        ).await.unwrap();
        
        let claims = security.verify_jwt_token(&token).await.unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let config = SecurityConfig::default();
        let security = SecurityService::new(config).unwrap();
        
        // Should allow initial requests
        assert!(security.check_rate_limit("127.0.0.1").await);
        
        // Simulate many requests
        for _ in 0..100 {
            security.check_rate_limit("127.0.0.1").await;
        }
        
        // Should be rate limited now
        assert!(!security.check_rate_limit("127.0.0.1").await);
    }
}