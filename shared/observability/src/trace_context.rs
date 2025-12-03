//! Trace context propagation for distributed tracing across ConHub services.
//!
//! Supports W3C Trace Context format and custom X-Request-ID headers.

use actix_web::{HttpRequest, HttpMessage};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Header names for trace context propagation
pub const TRACE_ID_HEADER: &str = "x-trace-id";
pub const SPAN_ID_HEADER: &str = "x-span-id";
pub const PARENT_SPAN_ID_HEADER: &str = "x-parent-span-id";
pub const REQUEST_ID_HEADER: &str = "x-request-id";
pub const W3C_TRACEPARENT_HEADER: &str = "traceparent";

/// Trace context containing IDs for distributed tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    /// Unique ID for the entire trace (spans multiple services)
    pub trace_id: String,
    /// ID of the current span
    pub span_id: String,
    /// ID of the parent span (if any)
    pub parent_span_id: Option<String>,
    /// Legacy request ID (backwards compatibility)
    pub request_id: String,
    /// Service that created this context
    pub origin_service: Option<String>,
    /// Tenant ID for multi-tenant isolation
    pub tenant_id: Option<Uuid>,
    /// User ID if authenticated
    pub user_id: Option<Uuid>,
}

impl TraceContext {
    /// Create a new trace context with fresh IDs
    pub fn new() -> Self {
        let trace_id = Uuid::new_v4().to_string();
        let span_id = generate_span_id();
        Self {
            trace_id: trace_id.clone(),
            span_id,
            parent_span_id: None,
            request_id: trace_id,
            origin_service: None,
            tenant_id: None,
            user_id: None,
        }
    }

    /// Create a child context (new span within same trace)
    pub fn child(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: generate_span_id(),
            parent_span_id: Some(self.span_id.clone()),
            request_id: self.request_id.clone(),
            origin_service: self.origin_service.clone(),
            tenant_id: self.tenant_id,
            user_id: self.user_id,
        }
    }

    /// Set the origin service name
    pub fn with_service(mut self, service: impl Into<String>) -> Self {
        self.origin_service = Some(service.into());
        self
    }

    /// Set tenant ID
    pub fn with_tenant(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    /// Set user ID
    pub fn with_user(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Extract trace context from HTTP request headers
    pub fn from_request(req: &HttpRequest) -> Self {
        let headers = req.headers();

        // Try W3C traceparent first
        if let Some(traceparent) = headers.get(W3C_TRACEPARENT_HEADER) {
            if let Ok(tp) = traceparent.to_str() {
                if let Some(ctx) = Self::parse_traceparent(tp) {
                    return ctx;
                }
            }
        }

        // Fall back to custom headers
        let trace_id = headers
            .get(TRACE_ID_HEADER)
            .and_then(|h| h.to_str().ok())
            .map(String::from)
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let request_id = headers
            .get(REQUEST_ID_HEADER)
            .and_then(|h| h.to_str().ok())
            .map(String::from)
            .unwrap_or_else(|| trace_id.clone());

        let parent_span_id = headers
            .get(SPAN_ID_HEADER)
            .and_then(|h| h.to_str().ok())
            .map(String::from);

        Self {
            trace_id,
            span_id: generate_span_id(),
            parent_span_id,
            request_id,
            origin_service: None,
            tenant_id: None,
            user_id: None,
        }
    }

    /// Parse W3C traceparent header
    /// Format: version-trace_id-parent_id-flags (e.g., "00-xxx-yyy-01")
    fn parse_traceparent(value: &str) -> Option<Self> {
        let parts: Vec<&str> = value.split('-').collect();
        if parts.len() >= 3 {
            let trace_id = parts[1].to_string();
            let parent_span_id = parts[2].to_string();
            Some(Self {
                trace_id: trace_id.clone(),
                span_id: generate_span_id(),
                parent_span_id: Some(parent_span_id),
                request_id: trace_id,
                origin_service: None,
                tenant_id: None,
                user_id: None,
            })
        } else {
            None
        }
    }

    /// Generate headers for outgoing HTTP requests
    pub fn to_headers(&self) -> Vec<(String, String)> {
        let mut headers = vec![
            (TRACE_ID_HEADER.to_string(), self.trace_id.clone()),
            (SPAN_ID_HEADER.to_string(), self.span_id.clone()),
            (REQUEST_ID_HEADER.to_string(), self.request_id.clone()),
        ];

        if let Some(ref parent) = self.parent_span_id {
            headers.push((PARENT_SPAN_ID_HEADER.to_string(), parent.clone()));
        }

        // Also add W3C traceparent for compatibility
        let traceparent = format!("00-{}-{}-01", self.trace_id, self.span_id);
        headers.push((W3C_TRACEPARENT_HEADER.to_string(), traceparent));

        headers
    }

    /// Get an actix-web compatible header map
    pub fn to_actix_headers(&self) -> actix_web::http::header::HeaderMap {
        let mut headers = actix_web::http::header::HeaderMap::new();
        for (name, value) in self.to_headers() {
            if let (Ok(name), Ok(value)) = (
                actix_web::http::header::HeaderName::try_from(name),
                actix_web::http::header::HeaderValue::from_str(&value),
            ) {
                headers.insert(name, value);
            }
        }
        headers
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TraceContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "trace_id={} span_id={}", self.trace_id, self.span_id)
    }
}

/// Generate a 16-character span ID
fn generate_span_id() -> String {
    Uuid::new_v4().to_string()[..16].to_string()
}

/// Extension trait to extract TraceContext from actix-web requests
pub trait TraceContextExt {
    fn trace_context(&self) -> TraceContext;
}

impl TraceContextExt for HttpRequest {
    fn trace_context(&self) -> TraceContext {
        // First check if already stored in extensions
        if let Some(ctx) = self.extensions().get::<TraceContext>() {
            return ctx.clone();
        }
        // Otherwise extract from headers
        TraceContext::from_request(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_trace_context() {
        let ctx = TraceContext::new();
        assert!(!ctx.trace_id.is_empty());
        assert!(!ctx.span_id.is_empty());
        assert!(ctx.parent_span_id.is_none());
    }

    #[test]
    fn test_child_context() {
        let parent = TraceContext::new();
        let child = parent.child();
        
        assert_eq!(parent.trace_id, child.trace_id);
        assert_ne!(parent.span_id, child.span_id);
        assert_eq!(child.parent_span_id, Some(parent.span_id));
    }

    #[test]
    fn test_parse_traceparent() {
        let ctx = TraceContext::parse_traceparent("00-abc123-def456-01").unwrap();
        assert_eq!(ctx.trace_id, "abc123");
        assert_eq!(ctx.parent_span_id, Some("def456".to_string()));
    }
}
