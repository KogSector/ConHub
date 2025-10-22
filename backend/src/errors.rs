use thiserror::Error;
use actix_web::{HttpResponse, ResponseError};
use serde_json::json;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Internal server error")]
    InternalServerError,
    
    #[error("Not found")]
    NotFound,
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Mutex lock error: {0}")]
    MutexLockError(String),
    
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("External API error: {0}")]
    ExternalApiError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Time error: {0}")]
    TimeError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("IO error: {0}")]
    IoError(String),
}

impl From<sqlx::Error> for ServiceError {
    fn from(err: sqlx::Error) -> Self {
        ServiceError::DatabaseError(err.to_string())
    }
}

impl From<std::io::Error> for ServiceError {
    fn from(err: std::io::Error) -> Self {
        ServiceError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for ServiceError {
    fn from(err: serde_json::Error) -> Self {
        ServiceError::ParseError(err.to_string())
    }
}

impl From<std::time::SystemTimeError> for ServiceError {
    fn from(err: std::time::SystemTimeError) -> Self {
        ServiceError::TimeError(err.to_string())
    }
}

impl<T> From<std::sync::PoisonError<T>> for ServiceError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        ServiceError::MutexLockError(err.to_string())
    }
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::NotFound => HttpResponse::NotFound().json(json!({"error": self.to_string()})),
            ServiceError::BadRequest(_) => HttpResponse::BadRequest().json(json!({"error": self.to_string()})),
            ServiceError::ValidationError(_) => HttpResponse::BadRequest().json(json!({"error": self.to_string()})),
            ServiceError::AuthenticationError(_) => HttpResponse::Unauthorized().json(json!({"error": self.to_string()})),
            _ => HttpResponse::InternalServerError().json(json!({"error": "Internal server error"})),
        }
    }
}