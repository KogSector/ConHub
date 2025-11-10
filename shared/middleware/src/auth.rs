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
use rsa::{RsaPublicKey, pkcs8::DecodePublicKey};
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
        let public_key = load_public_key()?;
        Ok(Self { mode: AuthMode::Enabled(Arc::new(public_key)) })
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

// Load public key from environment variable or file
fn load_public_key() -> Result<RsaPublicKey, Box<dyn std::error::Error>> {
    // Try to load from environment variable first
    if let Ok(public_key_pem) = std::env::var("JWT_PUBLIC_KEY") {
        let public_key = RsaPublicKey::from_public_key_pem(&public_key_pem)?;
        return Ok(public_key);
    }
    
    // Try to load from file
    if let Ok(public_key_path) = std::env::var("JWT_PUBLIC_KEY_PATH") {
        let public_key_pem = std::fs::read_to_string(public_key_path)?;
        let public_key = RsaPublicKey::from_public_key_pem(&public_key_pem)?;
        return Ok(public_key);
    }
    
    Err("No public key found. Set JWT_PUBLIC_KEY or JWT_PUBLIC_KEY_PATH environment variable".into())
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
