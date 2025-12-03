//! ConHub Observability Library
//!
//! Provides unified logging, tracing, and metrics infrastructure for all ConHub microservices.
//!
//! # Features
//! - Structured JSON logging with consistent schema
//! - Distributed trace ID propagation across services
//! - Domain event logging macros
//! - HTTP middleware for request/response logging
//! - Performance tracking and slow request detection

pub mod trace_context;
pub mod domain_events;
pub mod middleware;
pub mod init;
pub mod macros;

pub use trace_context::*;
pub use domain_events::*;
pub use middleware::*;
pub use init::*;

// Re-export tracing for convenience
pub use tracing::{debug, error, info, warn, trace, span, Level, Instrument};
pub use tracing::instrument;
