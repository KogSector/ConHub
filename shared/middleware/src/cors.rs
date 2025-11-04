use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::{
        header::{
            ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS,
            ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN,
            ACCESS_CONTROL_EXPOSE_HEADERS, ACCESS_CONTROL_MAX_AGE, ORIGIN,
        },
        Method, StatusCode,
    },
    Error, HttpResponse, Result,
    body::EitherBody,
};
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    future::{ready, Ready},
    rc::Rc,
};
use log::{debug, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub exposed_headers: Vec<String>,
    pub allow_credentials: bool,
    pub max_age: Option<u32>,
    pub allow_any_origin: bool,
    pub allow_any_method: bool,
    pub allow_any_header: bool,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec![
                "http://localhost:3000".to_string(),
                "http://localhost:3001".to_string(),
                "http://localhost:8080".to_string(),
                "https://conhub.dev".to_string(),
            ],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "PATCH".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "X-Requested-With".to_string(),
                "Accept".to_string(),
                "Origin".to_string(),
                "X-API-Key".to_string(),
                "X-Client-Version".to_string(),
            ],
            exposed_headers: vec![
                "X-Total-Count".to_string(),
                "X-Page-Count".to_string(),
                "X-Rate-Limit-Remaining".to_string(),
                "X-Rate-Limit-Reset".to_string(),
            ],
            allow_credentials: true,
            max_age: Some(86400), // 24 hours
            allow_any_origin: false,
            allow_any_method: false,
            allow_any_header: false,
        }
    }
}

impl CorsConfig {
    pub fn permissive() -> Self {
        Self {
            allow_any_origin: true,
            allow_any_method: true,
            allow_any_header: true,
            allow_credentials: false, // Cannot be true with allow_any_origin
            ..Default::default()
        }
    }

    pub fn strict() -> Self {
        Self {
            allowed_origins: vec!["https://conhub.dev".to_string()],
            allowed_methods: vec!["GET".to_string(), "POST".to_string()],
            allowed_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
            ],
            exposed_headers: vec![],
            allow_credentials: true,
            max_age: Some(3600), // 1 hour
            allow_any_origin: false,
            allow_any_method: false,
            allow_any_header: false,
        }
    }

    pub fn development() -> Self {
        Self {
            allowed_origins: vec![
                "http://localhost:3000".to_string(),
                "http://localhost:3001".to_string(),
                "http://localhost:8080".to_string(),
                "http://127.0.0.1:3000".to_string(),
                "http://127.0.0.1:3001".to_string(),
                "http://127.0.0.1:8080".to_string(),
            ],
            allow_credentials: true,
            ..Default::default()
        }
    }
}

#[derive(Clone)]
pub struct CorsMiddleware {
    config: CorsConfig,
    allowed_origins_set: HashSet<String>,
    allowed_methods_set: HashSet<String>,
    allowed_headers_set: HashSet<String>,
}

impl CorsMiddleware {
    pub fn new(config: CorsConfig) -> Self {
        let allowed_origins_set = config.allowed_origins.iter().cloned().collect();
        let allowed_methods_set = config.allowed_methods.iter().cloned().collect();
        let allowed_headers_set = config
            .allowed_headers
            .iter()
            .map(|h| h.to_lowercase())
            .collect();

        Self {
            config,
            allowed_origins_set,
            allowed_methods_set,
            allowed_headers_set,
        }
    }

    fn is_origin_allowed(&self, origin: &str) -> bool {
        if self.config.allow_any_origin {
            return true;
        }

        self.allowed_origins_set.contains(origin)
            || self.allowed_origins_set.contains("*")
            || self.check_wildcard_origin(origin)
    }

    fn check_wildcard_origin(&self, origin: &str) -> bool {
        for allowed in &self.config.allowed_origins {
            if allowed.contains('*') {
                let pattern = allowed.replace('*', ".*");
                if let Ok(regex) = regex::Regex::new(&pattern) {
                    if regex.is_match(origin) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn is_method_allowed(&self, method: &str) -> bool {
        if self.config.allow_any_method {
            return true;
        }

        self.allowed_methods_set.contains(method)
            || self.allowed_methods_set.contains("*")
    }

    fn are_headers_allowed(&self, headers: &[String]) -> bool {
        if self.config.allow_any_header {
            return true;
        }

        if self.allowed_headers_set.contains("*") {
            return true;
        }

        headers.iter().all(|header| {
            let header_lower = header.to_lowercase();
            self.allowed_headers_set.contains(&header_lower)
                || self.is_simple_header(&header_lower)
        })
    }

    fn is_simple_header(&self, header: &str) -> bool {
        matches!(
            header,
            "accept"
                | "accept-language"
                | "content-language"
                | "content-type"
                | "range"
        )
    }

    fn build_cors_response(&self, req: &ServiceRequest, origin: Option<&str>) -> HttpResponse {
        let mut response = HttpResponse::Ok();

        // Set Access-Control-Allow-Origin
        if let Some(origin) = origin {
            if self.is_origin_allowed(origin) {
                if self.config.allow_any_origin && !self.config.allow_credentials {
                    response.insert_header((ACCESS_CONTROL_ALLOW_ORIGIN, "*"));
                } else {
                    response.insert_header((ACCESS_CONTROL_ALLOW_ORIGIN, origin));
                }
            }
        } else if self.config.allow_any_origin && !self.config.allow_credentials {
            response.insert_header((ACCESS_CONTROL_ALLOW_ORIGIN, "*"));
        }

        // Set Access-Control-Allow-Credentials
        if self.config.allow_credentials {
            response.insert_header((ACCESS_CONTROL_ALLOW_CREDENTIALS, "true"));
        }

        // Set Access-Control-Allow-Methods
        if self.config.allow_any_method {
            response.insert_header((ACCESS_CONTROL_ALLOW_METHODS, "*"));
        } else {
            let methods = self.config.allowed_methods.join(", ");
            response.insert_header((ACCESS_CONTROL_ALLOW_METHODS, methods));
        }

        // Set Access-Control-Allow-Headers
        if self.config.allow_any_header {
            response.insert_header((ACCESS_CONTROL_ALLOW_HEADERS, "*"));
        } else {
            let headers = self.config.allowed_headers.join(", ");
            response.insert_header((ACCESS_CONTROL_ALLOW_HEADERS, headers));
        }

        // Set Access-Control-Expose-Headers
        if !self.config.exposed_headers.is_empty() {
            let exposed = self.config.exposed_headers.join(", ");
            response.insert_header((ACCESS_CONTROL_EXPOSE_HEADERS, exposed));
        }

        // Set Access-Control-Max-Age
        if let Some(max_age) = self.config.max_age {
            response.insert_header((ACCESS_CONTROL_MAX_AGE, max_age.to_string()));
        }

        response.finish()
    }

    fn handle_preflight(&self, req: &ServiceRequest) -> Result<HttpResponse, Error> {
        let origin = req
            .headers()
            .get(ORIGIN)
            .and_then(|h| h.to_str().ok());

        // Check if origin is allowed
        if let Some(origin) = origin {
            if !self.is_origin_allowed(origin) {
                warn!("CORS preflight rejected: origin not allowed: {}", origin);
                return Ok(HttpResponse::Forbidden().finish());
            }
        }

        // Check requested method
        if let Some(method_header) = req.headers().get("Access-Control-Request-Method") {
            if let Ok(method) = method_header.to_str() {
                if !self.is_method_allowed(method) {
                    warn!("CORS preflight rejected: method not allowed: {}", method);
                    return Ok(HttpResponse::Forbidden().finish());
                }
            }
        }

        // Check requested headers
        if let Some(headers_header) = req.headers().get("Access-Control-Request-Headers") {
            if let Ok(headers_str) = headers_header.to_str() {
                let requested_headers: Vec<String> = headers_str
                    .split(',')
                    .map(|h| h.trim().to_string())
                    .collect();

                if !self.are_headers_allowed(&requested_headers) {
                    warn!("CORS preflight rejected: headers not allowed: {:?}", requested_headers);
                    return Ok(HttpResponse::Forbidden().finish());
                }
            }
        }

        debug!("CORS preflight approved for origin: {:?}", origin);
        Ok(self.build_cors_response(req, origin))
    }
}

impl<S, B> Transform<S, ServiceRequest> for CorsMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = CorsMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CorsMiddlewareService {
            service: Rc::new(service),
            middleware: self.clone(),
        }))
    }
}

pub struct CorsMiddlewareService<S> {
    service: Rc<S>,
    middleware: CorsMiddleware,
}

impl<S, B> Service<ServiceRequest> for CorsMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let middleware = self.middleware.clone();
        let service = self.service.clone();

        Box::pin(async move {
            // Handle preflight requests
            if req.method() == Method::OPTIONS {
                match middleware.handle_preflight(&req) {
                    Ok(response) => return Ok(req.into_response(response).map_into_right_body()),
                    Err(e) => return Err(e),
                }
            }

            // For actual requests, add CORS headers to the response
            let origin = req
                .headers()
                .get(ORIGIN)
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string());

            let fut = service.call(req);
            let mut res = fut.await?;

            // Add CORS headers to the response
            let headers = res.headers_mut();

            if let Some(ref origin) = origin {
                if middleware.is_origin_allowed(origin) {
                    if middleware.config.allow_any_origin && !middleware.config.allow_credentials {
                        headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
                    } else {
                        headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, origin.parse().unwrap());
                    }
                }
            } else if middleware.config.allow_any_origin && !middleware.config.allow_credentials {
                headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
            }

            if middleware.config.allow_credentials {
                headers.insert(ACCESS_CONTROL_ALLOW_CREDENTIALS, "true".parse().unwrap());
            }

            if !middleware.config.exposed_headers.is_empty() {
                let exposed = middleware.config.exposed_headers.join(", ");
                headers.insert(ACCESS_CONTROL_EXPOSE_HEADERS, exposed.parse().unwrap());
            }

            Ok(res.map_into_left_body())
        })
    }
}

// Helper functions
pub fn create_cors_middleware() -> CorsMiddleware {
    CorsMiddleware::new(CorsConfig::default())
}

pub fn create_permissive_cors() -> CorsMiddleware {
    CorsMiddleware::new(CorsConfig::permissive())
}

pub fn create_strict_cors() -> CorsMiddleware {
    CorsMiddleware::new(CorsConfig::strict())
}

pub fn create_development_cors() -> CorsMiddleware {
    CorsMiddleware::new(CorsConfig::development())
}