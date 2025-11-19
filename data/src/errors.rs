use actix_web::{HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
pub enum ServiceError {
    InternalServerError(String),
    BadRequest(String),
    NotFound(String),
    Unauthorized(String),
    MutexLockError(String),
    DatabaseError(String),
    ValidationError(String),
    IoError(String),
    ExternalApiError(String),
    TimeError(String),
    ConfigurationError(String),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::InternalServerError(msg) => write!(f, "Internal Server Error: {}", msg),
            ServiceError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            ServiceError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            ServiceError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            ServiceError::MutexLockError(msg) => write!(f, "Mutex Lock Error: {}", msg),
            ServiceError::DatabaseError(msg) => write!(f, "Database Error: {}", msg),
            ServiceError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
            ServiceError::IoError(msg) => write!(f, "IO Error: {}", msg),
            ServiceError::ExternalApiError(msg) => write!(f, "External API Error: {}", msg),
            ServiceError::TimeError(msg) => write!(f, "Time Error: {}", msg),
            ServiceError::ConfigurationError(msg) => write!(f, "Configuration Error: {}", msg),
        }
    }
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::InternalServerError(msg) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": msg
                }))
            }
            ServiceError::BadRequest(msg) => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Bad Request",
                    "message": msg
                }))
            }
            ServiceError::NotFound(msg) => {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Not Found",
                    "message": msg
                }))
            }
            ServiceError::Unauthorized(msg) => {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Unauthorized",
                    "message": msg
                }))
            }
            ServiceError::MutexLockError(msg) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Mutex Lock Error",
                    "message": msg
                }))
            }
            ServiceError::DatabaseError(msg) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Database Error",
                    "message": msg
                }))
            }
            ServiceError::ValidationError(msg) => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Validation Error",
                    "message": msg
                }))
            }
            ServiceError::IoError(msg) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "IO Error",
                    "message": msg
                }))
            }
            ServiceError::ExternalApiError(msg) => {
                HttpResponse::BadGateway().json(serde_json::json!({
                    "error": "External API Error",
                    "message": msg
                }))
            }
            ServiceError::TimeError(msg) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Time Error",
                    "message": msg
                }))
            }
            ServiceError::ConfigurationError(msg) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Configuration Error",
                    "message": msg
                }))
            }
        }
    }
}

impl From<std::io::Error> for ServiceError {
    fn from(error: std::io::Error) -> Self {
        ServiceError::IoError(error.to_string())
    }
}

impl From<serde_json::Error> for ServiceError {
    fn from(error: serde_json::Error) -> Self {
        ServiceError::ValidationError(error.to_string())
    }
}

impl From<std::time::SystemTimeError> for ServiceError {
    fn from(error: std::time::SystemTimeError) -> Self {
        ServiceError::TimeError(error.to_string())
    }
}

impl From<reqwest::Error> for ServiceError {
    fn from(error: reqwest::Error) -> Self {
        ServiceError::ExternalApiError(error.to_string())
    }
}

impl From<(actix_web::http::StatusCode, String)> for ServiceError {
    fn from(error: (actix_web::http::StatusCode, String)) -> Self {
        match error.0 {
            actix_web::http::StatusCode::UNAUTHORIZED => ServiceError::Unauthorized(error.1),
            actix_web::http::StatusCode::NOT_FOUND => ServiceError::NotFound(error.1),
            actix_web::http::StatusCode::BAD_REQUEST => ServiceError::BadRequest(error.1),
            actix_web::http::StatusCode::FORBIDDEN => ServiceError::Unauthorized(error.1),
            _ => ServiceError::InternalServerError(error.1),
        }
    }
}
