use thiserror::Error;

#[derive(Error, Debug)]
pub enum MCPError {
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    
    #[error("Tool execution failed: {0}")]
    ToolExecutionFailed(String),
    
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    
    #[error("Agent registration failed: {0}")]
    AgentRegistrationFailed(String),
    
    #[error("Context sync failed: {0}")]
    ContextSyncFailed(String),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<sqlx::Error> for MCPError {
    fn from(err: sqlx::Error) -> Self {
        MCPError::DatabaseError(err.to_string())
    }
}

impl From<serde_json::Error> for MCPError {
    fn from(err: serde_json::Error) -> Self {
        MCPError::SerializationError(err.to_string())
    }
}
