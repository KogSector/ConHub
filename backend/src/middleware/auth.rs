use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse, HttpRequest,
    web::Data, body::EitherBody,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;
use crate::services::auth_service::{AuthService, AuthError};
use crate::services::feature_toggle_service::FeatureToggleService;
use crate::services::session_service::SessionService;
use crate::models::auth::Claims;

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService { service: Rc::new(service) }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
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
        // Skip auth for public routes
        let path = req.path().to_string();
        let is_public_route = path.starts_with("/api/auth/") 
            || path == "/api/health" 
            || path.starts_with("/public/")
            || req.method() == "OPTIONS";

        if is_public_route {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_left_body())
            });
        }

        // Extract services from app data
        let auth_service = req.app_data::<Data<AuthService>>();
        let session_service = req.app_data::<Data<SessionService>>();
        let feature_toggle_service = req.app_data::<Data<FeatureToggleService>>();
        
        if auth_service.is_none() {
            return Box::pin(async move {
                let (req, _payload) = req.into_parts();
                let response = HttpResponse::InternalServerError()
                    .json(serde_json::json!({"error": "Auth service not configured"}));
                Ok(ServiceResponse::new(req, response).map_into_right_body())
            });
        }

        let auth_service = auth_service.unwrap().clone();
        let session_service = session_service.cloned();
        let feature_toggle_service = feature_toggle_service.cloned();
        let service = self.service.clone();

        Box::pin(async move {
            // Check feature toggles first
            if let Some(toggle_service) = feature_toggle_service {
                if toggle_service.should_bypass_auth(&path).await {
                    let res = service.call(req).await?;
                    return Ok(res.map_into_left_body());
                }
            }
            // Try session-based auth first if session service is available
            if let Some(session_service) = session_service {
                // Check for session cookie
                if let Some(cookie_header) = req.headers().get("Cookie") {
                    if let Ok(cookie_str) = cookie_header.to_str() {
                        // Simple cookie parsing - in production use a proper cookie library
                        for cookie in cookie_str.split(';') {
                            let cookie = cookie.trim();
                            if cookie.starts_with("session_id=") {
                                let session_id = &cookie[11..]; // Remove "session_id="
                                
                                // Check if we have a valid session
                                if let Some(session) = session_service.validate_session(session_id).await {
                                    // Session is valid, inject claims and continue
                                    req.extensions_mut().insert(session.to_claims());
                                    let res = service.call(req).await?;
                                    return Ok(res.map_into_left_body());
                                }
                            }
                        }
                    }
                }
            }

            // Fall back to token-based auth
            let auth_header = req.headers().get("Authorization");
            let token = match auth_header {
                Some(header) => {
                    let header_str = header.to_str().unwrap_or("");
                    if header_str.starts_with("Bearer ") {
                        &header_str[7..]
                    } else {
                        ""
                    }
                }
                None => "",
            };

            if token.is_empty() {
                let (req, _payload) = req.into_parts();
                let response = HttpResponse::Unauthorized()
                    .json(serde_json::json!({"error": "Missing or invalid authorization header"}));
                return Ok(ServiceResponse::new(req, response).map_into_right_body());
            }

            // Verify token
            let claims = match auth_service.verify_token(token) {
                Ok(claims) => claims,
                Err(AuthError::TokenExpired) => {
                    let (req, _payload) = req.into_parts();
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({"error": "Token expired"}));
                    return Ok(ServiceResponse::new(req, response).map_into_right_body());
                }
                Err(_) => {
                    let (req, _payload) = req.into_parts();
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({"error": "Invalid token"}));
                    return Ok(ServiceResponse::new(req, response).map_into_right_body());
                }
            };

            // Add claims to request extensions
            req.extensions_mut().insert(claims);

            let res = service.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}

// Helper function to extract claims from request
#[allow(dead_code)]
pub fn extract_claims(req: &ServiceRequest) -> Option<Claims> {
    req.extensions().get::<Claims>().cloned()
}

// Helper function to extract claims from HttpRequest
pub fn extract_claims_from_http_request(req: &HttpRequest) -> Option<Claims> {
    req.extensions().get::<Claims>().cloned()
}

// Helper function to extract user ID from request
#[allow(dead_code)]
pub fn extract_user_id(req: &ServiceRequest) -> Option<uuid::Uuid> {
    extract_claims(req)
        .and_then(|claims| uuid::Uuid::parse_str(&claims.sub).ok())
}

// Helper function to extract user ID from HttpRequest
pub fn extract_user_id_from_http_request(req: &HttpRequest) -> Option<uuid::Uuid> {
    extract_claims_from_http_request(req)
        .and_then(|claims| uuid::Uuid::parse_str(&claims.sub).ok())
}