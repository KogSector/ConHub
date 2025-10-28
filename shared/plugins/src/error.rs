use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginError {
    InitializationFailed(String),
    ConfigurationError(String),
    RuntimeError(String),
    NotFound(String),
    AlreadyExists(String),
    DependencyError(String),
    ValidationError(String),
    NetworkError(String),
    AuthenticationError(String),
    PermissionError(String),
    Unknown(String),
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::InitializationFailed(msg) => write!(f, "Plugin initialization failed: {}", msg),
            PluginError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            PluginError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            PluginError::NotFound(msg) => write!(f, "Not found: {}", msg),
            PluginError::AlreadyExists(msg) => write!(f, "Already exists: {}", msg),
            PluginError::DependencyError(msg) => write!(f, "Dependency error: {}", msg),
            PluginError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            PluginError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            PluginError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            PluginError::PermissionError(msg) => write!(f, "Permission error: {}", msg),
            PluginError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for PluginError {}

impl From<anyhow::Error> for PluginError {
    fn from(err: anyhow::Error) -> Self {
        PluginError::Unknown(err.to_string())
    }
}