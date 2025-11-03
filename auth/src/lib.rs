// Authentication service library
//
// This module provides authentication functionality including:
// - User authentication (email/password)
// - JWT token generation and validation
// - OAuth integrations (Google, GitHub, Microsoft)
// - Password hashing and verification
// - Session management

pub mod handlers;
pub mod services;

pub use handlers::*;
pub use services::*;
