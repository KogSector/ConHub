use std::fmt;

/// A shared error type that can be used across different services
#[derive(Debug, Clone)]
pub struct SharedError {
    message: String,
    source: Option<Box<SharedError>>,
}

impl SharedError {
    /// Create a new shared error with a message
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// Create a new shared error with a message and source
    pub fn with_source(message: impl Into<String>, source: SharedError) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Get the error message
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get the source error if any
    pub fn source(&self) -> Option<&SharedError> {
        self.source.as_deref()
    }
}

impl fmt::Display for SharedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(source) = &self.source {
            write!(f, ": {}", source)?;
        }
        Ok(())
    }
}

impl std::error::Error for SharedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<anyhow::Error> for SharedError {
    fn from(err: anyhow::Error) -> Self {
        Self::new(err.to_string())
    }
}

/// A shared result type
pub type SharedResult<T> = std::result::Result<T, SharedError>;

/// Helper function to create a successful SharedResult
pub fn shared_ok<T>(value: T) -> SharedResult<T> {
    Ok(value)
}

/// Extension trait for Result types to provide additional functionality
pub trait SharedResultExt<T> {
    /// Convert to a SharedResult
    fn into_shared_result(self) -> SharedResult<T>;
    
    /// Add context to the error
    fn with_context<F>(self, f: F) -> SharedResult<T>
    where
        F: FnOnce() -> String;
}

impl<T, E> SharedResultExt<T> for std::result::Result<T, E>
where
    E: Into<SharedError>,
{
    fn into_shared_result(self) -> SharedResult<T> {
        self.map_err(|e| e.into())
    }

    fn with_context<F>(self, f: F) -> SharedResult<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| SharedError::with_source(f(), e.into()))
    }
}

/// Extension trait for Result types with reference errors
pub trait SharedResultExtRef<T> {
    /// Add context to the error with a reference
    fn with_context_ref<F>(self, f: F) -> SharedResult<T>
    where
        F: FnOnce() -> String;
}

impl<T> SharedResultExtRef<T> for SharedResult<T> {
    fn with_context_ref<F>(self, f: F) -> SharedResult<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| SharedError::with_source(f(), e))
    }
}

/// Extension trait to convert `SharedResult<T>` into `anyhow::Result<T>` conveniently.
pub trait SharedResultExtAnyhow<T> {
    fn anyhow_result(self) -> anyhow::Result<T>;
}

impl<T> SharedResultExtAnyhow<T> for SharedResult<T> {
    fn anyhow_result(self) -> anyhow::Result<T> {
        self.map_err(|e| anyhow::anyhow!(e.to_string()))
    }
}

impl<'a, T> SharedResultExtAnyhow<&'a T> for &'a SharedResult<T> {
    fn anyhow_result(self) -> anyhow::Result<&'a T> {
        self.as_ref().map_err(|e| anyhow::anyhow!(e.to_string()))
    }
}