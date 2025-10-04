use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Repository connection failed: {0}")]
    RepositoryConnectionError(String),

    #[error("Indexing failed: {0}")]
    IndexingError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("External service error: {0}")]
    ExternalServiceError(String),

    #[error("An unknown error occurred")]
    Unknown,
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::ExternalServiceError(err.to_string())
    }
}
