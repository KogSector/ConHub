use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, Result,
    http::{Method, header::HeaderMap},
};
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use std::{
    future::{ready, Ready},
    rc::Rc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use log::{info, warn, error, debug};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub log_requests: bool,
    pub log_responses: bool,
    pub log_request_body: bool,
    pub log_response_body: bool,
    pub log_headers: bool,
    pub log_performance: bool,
    pub exclude_paths: Vec<String>,
    pub exclude_methods: Vec<String>,
    pub max_body_size: usize,
    pub sensitive_headers: Vec<String>,
    pub log_level: LogLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            log_requests: true,
            log_responses: true,
            log_request_body: false, // Disabled by default for security
            log_response_body: false, // Disabled by default for performance
            log_headers: true,
            log_performance: true,
            exclude_paths: vec![
                "/health".to_string(),
                "/metrics".to_string(),
                "/favicon.ico".to_string(),
            ],
            exclude_methods: vec!["OPTIONS".to_string()],
            max_body_size: 1024, // 1KB max for body logging
            sensitive_headers: vec![
                "authorization".to_string(),
                "cookie".to_string(),
                "x-api-key".to_string(),
                "x-auth-token".to_string(),
            ],
            log_level: LogLevel::Info,
        }
    }
}

impl LoggingConfig {
    pub fn development() -> Self {
        Self {
            log_request_body: true,
            log_response_body: true,
            log_level: LogLevel::Debug,
            max_body_size: 4096, // 4KB for development
            ..Default::default()
        }
    }

    pub fn production() -> Self {
        Self {
            log_request_body: false,
            log_response_body: false,
            log_headers: false, // More restrictive in production
            log_level: LogLevel::Info,
            max_body_size: 512, // 512B for production
            ..Default::default()
        }
    }

    pub fn minimal() -> Self {
        Self {
            log_requests: true,
            log_responses: false,
            log_request_body: false,
            log_response_body: false,
            log_headers: false,
            log_performance: true,
            log_level: LogLevel::Warn,
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize)]
struct RequestLog {
    request_id: String,
    timestamp: u64,
    method: String,
    path: String,
    query_string: Option<String>,
    user_agent: Option<String>,
    remote_addr: Option<String>,
    headers: Option<serde_json::Value>,
    body: Option<String>,
    content_length: Option<u64>,
}

#[derive(Debug, Serialize)]
struct ResponseLog {
    request_id: String,
    timestamp: u64,
    status_code: u16,
    headers: Option<serde_json::Value>,
    body: Option<String>,
    content_length: Option<u64>,
    duration_ms: u64,
}

#[derive(Debug, Serialize)]
struct PerformanceLog {
    request_id: String,
    method: String,
    path: String,
    status_code: u16,
    duration_ms: u64,
    timestamp: u64,
    user_id: Option<String>,
}

#[derive(Clone)]
pub struct LoggingMiddleware {
    config: LoggingConfig,
}

impl LoggingMiddleware {
    pub fn new(config: LoggingConfig) -> Self {
        Self { config }
    }

    fn should_log_request(&self, req: &ServiceRequest) -> bool {
        let path = req.path();
        let method = req.method().as_str();

        // Check if path is excluded
        if self.config.exclude_paths.iter().any(|p| path.starts_with(p)) {
            return false;
        }

        // Check if method is excluded
        if self.config.exclude_methods.iter().any(|m| m == method) {
            return false;
        }

        true
    }

    fn sanitize_headers(&self, headers: &HeaderMap) -> serde_json::Value {
        let mut sanitized = serde_json::Map::new();

        for (name, value) in headers.iter() {
            let name_str = name.as_str().to_lowercase();
            
            if self.config.sensitive_headers.contains(&name_str) {
                sanitized.insert(name_str, serde_json::Value::String("[REDACTED]".to_string()));
            } else if let Ok(value_str) = value.to_str() {
                sanitized.insert(name_str, serde_json::Value::String(value_str.to_string()));
            }
        }

        serde_json::Value::Object(sanitized)
    }

    fn truncate_body(&self, body: &str) -> String {
        if body.len() > self.config.max_body_size {
            format!("{}... [TRUNCATED]", &body[..self.config.max_body_size])
        } else {
            body.to_string()
        }
    }

    fn get_user_id(&self, req: &ServiceRequest) -> Option<String> {
        // Try to extract user ID from JWT claims or session
        req.extensions().get::<String>().cloned()
    }

    fn log_request(&self, req: &ServiceRequest, request_id: &str) {
        if !self.config.log_requests {
            return;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let headers = if self.config.log_headers {
            Some(self.sanitize_headers(req.headers()))
        } else {
            None
        };

        let request_log = RequestLog {
            request_id: request_id.to_string(),
            timestamp,
            method: req.method().to_string(),
            path: req.path().to_string(),
            query_string: req.query_string().is_empty().then(|| req.query_string().to_string()),
            user_agent: req
                .headers()
                .get("user-agent")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string()),
            remote_addr: req
                .connection_info()
                .realip_remote_addr()
                .map(|s| s.to_string()),
            headers,
            body: None, // Body logging would require reading the payload
            content_length: req
                .headers()
                .get("content-length")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse().ok()),
        };

        match self.config.log_level {
            LogLevel::Debug => debug!("Request: {}", serde_json::to_string(&request_log).unwrap_or_default()),
            LogLevel::Info => info!("Request: {}", serde_json::to_string(&request_log).unwrap_or_default()),
            LogLevel::Warn => warn!("Request: {}", serde_json::to_string(&request_log).unwrap_or_default()),
            LogLevel::Error => error!("Request: {}", serde_json::to_string(&request_log).unwrap_or_default()),
        }
    }

    fn log_response<B>(&self, res: &ServiceResponse<B>, request_id: &str, duration: std::time::Duration) {
        if !self.config.log_responses {
            return;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let headers = if self.config.log_headers {
            Some(self.sanitize_headers(res.headers()))
        } else {
            None
        };

        let response_log = ResponseLog {
            request_id: request_id.to_string(),
            timestamp,
            status_code: res.status().as_u16(),
            headers,
            body: None, // Body logging would require intercepting the response
            content_length: res
                .headers()
                .get("content-length")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse().ok()),
            duration_ms: duration.as_millis() as u64,
        };

        match self.config.log_level {
            LogLevel::Debug => debug!("Response: {}", serde_json::to_string(&response_log).unwrap_or_default()),
            LogLevel::Info => info!("Response: {}", serde_json::to_string(&response_log).unwrap_or_default()),
            LogLevel::Warn => warn!("Response: {}", serde_json::to_string(&response_log).unwrap_or_default()),
            LogLevel::Error => error!("Response: {}", serde_json::to_string(&response_log).unwrap_or_default()),
        }
    }

    fn log_performance<B>(&self, req: &ServiceRequest, res: &ServiceResponse<B>, request_id: &str, duration: std::time::Duration) {
        if !self.config.log_performance {
            return;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let performance_log = PerformanceLog {
            request_id: request_id.to_string(),
            method: req.method().to_string(),
            path: req.path().to_string(),
            status_code: res.status().as_u16(),
            duration_ms: duration.as_millis() as u64,
            timestamp,
            user_id: self.get_user_id(req),
        };

        // Log performance warnings for slow requests
        let duration_ms = duration.as_millis() as u64;
        if duration_ms > 5000 {
            warn!("Slow request detected: {}", serde_json::to_string(&performance_log).unwrap_or_default());
        } else if duration_ms > 1000 {
            warn!("Performance: {}", serde_json::to_string(&performance_log).unwrap_or_default());
        } else {
            info!("Performance: {}", serde_json::to_string(&performance_log).unwrap_or_default());
        }
    }

    fn log_performance_with_info<B>(&self, method: &Method, path: &str, headers: &HeaderMap, res: &ServiceResponse<B>, request_id: &str, duration: std::time::Duration) {
        if !self.config.log_performance {
            return;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let performance_log = PerformanceLog {
            request_id: request_id.to_string(),
            method: method.to_string(),
            path: path.to_string(),
            status_code: res.status().as_u16(),
            duration_ms: duration.as_millis() as u64,
            timestamp,
            user_id: None, // Cannot extract user_id without ServiceRequest
        };

        // Log performance warnings for slow requests
        let duration_ms = duration.as_millis() as u64;
        if duration_ms > 5000 {
            warn!("Slow request detected: {}", serde_json::to_string(&performance_log).unwrap_or_default());
        } else if duration_ms > 1000 {
            warn!("Performance: {}", serde_json::to_string(&performance_log).unwrap_or_default());
        } else {
            info!("Performance: {}", serde_json::to_string(&performance_log).unwrap_or_default());
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for LoggingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = LoggingMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LoggingMiddlewareService {
            service: Rc::new(service),
            middleware: self.clone(),
        }))
    }
}

pub struct LoggingMiddlewareService<S> {
    service: Rc<S>,
    middleware: LoggingMiddleware,
}

impl<S, B> Service<ServiceRequest> for LoggingMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let middleware = self.middleware.clone();
        let service = self.service.clone();

        Box::pin(async move {
            // Generate unique request ID
            let request_id = Uuid::new_v4().to_string();
            
            // Add request ID to extensions for use in other middleware/handlers
            req.extensions_mut().insert(request_id.clone());

            // Check if we should log this request
            if !middleware.should_log_request(&req) {
                return service.call(req).await;
            }

            // Log the request
            middleware.log_request(&req, &request_id);

            // Start timing
            let start_time = Instant::now();

            // Store request info for performance logging
            let method = req.method().clone();
            let path = req.path().to_string();
            let headers = req.headers().clone();

            // Call the service
            let result = service.call(req).await;

            // Calculate duration
            let duration = start_time.elapsed();

            match result {
                Ok(res) => {
                    // Log the response
                    middleware.log_response(&res, &request_id, duration);
                    
                    // Log performance metrics using stored request info
                    middleware.log_performance_with_info(&method, &path, &headers, &res, &request_id, duration);

                    Ok(res)
                }
                Err(e) => {
                    error!("Request {} failed with error: {:?}", request_id, e);
                    Err(e)
                }
            }
        })
    }
}

// Helper functions
pub fn create_logging_middleware() -> LoggingMiddleware {
    LoggingMiddleware::new(LoggingConfig::default())
}

pub fn create_development_logging() -> LoggingMiddleware {
    LoggingMiddleware::new(LoggingConfig::development())
}

pub fn create_production_logging() -> LoggingMiddleware {
    LoggingMiddleware::new(LoggingConfig::production())
}

pub fn create_minimal_logging() -> LoggingMiddleware {
    LoggingMiddleware::new(LoggingConfig::minimal())
}