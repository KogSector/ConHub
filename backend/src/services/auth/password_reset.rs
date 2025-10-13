use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetToken {
    pub token: String,
    pub email: String,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
}



type TokenStore = Arc<Mutex<HashMap<String, PasswordResetToken>>>;

pub struct PasswordResetService {
    tokens: TokenStore,
}

impl PasswordResetService {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn generate_reset_token(&self, email: &str) -> Result<String, String> {
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::hours(1); 
        
        let reset_token = PasswordResetToken {
            token: token.clone(),
            email: email.to_string(),
            expires_at,
            used: false,
        };

        let mut tokens = self.tokens.lock().map_err(|_| "Failed to acquire lock")?;
        
        
        tokens.retain(|_, token| token.expires_at > Utc::now());
        
        
        tokens.insert(token.clone(), reset_token);
        
        log::info!("Generated password reset token for email: {}", email);
        Ok(token)
    }

    pub fn validate_token(&self, token: &str) -> Result<String, String> {
        let mut tokens = self.tokens.lock().map_err(|_| "Failed to acquire lock")?;
        
        match tokens.get_mut(token) {
            Some(reset_token) => {
                if reset_token.used {
                    return Err("Token has already been used".to_string());
                }
                
                if reset_token.expires_at < Utc::now() {
                    tokens.remove(token);
                    return Err("Token has expired".to_string());
                }
                
                
                reset_token.used = true;
                
                Ok(reset_token.email.clone())
            }
            None => Err("Invalid token".to_string()),
        }
    }

    pub fn cleanup_expired_tokens(&self) {
        if let Ok(mut tokens) = self.tokens.lock() {
            let before_count = tokens.len();
            tokens.retain(|_, token| token.expires_at > Utc::now());
            let after_count = tokens.len();
            
            if before_count != after_count {
                log::info!("Cleaned up {} expired password reset tokens", before_count - after_count);
            }
        }
    }
}



lazy_static::lazy_static! {
    pub static ref PASSWORD_RESET_SERVICE: PasswordResetService = PasswordResetService::new();
}