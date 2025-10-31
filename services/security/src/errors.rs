use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Resource not found")]
    NotFound,
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Authentication error: {0}")]
    Auth(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Unauthorized access")]
    Unauthorized,
    
    #[error("Forbidden access")]
    Forbidden,
    
    #[error("Bad request: {0}")]
    BadRequest(String),
}