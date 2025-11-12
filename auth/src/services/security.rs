use std::collections::HashMap;
use std::sync::Arc;
use std::num::NonZeroU32;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::PgPool;
use serde_json::json;
use tokio::sync::RwLock;
use rsa::{RsaPrivateKey, RsaPublicKey, pkcs1::DecodeRsaPrivateKey, pkcs1::EncodeRsaPrivateKey, pkcs1::DecodeRsaPublicKey, pkcs1::EncodeRsaPublicKey};
use jsonwebtoken::{encode, decode, Header, Algorithm, EncodingKey, DecodingKey, Validation};
use governor::{Quota, RateLimiter, state::{InMemoryState, NotKeyed}, clock::DefaultClock};
use nonzero_ext::*;
use ring::rand::{SystemRandom, SecureRandom};
use base64::{Engine as _, engine::general_purpose};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::{rand_core::OsRng, SaltString}};

use conhub_models::auth::*;

pub struct SecurityService {
    pool: PgPool,
    private_key: RsaPrivateKey,
    public_key: RsaPublicKey,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    rate_limiters: Arc<RwLock<HashMap<String, Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>>>,
    rng: SystemRandom,
    argon2: Argon2<'static>,
}

impl SecurityService {
    pub async fn new(pool: PgPool) -> Result<Self, Box<dyn std::error::Error>> {
        let (private_key, public_key) = Self::load_or_generate_keys().await?;
        
        let private_key_pem = private_key.to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)?;
        let public_key_pem = public_key.to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)?;
        
        let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())?;
        let decoding_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes())?;
        
        Ok(Self {
            pool,
            private_key,
            public_key,
            encoding_key,
            decoding_key,
            rate_limiters: Arc::new(RwLock::new(HashMap::new())),
            rng: SystemRandom::new(),
            argon2: Argon2::default(),
        })
    }
    
    async fn load_or_generate_keys() -> Result<(RsaPrivateKey, RsaPublicKey), Box<dyn std::error::Error>> {
        // Try to load keys from environment variables first
        if let (Ok(private_pem), Ok(public_pem)) = (
            std::env::var("RSA_PRIVATE_KEY"),
            std::env::var("RSA_PUBLIC_KEY")
        ) {
            let private_key = RsaPrivateKey::from_pkcs1_pem(&private_pem)?;
            let public_key = RsaPublicKey::from_pkcs1_pem(&public_pem)?;
            return Ok((private_key, public_key));
        }
        
        // Generate new keys if not found
        tracing::info!("Generating new RSA key pair for JWT signing");
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, 2048)?;
        let public_key = RsaPublicKey::from(&private_key);
        
        // Log the keys for deployment (in production, store these securely)
        let private_pem = private_key.to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)?;
        let public_pem = public_key.to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)?;
        
        tracing::warn!("Generated RSA keys. Store these securely:");
        tracing::warn!("RSA_PRIVATE_KEY={}", private_pem.as_str());
        tracing::warn!("RSA_PUBLIC_KEY={}", public_pem.as_str());
        
        Ok((private_key, public_key))
    }
    
    pub fn get_public_key_pem(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.public_key.to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)?.to_string())
    }
    
    pub async fn generate_jwt_token(&self, user: &User, session_id: Uuid, remember_me: bool) -> Result<(String, String, DateTime<Utc>, DateTime<Utc>), Box<dyn std::error::Error>> {
        let now = Utc::now();
        let token_expires = if remember_me {
            now + chrono::Duration::days(30)
        } else {
            now + chrono::Duration::hours(24)
        };
        let refresh_expires = now + chrono::Duration::days(90);
        
        let jti = Uuid::new_v4().to_string();
        
        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            roles: vec![format!("{:?}", user.role).to_lowercase()],
            exp: token_expires.timestamp() as usize,
            iat: now.timestamp() as usize,
            iss: "conhub-auth".to_string(),
            aud: "conhub-services".to_string(),
            session_id: session_id.to_string(),
            jti,
        };
        
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some("conhub-auth-key".to_string());
        
        let token = encode(&header, &claims, &self.encoding_key)?;
        let refresh_token = self.generate_refresh_token()?;
        
        Ok((token, refresh_token, token_expires, refresh_expires))
    }
    
    pub async fn verify_jwt_token(&self, token: &str) -> Result<Claims, Box<dyn std::error::Error>> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&["conhub-auth"]);
        validation.set_audience(&["conhub-services"]);
        
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;
        
        // Check if token is revoked (you might want to implement a token blacklist)
        // self.check_token_revocation(&token_data.claims.jti).await?;
        
        Ok(token_data.claims)
    }
    
    fn generate_refresh_token(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut bytes = [0u8; 32];
        self.rng.fill(&mut bytes).map_err(|_| "Failed to generate random bytes")?;
        Ok(general_purpose::URL_SAFE_NO_PAD.encode(&bytes))
    }
    
    pub async fn check_rate_limit(&self, identifier: &str, action: &str, max_attempts: u32, window_minutes: u32) -> Result<bool, Box<dyn std::error::Error>> {
        // DEVELOPMENT: Skip database rate limiting to avoid persistent blocks
        if std::env::var("NODE_ENV").unwrap_or_default() == "development" {
            tracing::debug!("Skipping rate limit check in development mode");
            return Ok(true);
        }
        
        let key = format!("{}:{}", identifier, action);
        
        // Check database rate limit first
        let db_blocked = self.check_db_rate_limit(identifier, action, max_attempts, window_minutes).await?;
        if db_blocked {
            return Ok(false);
        }
        
        // Check in-memory rate limiter
        let quota = Quota::per_minute(NonZeroU32::new(max_attempts).unwrap_or(NonZeroU32::new(1).unwrap()));
        
        // Get or create rate limiter
        {
            let mut limiters = self.rate_limiters.write().await;
            if !limiters.contains_key(&key) {
                limiters.insert(key.clone(), Arc::new(RateLimiter::direct(quota)));
            }
        }
        
        // Get the rate limiter for checking
        let rate_limiter = {
            let limiters = self.rate_limiters.read().await;
            limiters.get(&key).unwrap().clone()
        };
        
        match rate_limiter.check() {
            Ok(_) => Ok(true),
            Err(_) => {
                // Update database with rate limit violation
                self.record_rate_limit_violation(identifier, action).await?;
                Ok(false)
            }
        }
    }
    
    async fn check_db_rate_limit(&self, identifier: &str, action: &str, max_attempts: u32, window_minutes: u32) -> Result<bool, Box<dyn std::error::Error>> {
        let window_start = Utc::now() - chrono::Duration::minutes(window_minutes as i64);
        
        let rate_limit = sqlx::query_as::<_, RateLimit>(
            "SELECT * FROM rate_limits WHERE identifier = $1 AND action = $2"
        )
        .bind(identifier)
        .bind(action)
        .fetch_optional(&self.pool)
        .await?;
        
        match rate_limit {
            Some(mut limit) => {
                if limit.window_start < window_start {
                    // Reset the window
                    limit.attempts = 1;
                    limit.window_start = Utc::now();
                    limit.blocked_until = None;
                    
                    sqlx::query(
                        "UPDATE rate_limits SET attempts = $1, window_start = $2, blocked_until = NULL, updated_at = NOW() WHERE id = $3"
                    )
                    .bind(limit.attempts)
                    .bind(limit.window_start)
                    .bind(limit.id)
                    .execute(&self.pool)
                    .await?;
                    
                    Ok(false)
                } else if limit.attempts >= max_attempts as i32 {
                    // Check if still blocked
                    if let Some(blocked_until) = limit.blocked_until {
                        if Utc::now() < blocked_until {
                            return Ok(true);
                        }
                    }
                    
                    // Block for exponential backoff
                    let block_duration = std::cmp::min(
                        Duration::from_secs(60 * (2_u64.pow(limit.attempts as u32 - max_attempts))),
                        Duration::from_secs(3600) // Max 1 hour
                    );
                    let blocked_until = Utc::now() + chrono::Duration::from_std(block_duration)?;
                    
                    sqlx::query(
                        "UPDATE rate_limits SET blocked_until = $1, updated_at = NOW() WHERE id = $2"
                    )
                    .bind(blocked_until)
                    .bind(limit.id)
                    .execute(&self.pool)
                    .await?;
                    
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            None => {
                // Create new rate limit entry
                sqlx::query(
                    "INSERT INTO rate_limits (identifier, action, attempts, window_start) VALUES ($1, $2, 1, NOW())"
                )
                .bind(identifier)
                .bind(action)
                .execute(&self.pool)
                .await?;
                
                Ok(false)
            }
        }
    }
    
    async fn record_rate_limit_violation(&self, identifier: &str, action: &str) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query(
            "UPDATE rate_limits SET attempts = attempts + 1, updated_at = NOW() WHERE identifier = $1 AND action = $2"
        )
        .bind(identifier)
        .bind(action)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn log_security_event(&self, user_id: Option<Uuid>, event_type: AuditEventType, ip_address: Option<String>, user_agent: Option<String>, details: Option<serde_json::Value>, risk_score: Option<i32>, session_id: Option<Uuid>) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query(
            "INSERT INTO security_audit_log (user_id, event_type, ip_address, user_agent, details, risk_score, session_id) VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(user_id)
        .bind(event_type)
        .bind(ip_address)
        .bind(user_agent)
        .bind(details)
        .bind(risk_score)
        .bind(session_id)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn detect_suspicious_activity(&self, user_id: Uuid, ip_address: Option<String>, user_agent: Option<String>) -> Result<i32, Box<dyn std::error::Error>> {
        let mut risk_score = 0;
        
        // Check for multiple IPs in short time
        let recent_ips = if let Some(_ip) = &ip_address {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(DISTINCT ip_address) FROM security_audit_log 
                 WHERE user_id = $1 AND created_at > NOW() - INTERVAL '1 hour' AND ip_address IS NOT NULL"
            )
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?
        } else {
            0
        };
        
        if recent_ips > 3 {
            risk_score += 30;
        }
        
        // Check for rapid login attempts
        let recent_logins = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM security_audit_log 
             WHERE user_id = $1 AND event_type = 'login_success' AND created_at > NOW() - INTERVAL '10 minutes'"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        
        if recent_logins > 5 {
            risk_score += 25;
        }
        
        // Check for failed attempts before success
        let recent_failures = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM security_audit_log 
             WHERE user_id = $1 AND event_type = 'login_failed' AND created_at > NOW() - INTERVAL '30 minutes'"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        
        if recent_failures > 3 {
            risk_score += 20;
        }
        
        // Log suspicious activity if risk score is high
        if risk_score >= 50 {
            self.log_security_event(
                Some(user_id),
                AuditEventType::SuspiciousActivity,
                ip_address,
                user_agent,
                Some(json!({
                    "risk_score": risk_score,
                    "recent_ips": recent_ips,
                    "recent_logins": recent_logins,
                    "recent_failures": recent_failures
                })),
                Some(risk_score),
                None
            ).await?;
        }
        
        Ok(risk_score)
    }
    
    pub async fn should_lock_account(&self, user_id: Uuid) -> Result<bool, Box<dyn std::error::Error>> {
        let failed_attempts = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM security_audit_log 
             WHERE user_id = $1 AND event_type = 'login_failed' AND created_at > NOW() - INTERVAL '15 minutes'"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(failed_attempts >= 5)
    }
    
    pub async fn lock_account(&self, user_id: Uuid, duration_minutes: i32) -> Result<(), Box<dyn std::error::Error>> {
        let locked_until = Utc::now() + chrono::Duration::minutes(duration_minutes as i64);
        
        sqlx::query(
            "UPDATE users SET is_locked = true, locked_until = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(locked_until)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        
        self.log_security_event(
            Some(user_id),
            AuditEventType::AccountLocked,
            None,
            None,
            Some(json!({
                "locked_until": locked_until,
                "duration_minutes": duration_minutes
            })),
            Some(75),
            None
        ).await?;
        
        Ok(())
    }
    
    pub async fn unlock_account(&self, user_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query(
            "UPDATE users SET is_locked = false, locked_until = NULL, failed_login_attempts = 0, updated_at = NOW() WHERE id = $1"
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        
        self.log_security_event(
            Some(user_id),
            AuditEventType::AccountUnlocked,
            None,
            None,
            None,
            Some(0),
            None
        ).await?;
        
        Ok(())
    }
    
    pub async fn generate_totp_secret(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut secret = [0u8; 20];
        self.rng.fill(&mut secret).map_err(|_| "Failed to generate TOTP secret")?;
        Ok(general_purpose::STANDARD.encode(&secret))
    }
    
    pub fn verify_totp(&self, secret: &str, code: &str) -> Result<bool, Box<dyn std::error::Error>> {
        // This is a simplified TOTP verification
        // In production, use a proper TOTP library like `totp-lite`
        let decoded_secret = general_purpose::STANDARD.decode(secret)?;
        
        let time_step = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs() / 30;
        
        // Check current time step and ±1 for clock skew
        for step in [time_step - 1, time_step, time_step + 1] {
            let expected_code = self.generate_totp_code(&decoded_secret, step)?;
            if constant_time_eq::constant_time_eq(code.as_bytes(), expected_code.as_bytes()) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    fn generate_totp_code(&self, secret: &[u8], time_step: u64) -> Result<String, Box<dyn std::error::Error>> {
        use ring::hmac;
        
        let key = hmac::Key::new(hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY, secret);
        let time_bytes = time_step.to_be_bytes();
        let signature = hmac::sign(&key, &time_bytes);
        let signature_bytes = signature.as_ref();
        
        let offset = (signature_bytes[19] & 0xf) as usize;
        let code = ((signature_bytes[offset] & 0x7f) as u32) << 24
            | ((signature_bytes[offset + 1] & 0xff) as u32) << 16
            | ((signature_bytes[offset + 2] & 0xff) as u32) << 8
            | (signature_bytes[offset + 3] & 0xff) as u32;
        
        Ok(format!("{:06}", code % 1_000_000))
    }
    
    pub async fn generate_backup_codes(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut codes = Vec::new();
        for _ in 0..10 {
            let mut bytes = [0u8; 6];
            self.rng.fill(&mut bytes).map_err(|_| "Failed to generate backup code")?;
            let code = bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
            codes.push(format!("{}-{}", &code[..6], &code[6..]));
        }
        Ok(codes)
    }

    // Password hashing and validation methods
    pub fn hash_password(&self, password: &str) -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self.argon2.hash_password(password.as_bytes(), &salt)?;
        Ok(password_hash.to_string())
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
        let parsed_hash = PasswordHash::new(hash)?;
        match self.argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub fn validate_password_strength(&self, password: &str) -> Result<(), String> {
        if password.len() < 8 {
            return Err("Password must be at least 8 characters long".to_string());
        }

        if !password.chars().any(|c| c.is_uppercase()) {
            return Err("Password must contain at least one uppercase letter".to_string());
        }

        if !password.chars().any(|c| c.is_lowercase()) {
            return Err("Password must contain at least one lowercase letter".to_string());
        }

        if !password.chars().any(|c| c.is_numeric()) {
            return Err("Password must contain at least one number".to_string());
        }

        if !password.chars().any(|c| !c.is_alphanumeric()) {
            return Err("Password must contain at least one special character".to_string());
        }

        Ok(())
    }
}