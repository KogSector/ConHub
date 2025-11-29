use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error as ActixError, HttpMessage, HttpResponse,
    body::EitherBody,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;
use conhub_models::auth::{Claims, UserRole};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde_json::json;
use chrono::Utc;
use tracing;

pub struct AuthMiddleware<S> {
    service: Rc<S>,
    jwt_secret: String,
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
        let jwt_secret = self.jwt_secret.clone();

        Box::pin(async move {
            // Check if this is a public endpoint that doesn't require authentication
            if is_public_endpoint(req.path()) {
                let res = service.call(req).await?;
                return Ok(res.map_into_left_body());
            }

            if let Some(auth_header) = req.headers().get("Authorization") {
                if let Ok(auth_str) = auth_header.to_str() {
                    if auth_str.starts_with("Bearer ") {
                        let token = &auth_str[7..];
                        
                        match verify_jwt_token(token, &jwt_secret).await {
                            Ok(claims) => {
                                // Insert claims into request extensions for handlers to use
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
        })
    }
}

#[derive(Clone)]
pub struct AuthMiddlewareFactory {
    jwt_secret: String,
}

impl AuthMiddlewareFactory {
    pub fn new() -> Self {
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "conhub_super_secret_jwt_key_2024_development_only".to_string());
        
        Self { jwt_secret }
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
            jwt_secret: self.jwt_secret.clone(),
        }))
    }
}

// JWT verification function using HMAC
async fn verify_jwt_token(token: &str, jwt_secret: &str) -> Result<Claims, Box<dyn std::error::Error>> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&["conhub"]);
    validation.set_audience(&["conhub-frontend"]);
    
    let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());
    
    let token_data = decode::<Claims>(token, &decoding_key, &validation)?;
    
    // Additional validation: check if token is not expired
    let now = Utc::now().timestamp() as usize;
    if token_data.claims.exp < now {
        return Err("Token has expired".into());
    }
    
    Ok(token_data.claims)
}

// Check if the endpoint is public and doesn't require authentication
fn is_public_endpoint(path: &str) -> bool {
    let public_paths = [
        "/health",
        "/metrics",
        "/api/auth/login",
        "/api/auth/register",
        "/api/auth/forgot-password",
        "/api/auth/reset-password",
        "/api/auth/verify-email",
        "/api/auth/oauth",
        "/docs",
        "/swagger",
    ];
    
    public_paths.iter().any(|&public_path| path.starts_with(public_path))
}

// Role-based authorization middleware
pub struct RoleAuthMiddleware<S> {
    service: Rc<S>,
    required_roles: Vec<String>,
}

impl<S> RoleAuthMiddleware<S> {
    // This method is not needed since we use the factory pattern
}

// Helper function to create role auth middleware
pub fn role_auth_middleware(required_roles: Vec<UserRole>) -> RoleAuthMiddlewareFactory {
    let role_strings = required_roles.into_iter().map(|r| match r {
        UserRole::Admin => "admin".to_string(),
        UserRole::User => "user".to_string(),
    }).collect();
    RoleAuthMiddlewareFactory::new(role_strings)
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
            // Extract claims data before using req
            let has_permission = if let Some(claims) = req.extensions().get::<Claims>() {
                let user_roles = &claims.roles;
                required_roles.iter().any(|role| user_roles.contains(role)) || 
                user_roles.contains(&"admin".to_string())
            } else {
                false
            };
            
            if has_permission {
                let res = service.call(req).await?;
                return Ok(res.map_into_left_body());
            }

            Ok(req.into_response(
                HttpResponse::Forbidden()
                    .json(json!({
                        "error": "Insufficient permissions",
                        "message": format!("This endpoint requires one of the following roles: {}", required_roles.join(", "))
                    }))
            ).map_into_right_body())
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

// Helper functions for extracting information from requests
pub fn extract_claims_from_request(req: &actix_web::HttpRequest) -> Option<Claims> {
    req.extensions().get::<Claims>().cloned()
}

/// Extract user_id from request - tries UUID parse first, then falls back to Auth0 sub lookup
/// For Auth0 tokens, the `sub` is like "auth0|123..." which isn't a UUID
pub fn extract_user_id_from_request(req: &actix_web::HttpRequest) -> Option<uuid::Uuid> {
    let claims = extract_claims_from_request(req)?;
    
    // First try direct UUID parse (for legacy ConHub tokens)
    if let Ok(uuid) = claims.sub.parse() {
        return Some(uuid);
    }
    
    // Auth0 sub format - can't resolve synchronously, return None
    // Handlers should use extract_user_id_from_request_async instead
    None
}

/// Async version that resolves Auth0 sub to local user_id via database lookup
pub async fn extract_user_id_from_request_async(
    req: &actix_web::HttpRequest,
    pool: &sqlx::PgPool,
) -> Option<uuid::Uuid> {
    let claims = extract_claims_from_request(req)?;
    
    // First try direct UUID parse (for legacy ConHub tokens)
    if let Ok(uuid) = claims.sub.parse() {
        return Some(uuid);
    }
    
    // Auth0 sub format - look up in database
    crate::services::users::get_user_id_from_auth0_sub(pool, &claims.sub)
        .await
        .ok()
}

/// Get Auth0 sub directly from claims (useful for user provisioning)
pub fn extract_auth0_sub_from_request(req: &actix_web::HttpRequest) -> Option<String> {
    extract_claims_from_request(req).map(|c| c.sub)
}

pub fn extract_session_id_from_request(req: &actix_web::HttpRequest) -> Option<uuid::Uuid> {
    extract_claims_from_request(req)?
        .session_id
        .parse()
        .ok()
}