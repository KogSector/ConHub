use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token expired")]
    TokenExpired,
}

pub struct AuthService;

impl AuthService {
    pub fn new() -> Self {
        Self
    }

    pub fn verify_token(&self, _token: &str) -> Result<bool, AuthError> {
        Ok(true)
    }
}