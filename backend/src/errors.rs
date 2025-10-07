use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Internal server error")]
    InternalServerError,
    #[error("Not found")]
    NotFound,
    #[error("Bad request: {0}")]
    BadRequest(String),
}