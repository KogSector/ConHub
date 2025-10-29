use actix_web::{HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
pub enum ServiceError {
    InternalError(String),
    BadRequest(String),
    NotFound(String),
    Unauthorized(String),
    DatabaseError(String),
    StripeError(String),
    ValidationError(String),
    ParseError(String),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::InternalError(msg) => write!(f, "Internal Error: {}", msg),
            ServiceError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            ServiceError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            ServiceError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            ServiceError::DatabaseError(msg) => write!(f, "Database Error: {}", msg),
            ServiceError::StripeError(msg) => write!(f, "Stripe error: {}", msg),
            ServiceError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ServiceError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::InternalError(msg) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Internal server error",
                    "message": msg
                }))
            }
            ServiceError::BadRequest(msg) => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Bad request",
                    "message": msg
                }))
            }
            ServiceError::NotFound(msg) => {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Not found",
                    "message": msg
                }))
            }
            ServiceError::Unauthorized(msg) => {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Unauthorized",
                    "message": msg
                }))
            }
            ServiceError::DatabaseError(msg) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Database error",
                    "message": msg
                }))
            }
            ServiceError::StripeError(msg) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Payment processing error",
                    "message": msg
                }))
            }
            ServiceError::ValidationError(msg) => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Validation error",
                    "message": msg
                }))
            }
            ServiceError::ParseError(msg) => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Parse error",
                    "message": msg
                }))
            }
        }
    }
}

impl From<sqlx::Error> for ServiceError {
    fn from(err: sqlx::Error) -> Self {
        ServiceError::DatabaseError(err.to_string())
    }
}