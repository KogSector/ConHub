//! Tracing initialization for ConHub services.
//!
//! Provides standardized tracing subscriber setup with JSON or pretty formatting.

use std::env;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Configuration for tracing initialization
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Service name for log attribution
    pub service_name: String,
    /// Environment (dev, staging, prod)
    pub environment: String,
    /// Log format: "json" or "pretty"
    pub format: String,
    /// Log level filter (e.g., "info", "debug", "conhub=debug,info")
    pub level: String,
    /// Whether to log span events (enter/exit)
    pub log_spans: bool,
    /// Whether to include file/line in logs
    pub include_location: bool,
    /// Whether to include target (module path)
    pub include_target: bool,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            service_name: "conhub".to_string(),
            environment: env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string()),
            format: env::var("LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string()),
            level: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            log_spans: env::var("LOG_SPANS").map(|v| v == "true").unwrap_or(false),
            include_location: env::var("LOG_LOCATION").map(|v| v == "true").unwrap_or(true),
            include_target: true,
        }
    }
}

impl TracingConfig {
    /// Create config for a specific service
    pub fn for_service(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            ..Default::default()
        }
    }

    /// Set log level
    pub fn with_level(mut self, level: impl Into<String>) -> Self {
        self.level = level.into();
        self
    }

    /// Set format to JSON
    pub fn json(mut self) -> Self {
        self.format = "json".to_string();
        self
    }

    /// Set format to pretty (human-readable)
    pub fn pretty(mut self) -> Self {
        self.format = "pretty".to_string();
        self
    }

    /// Enable span logging
    pub fn with_spans(mut self) -> Self {
        self.log_spans = true;
        self
    }

    /// Set environment
    pub fn with_environment(mut self, env: impl Into<String>) -> Self {
        self.environment = env.into();
        self
    }
}

/// Initialize tracing with the given configuration
///
/// # Example
/// ```ignore
/// use conhub_observability::init_tracing;
///
/// init_tracing(TracingConfig::for_service("data-service"));
/// ```
pub fn init_tracing(config: TracingConfig) {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.level))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let span_events = if config.log_spans {
        FmtSpan::NEW | FmtSpan::CLOSE
    } else {
        FmtSpan::NONE
    };

    if config.format == "json" {
        // JSON format for production
        let layer = fmt::layer()
            .json()
            .with_span_events(span_events)
            .with_current_span(true)
            .with_file(config.include_location)
            .with_line_number(config.include_location)
            .with_target(config.include_target)
            .with_thread_ids(false)
            .with_thread_names(false);

        tracing_subscriber::registry()
            .with(filter)
            .with(layer)
            .init();
    } else {
        // Pretty format for development
        let layer = fmt::layer()
            .pretty()
            .with_span_events(span_events)
            .with_file(config.include_location)
            .with_line_number(config.include_location)
            .with_target(config.include_target);

        tracing_subscriber::registry()
            .with(filter)
            .with(layer)
            .init();
    }

    tracing::info!(
        service = %config.service_name,
        environment = %config.environment,
        format = %config.format,
        "Tracing initialized"
    );
}

/// Quick initialization with defaults for a service
///
/// # Example
/// ```ignore
/// conhub_observability::init_tracing_for("auth-service");
/// ```
pub fn init_tracing_for(service_name: &str) {
    init_tracing(TracingConfig::for_service(service_name));
}

/// Initialize tracing based on environment variables only
pub fn init_tracing_from_env() {
    let service = env::var("SERVICE_NAME").unwrap_or_else(|_| "conhub".to_string());
    init_tracing(TracingConfig::for_service(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = TracingConfig::for_service("test")
            .with_level("debug")
            .json()
            .with_spans();

        assert_eq!(config.service_name, "test");
        assert_eq!(config.level, "debug");
        assert_eq!(config.format, "json");
        assert!(config.log_spans);
    }
}
