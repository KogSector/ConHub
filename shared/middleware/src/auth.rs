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
use jsonwebtoken::{decode, DecodingKey, Validation};

pub struct AuthMiddleware<S> {
    service: Rc<S>,
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

        Box::pin(async move {
            if let Some(auth_header) = req.headers().get("Authorization") {
                if let Ok(auth_str) = auth_header.to_str() {
                    if auth_str.starts_with("Bearer ") {
                        let token = &auth_str[7..];
                        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
                        
                        if let Ok(token_data) = decode::<Claims>(
                            token,
                            &DecodingKey::from_secret(secret.as_ref()),
                            &Validation::default(),
                        ) {
                            req.extensions_mut().insert(token_data.claims);
                            let res = service.call(req).await?;
                            return Ok(res.map_into_left_body());
                        }
                    }
                }
            }

            Ok(req.into_response(
                HttpResponse::Unauthorized()
                    .json(serde_json::json!({
                        "error": "Authentication required"
                    }))
            ).map_into_right_body())
        })
    }
}

pub struct AuthMiddlewareFactory;

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
        }))
    }
}


#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn extract_session_id_from_request(req: &HttpRequest) -> Option<uuid::Uuid> {
    extract_claims_from_request(req)?
        .session_id
        .parse()
        .ok()
}

pub fn extract_user_id_from_http_request(req: &HttpRequest) -> Option<uuid::Uuid> {
    extract_claims_from_request(req)?
        .sub
        .parse()
        .ok()
}

pub fn extract_claims_from_http_request(req: &HttpRequest) -> Option<Claims> {
    extract_claims_from_request(req)
}
