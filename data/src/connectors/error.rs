use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConnectorError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Sync failed: {0}")]
    SyncFailed(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    
    #[error("Document not found: {0}")]
    DocumentNotFound(String),
    
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("HTTP error: {0}")]
    HttpError(String),
    
    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

// Removed sqlx dependency - not needed for GitHub connector

impl From<reqwest::Error> for ConnectorError {
    fn from(err: reqwest::Error) -> Self {
        ConnectorError::HttpError(err.to_string())
    }
}

impl From<std::io::Error> for ConnectorError {
    fn from(err: std::io::Error) -> Self {
        ConnectorError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for ConnectorError {
    fn from(err: serde_json::Error) -> Self {
        ConnectorError::SerializationError(err.to_string())
    }
}
