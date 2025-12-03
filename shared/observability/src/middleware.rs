//! HTTP middleware for request/response logging with trace context.
//!
//! Provides actix-web middleware that:
//! - Extracts or generates trace context from headers
//! - Logs requests and responses with structured fields
//! - Tracks request duration
//! - Propagates trace context to downstream services

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::HeaderMap,
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use serde::Serialize;
use std::{
    future::{ready, Ready},
    rc::Rc,
    time::Instant,
};
use tracing::{info, warn, error, debug, span, Level, Instrument};

use crate::trace_context::TraceContext;

/// Configuration for observability middleware
#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    /// Service name for log attribution
    pub service_name: String,
    /// Whether to log request bodies (careful with sensitive data)
    pub log_request_body: bool,
    /// Whether to log response bodies
    pub log_response_body: bool,
    /// Whether to log headers
    pub log_headers: bool,
    /// Paths to exclude from logging (e.g., /health, /metrics)
    pub exclude_paths: Vec<String>,
    /// Threshold in ms for slow request warnings
    pub slow_request_threshold_ms: u64,
    /// Headers to redact from logs
    pub sensitive_headers: Vec<String>,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            service_name: "conhub".to_string(),
            log_request_body: false,
            log_response_body: false,
            log_headers: false,
            exclude_paths: vec![
                "/health".to_string(),
                "/metrics".to_string(),
                "/favicon.ico".to_string(),
                "/_next".to_string(),
            ],
            slow_request_threshold_ms: 1000,
            sensitive_headers: vec![
                "authorization".to_string(),
                "cookie".to_string(),
                "x-api-key".to_string(),
                "x-auth-token".to_string(),
            ],
        }
    }
}

impl ObservabilityConfig {
    pub fn for_service(name: impl Into<String>) -> Self {
        Self {
            service_name: name.into(),
            ..Default::default()
        }
    }

    pub fn with_slow_threshold(mut self, ms: u64) -> Self {
        self.slow_request_threshold_ms = ms;
        self
    }

    pub fn with_headers(mut self) -> Self {
        self.log_headers = true;
        self
    }

    pub fn exclude_path(mut self, path: impl Into<String>) -> Self {
        self.exclude_paths.push(path.into());
        self
    }
}

/// Observability middleware for actix-web
#[derive(Clone)]
pub struct ObservabilityMiddleware {
    config: ObservabilityConfig,
}

impl ObservabilityMiddleware {
    pub fn new(config: ObservabilityConfig) -> Self {
        Self { config }
    }

    pub fn for_service(name: impl Into<String>) -> Self {
        Self::new(ObservabilityConfig::for_service(name))
    }
}

impl<S, B> Transform<S, ServiceRequest> for ObservabilityMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ObservabilityMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ObservabilityMiddlewareService {
            service: Rc::new(service),
            config: self.config.clone(),
        }))
    }
}

pub struct ObservabilityMiddlewareService<S> {
    service: Rc<S>,
    config: ObservabilityConfig,
}

/// Structured log entry for HTTP requests
#[derive(Debug, Serialize)]
struct HttpRequestLog {
    event: &'static str,
    service: String,
    trace_id: String,
    span_id: String,
    request_id: String,
    method: String,
    path: String,
    query: Option<String>,
    user_agent: Option<String>,
    remote_ip: Option<String>,
    user_id: Option<String>,
    tenant_id: Option<String>,
}

/// Structured log entry for HTTP responses
#[derive(Debug, Serialize)]
struct HttpResponseLog {
    event: &'static str,
    service: String,
    trace_id: String,
    span_id: String,
    request_id: String,
    method: String,
    path: String,
    status_code: u16,
    duration_ms: u64,
    user_id: Option<String>,
    tenant_id: Option<String>,
}

impl<S, B> Service<ServiceRequest> for ObservabilityMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let config = self.config.clone();
        let service = self.service.clone();

        Box::pin(async move {
            let path = req.path().to_string();
            let method = req.method().to_string();

            // Check if path is excluded
            if config.exclude_paths.iter().any(|p| path.starts_with(p)) {
                return service.call(req).await;
            }

            // Extract or create trace context
            let trace_ctx = TraceContext::from_request(req.request())
                .with_service(&config.service_name);

            // Store trace context in request extensions
            req.extensions_mut().insert(trace_ctx.clone());

            // Extract user/tenant info if available (from auth middleware)
            let user_id = req.extensions()
                .get::<String>()
                .cloned();

            let query = if req.query_string().is_empty() {
                None
            } else {
                Some(req.query_string().to_string())
            };

            let user_agent = req.headers()
                .get("user-agent")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string());

            let remote_ip = req.connection_info()
                .realip_remote_addr()
                .map(|s| s.to_string());

            // Log request
            let request_log = HttpRequestLog {
                event: "http_request",
                service: config.service_name.clone(),
                trace_id: trace_ctx.trace_id.clone(),
                span_id: trace_ctx.span_id.clone(),
                request_id: trace_ctx.request_id.clone(),
                method: method.clone(),
                path: path.clone(),
                query,
                user_agent,
                remote_ip,
                user_id: user_id.clone(),
                tenant_id: trace_ctx.tenant_id.map(|t| t.to_string()),
            };

            debug!(
                trace_id = %trace_ctx.trace_id,
                span_id = %trace_ctx.span_id,
                method = %method,
                path = %path,
                "→ {}", serde_json::to_string(&request_log).unwrap_or_default()
            );

            // Create a span for this request
            let request_span = span!(
                Level::INFO,
                "http_request",
                trace_id = %trace_ctx.trace_id,
                span_id = %trace_ctx.span_id,
                method = %method,
                path = %path,
                service = %config.service_name,
            );

            let start = Instant::now();

            // Call the service within the span
            let result = service.call(req).instrument(request_span).await;

            let duration_ms = start.elapsed().as_millis() as u64;

            match result {
                Ok(res) => {
                    let status_code = res.status().as_u16();

                    let response_log = HttpResponseLog {
                        event: "http_response",
                        service: config.service_name.clone(),
                        trace_id: trace_ctx.trace_id.clone(),
                        span_id: trace_ctx.span_id.clone(),
                        request_id: trace_ctx.request_id.clone(),
                        method: method.clone(),
                        path: path.clone(),
                        status_code,
                        duration_ms,
                        user_id,
                        tenant_id: trace_ctx.tenant_id.map(|t| t.to_string()),
                    };

                    // Log at appropriate level based on status and duration
                    if status_code >= 500 {
                        error!(
                            trace_id = %trace_ctx.trace_id,
                            status = status_code,
                            duration_ms = duration_ms,
                            "← {} {} {} {}ms",
                            method, path, status_code, duration_ms
                        );
                    } else if status_code >= 400 {
                        warn!(
                            trace_id = %trace_ctx.trace_id,
                            status = status_code,
                            duration_ms = duration_ms,
                            "← {} {} {} {}ms",
                            method, path, status_code, duration_ms
                        );
                    } else if duration_ms > config.slow_request_threshold_ms {
                        warn!(
                            trace_id = %trace_ctx.trace_id,
                            status = status_code,
                            duration_ms = duration_ms,
                            "← SLOW {} {} {} {}ms",
                            method, path, status_code, duration_ms
                        );
                    } else {
                        info!(
                            trace_id = %trace_ctx.trace_id,
                            status = status_code,
                            duration_ms = duration_ms,
                            "← {} {} {} {}ms",
                            method, path, status_code, duration_ms
                        );
                    }

                    Ok(res)
                }
                Err(e) => {
                    error!(
                        trace_id = %trace_ctx.trace_id,
                        duration_ms = duration_ms,
                        error = %e,
                        "← {} {} ERROR {}ms: {}",
                        method, path, duration_ms, e
                    );
                    Err(e)
                }
            }
        })
    }
}

/// Helper to create observability middleware for a service
pub fn observability(service_name: impl Into<String>) -> ObservabilityMiddleware {
    ObservabilityMiddleware::for_service(service_name)
}

/// Extract trace context from a request for use in handlers
pub fn get_trace_context(req: &actix_web::HttpRequest) -> TraceContext {
    req.extensions()
        .get::<TraceContext>()
        .cloned()
        .unwrap_or_else(TraceContext::new)
}
