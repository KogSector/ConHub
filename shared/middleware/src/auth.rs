use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error as ActixError, HttpMessage, HttpResponse, HttpRequest,
    body::EitherBody,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::sync::Arc;
use conhub_models::auth::Claims;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use rsa::{RsaPublicKey, pkcs8::{DecodePublicKey, EncodePublicKey}, pkcs1::DecodeRsaPublicKey};
use base64;
use serde_json::json;
use chrono;
use tracing;

#[derive(Clone)]
enum AuthMode {
    Enabled(Arc<RsaPublicKey>),
    Disabled(Claims),
}

pub struct AuthMiddleware<S> {
    service: Rc<S>,
    mode: AuthMode,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = ActixError;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let mode = self.mode.clone();

        Box::pin(async move {
            // Check if this is a public endpoint that doesn't require authentication
            if is_public_endpoint(req.path()) {
                let res = service.call(req).await?;
                return Ok(res.map_into_left_body());
            }

            match mode {
                AuthMode::Enabled(public_key) => {
                    if let Some(auth_header) = req.headers().get("Authorization") {
                        if let Ok(auth_str) = auth_header.to_str() {
                            if auth_str.starts_with("Bearer ") {
                                let token = &auth_str[7..];
                                match verify_jwt_token(token, &public_key).await {
                                    Ok(claims) => {
                                        req.extensions_mut().insert(claims);
                                        let res = service.call(req).await?;
                                        return Ok(res.map_into_left_body());
                                    }
                                    Err(e) => {
                                        tracing::warn!("JWT verification failed: {}", e);
                                        return Ok(req.into_response(
                                            HttpResponse::Unauthorized()
                                                .json(json!({
                                                    "error": "Invalid or expired token",
                                                    "details": e.to_string()
                                                }))
                                        ).map_into_right_body());
                                    }
                                }
                            }
                        }
                    }
                    Ok(req.into_response(
                        HttpResponse::Unauthorized()
                            .json(json!({
                                "error": "Authentication required",
                                "message": "Please provide a valid Bearer token in the Authorization header"
                            }))
                    ).map_into_right_body())
                }
                AuthMode::Disabled(default_claims) => {
                    // Inject default claims and proceed
                    req.extensions_mut().insert(default_claims.clone());
                    let res = service.call(req).await?;
                    Ok(res.map_into_left_body())
                }
            }
        })
    }
}

#[derive(Clone)]
pub struct AuthMiddlewareFactory {
    mode: AuthMode,
}

impl AuthMiddlewareFactory {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        match load_public_key() {
            Ok(public_key) => Ok(Self { mode: AuthMode::Enabled(Arc::new(public_key)) }),
            Err(e) => {
                tracing::warn!("JWT public key not found or invalid ({}). Falling back to disabled auth (dev claims). Set `JWT_PUBLIC_KEY_PATH` or `JWT_PUBLIC_KEY` to enable.", e);
                Ok(Self::disabled())
            }
        }
    }

    pub fn from_public_key(public_key: RsaPublicKey) -> Self {
        Self { mode: AuthMode::Enabled(Arc::new(public_key)) }
    }

    pub fn disabled() -> Self {
        let claims = conhub_models::auth::default_dev_claims();
        Self { mode: AuthMode::Disabled(claims) }
    }

    pub fn new_with_enabled(enabled: bool) -> Result<Self, Box<dyn std::error::Error>> {
        if enabled {
            Self::new()
        } else {
            Ok(Self::disabled())
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = ActixError;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
            mode: self.mode.clone(),
        }))
    }
}

// JWT verification function using RSA public key
async fn verify_jwt_token(token: &str, public_key: &RsaPublicKey) -> Result<Claims, Box<dyn std::error::Error>> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&["conhub"]);
    validation.set_audience(&["conhub-users"]);
    
    // Convert RSA public key to PEM format for jsonwebtoken
    let public_key_pem = rsa::pkcs8::EncodePublicKey::to_public_key_pem(public_key, rsa::pkcs8::LineEnding::LF)?;
    let decoding_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes())?;
    
    let token_data = decode::<Claims>(token, &decoding_key, &validation)?;
    
    // Additional validation: check if token is not expired
    let now = chrono::Utc::now().timestamp() as usize;
    if token_data.claims.exp < now {
        return Err("Token has expired".into());
    }
    
    Ok(token_data.claims)
}

// Load public key from environment variable or file with robust parsing
fn load_public_key() -> Result<RsaPublicKey, Box<dyn std::error::Error>> {
    // Prefer file path to avoid .env multiline pitfalls
    if let Ok(public_key_path) = std::env::var("JWT_PUBLIC_KEY_PATH") {
        let raw = std::fs::read_to_string(&public_key_path)?;
        match parse_public_key_from_str(&raw) {
            Ok(key) => return Ok(key),
            Err(e) => {
                tracing::error!("Failed parsing JWT_PUBLIC_KEY_PATH ({}): {}", public_key_path, e);
                return Err(e);
            }
        }
    }

    // Fallback to inline env var
    if let Ok(inline) = std::env::var("JWT_PUBLIC_KEY") {
        match parse_public_key_from_str(&inline) {
            Ok(key) => return Ok(key),
            Err(e) => {
                tracing::error!("Failed parsing JWT_PUBLIC_KEY env var: {}", e);
                return Err(e);
            }
        }
    }

    Err("No public key found. Set JWT_PUBLIC_KEY_PATH or JWT_PUBLIC_KEY environment variable".into())
}

// Attempt to parse a public key from various formats:
// - Proper PEM with headers
// - PEM provided via .env with escaped newlines (\n)
// - Raw Base64 DER (PKCS#1 or PKCS#8)
fn parse_public_key_from_str(input: &str) -> Result<RsaPublicKey, Box<dyn std::error::Error>> {
    // Trim and unquote if value wrapped in quotes
    let mut s = input.trim().to_string();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s = s[1..s.len()-1].to_string();
    }
    // Replace literal \n with newline and normalize Windows CRLF
    let s = s.replace("\\n", "\n").replace("\\r", "\r");

    // If it looks like PEM, try PEM first
    if s.contains("-----BEGIN") {
        // Some environments may inadvertently include carriage returns; allow them
        if let Ok(key) = RsaPublicKey::from_public_key_pem(&s) {
            return Ok(key);
        }
        // If we only received the header line, this is likely a multiline .env issue
        if s.trim().lines().count() == 1 {
            return Err("PEM appears truncated (only header). Use JWT_PUBLIC_KEY_PATH or escape newlines (\\n).".into());
        }
    }

    // Otherwise, try raw Base64 DER (strip whitespace)
    let b64 = s.lines().collect::<Vec<_>>().join("");
    let b64 = b64.replace(' ', "");
    // Ignore empty or obviously placeholder values
    if b64.is_empty() || b64.contains("YOUR_PUBLIC_KEY_HERE") {
        return Err("Public key value is empty or placeholder".into());
    }

    let bytes = base64::decode(b64.as_bytes())?;

    // Try PKCS#8 DER first
    if let Ok(key) = RsaPublicKey::from_public_key_der(&bytes) {
        return Ok(key);
    }
    // Fallback to PKCS#1 DER
    if let Ok(key) = RsaPublicKey::from_pkcs1_der(&bytes) {
        return Ok(key);
    }

    Err("Unsupported public key format: expected PEM or Base64 DER".into())
}

// Check if the endpoint is public and doesn't require authentication
fn is_public_endpoint(path: &str) -> bool {
    let public_paths = [
        "/health",
        "/metrics",
        "/auth/login",
        "/auth/register",
        "/auth/forgot-password",
        "/auth/reset-password",
        "/auth/verify-email",
        "/auth/oauth",
        "/docs",
        "/swagger",
    ];
    
    public_paths.iter().any(|&public_path| path.starts_with(public_path))
}

// Helper functions for extracting information from requests
pub fn extract_token_from_request(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(|s| s.to_string())
}

pub fn extract_claims_from_request(req: &HttpRequest) -> Option<Claims> {
    req.extensions().get::<Claims>().cloned()
}

pub fn extract_session_id_from_request(req: &HttpRequest) -> Option<uuid::Uuid> {
    extract_claims_from_request(req)?
        .session_id
        .parse()
        .ok()
}

pub fn extract_user_id_from_request(req: &HttpRequest) -> Option<uuid::Uuid> {
    extract_claims_from_request(req)?
        .sub
        .parse()
        .ok()
}

pub fn extract_user_id_from_http_request(req: &HttpRequest) -> Option<uuid::Uuid> {
    extract_user_id_from_request(req)
}

pub fn extract_claims_from_http_request(req: &HttpRequest) -> Option<Claims> {
    extract_claims_from_request(req)
}

// Optional middleware for role-based authorization
pub struct RoleAuthMiddleware<S> {
    service: Rc<S>,
    required_roles: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for RoleAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = ActixError;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let required_roles = self.required_roles.clone();

        Box::pin(async move {
            // Check if claims exist and validate roles before proceeding
            let has_permission = if let Some(claims) = req.extensions().get::<Claims>() {
                required_roles.is_empty() || 
                required_roles.iter().any(|role| claims.roles.contains(role))
            } else {
                false
            };

            if has_permission {
                let res = service.call(req).await?;
                Ok(res.map_into_left_body())
            } else {
                Ok(req.into_response(
                    HttpResponse::Forbidden()
                        .json(json!({
                            "error": "Insufficient permissions",
                            "required_roles": required_roles
                        }))
                ).map_into_right_body())
            }
        })
    }
}

pub struct RoleAuthMiddlewareFactory {
    required_roles: Vec<String>,
}

impl RoleAuthMiddlewareFactory {
    pub fn new(required_roles: Vec<String>) -> Self {
        Self { required_roles }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RoleAuthMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = ActixError;
    type Transform = RoleAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RoleAuthMiddleware {
            service: Rc::new(service),
            required_roles: self.required_roles.clone(),
        }))
    }
}

// default_dev_claims is provided by conhub_models::auth
