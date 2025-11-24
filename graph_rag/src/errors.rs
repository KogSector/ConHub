use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Neo4j error: {0}")]
    Neo4j(String),

    #[error("Vector database error: {0}")]
    VectorDb(String),

    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    #[error("Entity resolution failed: {0}")]
    ResolutionFailed(String),

    #[error("Invalid entity type: {0}")]
    InvalidEntityType(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl ResponseError for GraphError {
    fn error_response(&self) -> HttpResponse {
        let status_code = match self {
            GraphError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            GraphError::InvalidEntityType(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        HttpResponse::build(status_code).json(serde_json::json!({
            "error": self.to_string()
        }))
    }
}

pub type GraphResult<T> = Result<T, GraphError>;
