use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    body::EitherBody,
    Error as ActixError, HttpMessage, HttpRequest, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::sync::Arc;
use conhub_models::auth::Claims;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde_json::json;
use chrono;
use tracing;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use reqwest::Client;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Clone)]
struct Auth0Config {
    domain: String,
    issuer: String,
    audience: String,
    jwks_uri: String,
    leeway_secs: u64,
}

impl Auth0Config {
    fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let domain = std::env::var("AUTH0_DOMAIN")?;
        let issuer = std::env::var("AUTH0_ISSUER")?;
        let audience = std::env::var("AUTH0_AUDIENCE")?;
        let jwks_uri = std::env::var("AUTH0_JWKS_URI")
            .unwrap_or_else(|_| format!("https://{}/.well-known/jwks.json", domain));
        Ok(Self {
            domain,
            issuer,
            audience,
            jwks_uri,
            leeway_secs: 60,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Jwk {
    kid: String,
    kty: String,
    n: String,
    e: String,
    alg: Option<String>,
    #[serde(rename = "use")]
    use_: Option<String>,

    #[serde(flatten)]
    extra: std::collections::HashMap<String, JsonValue>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

struct Auth0JwksCache {
    client: Client,
    config: Auth0Config,
    cached: Option<(Jwks, Instant)>,
    ttl: Duration,
}

impl Auth0JwksCache {
    fn new(config: Auth0Config) -> Self {
        Self {
            client: Client::new(),
            config,
            cached: None,
            ttl: Duration::from_secs(600),
        }
    }

    async fn get_jwks(&mut self) -> Result<Jwks, Box<dyn std::error::Error>> {
        if let Some((jwks, ts)) = &self.cached {
            if ts.elapsed() < self.ttl {
                return Ok(jwks.clone());
            }
        }
        let resp = self.client.get(&self.config.jwks_uri).send().await?;
        if !resp.status().is_success() {
            return Err(format!("Failed to fetch JWKS: {}", resp.status()).into());
        }
        let jwks: Jwks = resp.json().await?;
        self.cached = Some((jwks.clone(), Instant::now()));
        Ok(jwks)
    }

    async fn get_key(&mut self, kid: &str) -> Result<Jwk, Box<dyn std::error::Error>> {
        let jwks = self.get_jwks().await?;
        jwks
            .keys
            .into_iter()
            .find(|k| k.kid == kid)
            .ok_or_else(|| "JWK not found for kid".into())
    }
}

#[derive(Debug, Deserialize)]
struct RawAuth0Claims {
    sub: String,
    iss: String,
    aud: JsonValue,
    exp: usize,
    iat: Option<usize>,
    email: Option<String>,
    scope: Option<String>,
    permissions: Option<Vec<String>>, 
    #[serde(flatten)]
    extra: std::collections::HashMap<String, JsonValue>,
}

#[derive(Clone)]
struct Auth0Verifier {
    config: Auth0Config,
    jwks_cache: Arc<tokio::sync::Mutex<Auth0JwksCache>>,
}

#[derive(Clone)]
enum AuthMode {
    Enabled(Arc<Auth0Verifier>),
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
                AuthMode::Enabled(verifier) => {
                    if let Some(auth_header) = req.headers().get("Authorization") {
                        if let Ok(auth_str) = auth_header.to_str() {
                            if auth_str.starts_with("Bearer ") {
                                let token = &auth_str[7..];
                                match verify_auth0_jwt_token(token, &verifier).await {
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
        let config = Auth0Config::from_env()?;
        let cache = Auth0JwksCache::new(config.clone());
        let verifier = Auth0Verifier {
            config,
            jwks_cache: Arc::new(tokio::sync::Mutex::new(cache)),
        };
        Ok(Self { mode: AuthMode::Enabled(Arc::new(verifier)) })
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

async fn verify_auth0_jwt_token(
    token: &str,
    verifier: &Auth0Verifier,
) -> Result<conhub_models::auth::Claims, Box<dyn std::error::Error>> {
    let header = decode_header(token)?;
    let kid = header.kid.ok_or("Missing kid in JWT header")?;

    let mut cache = verifier.jwks_cache.lock().await;
    let jwk = cache.get_key(&kid).await?;

    if jwk.kty != "RSA" {
        return Err("Unsupported JWK kty".into());
    }

    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.iss = Some(std::collections::HashSet::from([verifier.config.issuer.clone()]));
    // Audience can be string or array in Auth0; validate manually
    validation.validate_aud = false;

    let token_data = decode::<RawAuth0Claims>(token, &decoding_key, &validation)?;
    let claims = token_data.claims;

    let now = chrono::Utc::now().timestamp() as usize;
    if claims.exp + (verifier.config.leeway_secs as usize) < now {
        return Err("Token expired".into());
    }

    let aud_ok = match &claims.aud {
        JsonValue::String(aud) => aud == &verifier.config.audience,
        JsonValue::Array(arr) => arr.iter().any(|v| v == &JsonValue::String(verifier.config.audience.clone())),
        _ => false,
    };
    if !aud_ok {
        return Err("Invalid audience".into());
    }

    if let Ok(required_scope) = std::env::var("AUTH0_REQUIRED_SCOPE") {
        if let Some(scope_str) = &claims.scope {
            let scopes: std::collections::HashSet<_> =
                scope_str.split_whitespace().map(|s| s.to_string()).collect();
            if !scopes.contains(&required_scope) {
                return Err("Missing required scope".into());
            }
        } else {
            return Err("Missing scope claim".into());
        }
    }

    let now = chrono::Utc::now().timestamp() as usize;
    let iat = claims.iat.unwrap_or(now);
    let email = claims.email.unwrap_or_default();

    let mut roles: Vec<String> = Vec::new();
    if let Some(perms) = claims.permissions.clone() {
        for p in perms {
            if p.starts_with("admin") && !roles.contains(&"admin".to_string()) {
                roles.push("admin".to_string());
            }
        }
    }
    if roles.is_empty() {
        roles.push("user".to_string());
    }

    let session_id = Uuid::new_v4().to_string();
    let jti = Uuid::new_v4().to_string();

    let internal_claims = conhub_models::auth::Claims {
        sub: claims.sub.clone(),
        email,
        roles,
        exp: claims.exp,
        iat,
        iss: claims.iss.clone(),
        aud: verifier.config.audience.clone(),
        session_id,
        jti,
    };

    Ok(internal_claims)
}

// Load public key from environment variable or file with robust parsing
// RSA key loading and parsing removed; Auth0 JWKS is now the source of truth

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{web, App, HttpResponse, HttpServer};
    use jsonwebtoken::{encode, Header, Algorithm, EncodingKey};
    use rsa::{RsaPrivateKey, RsaPublicKey};
    use rsa::pkcs8::EncodePrivateKey;
    use rsa::traits::PublicKeyParts;
    use std::net::TcpListener;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;

    fn make_jwk(public_key: &RsaPublicKey, kid: &str) -> Jwk {
        let n_bytes = public_key.n().to_bytes_be();
        let e_bytes = public_key.e().to_bytes_be();
        Jwk {
            kid: kid.to_string(),
            kty: "RSA".to_string(),
            n: URL_SAFE_NO_PAD.encode(n_bytes),
            e: URL_SAFE_NO_PAD.encode(e_bytes),
            alg: Some("RS256".to_string()),
            use_: Some("sig".to_string()),
            extra: Default::default(),
        }
    }

    async fn start_jwks_server(jwks: Jwks) -> (String, actix_web::dev::Server) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind");
        let addr = listener.local_addr().unwrap();
        let jwks_data = web::Data::new(jwks);
        let server = HttpServer::new(move || {
            App::new()
                .app_data(jwks_data.clone())
                .route("/.well-known/jwks.json", web::get().to(|data: web::Data<Jwks>| async move {
                    HttpResponse::Ok().json(&*data)
                }))
        })
        .listen(listener)
        .unwrap()
        .run();
        let base = format!("http://{}:{}", addr.ip(), addr.port());
        (format!("{}/.well-known/jwks.json", base), server)
    }

    fn make_encoding_key(private_key: &RsaPrivateKey) -> EncodingKey {
        let pem = private_key.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF).unwrap();
        EncodingKey::from_rsa_pem(pem.as_bytes()).unwrap()
    }

    #[tokio::test]
    async fn valid_token_ok() {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let public_key = RsaPublicKey::from(&private_key);
        let kid = "test-kid";
        let jwk = make_jwk(&public_key, kid);
        let (jwks_uri, server) = start_jwks_server(Jwks { keys: vec![jwk] }).await;
        let _ = tokio::spawn(server);

        std::env::set_var("AUTH0_DOMAIN", "test.local");
        std::env::set_var("AUTH0_ISSUER", "https://test.local/");
        std::env::set_var("AUTH0_AUDIENCE", "https://api.conhub.dev");
        std::env::set_var("AUTH0_JWKS_URI", jwks_uri);

        let verifier = Auth0Verifier {
            config: Auth0Config::from_env().unwrap(),
            jwks_cache: Arc::new(tokio::sync::Mutex::new(Auth0JwksCache::new(Auth0Config::from_env().unwrap()))),
        };

        #[derive(Serialize)]
        struct ClaimsForSign {
            sub: String,
            iss: String,
            aud: String,
            exp: usize,
            iat: usize,
            email: String,
            scope: String,
        }

        let now = chrono::Utc::now().timestamp() as usize;
        let claims = ClaimsForSign {
            sub: "user-1".to_string(),
            iss: verifier.config.issuer.clone(),
            aud: verifier.config.audience.clone(),
            exp: now + 3600,
            iat: now,
            email: "u@example.com".to_string(),
            scope: "read:conhub".to_string(),
        };

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(kid.to_string());
        let token = encode(&header, &claims, &make_encoding_key(&private_key)).unwrap();

        let res = verify_auth0_jwt_token(&token, &verifier).await;
        assert!(res.is_ok());
        let c = res.unwrap();
        assert_eq!(c.sub, "user-1");
        assert_eq!(c.aud, verifier.config.audience);
    }

    #[tokio::test]
    async fn wrong_issuer_err() {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let public_key = RsaPublicKey::from(&private_key);
        let kid = "kid2";
        let jwk = make_jwk(&public_key, kid);
        let (jwks_uri, server) = start_jwks_server(Jwks { keys: vec![jwk] }).await;
        let _ = tokio::spawn(server);

        std::env::set_var("AUTH0_DOMAIN", "test.local");
        std::env::set_var("AUTH0_ISSUER", "https://expected.local/");
        std::env::set_var("AUTH0_AUDIENCE", "https://api.conhub.dev");
        std::env::set_var("AUTH0_JWKS_URI", jwks_uri);

        let verifier = Auth0Verifier {
            config: Auth0Config::from_env().unwrap(),
            jwks_cache: Arc::new(tokio::sync::Mutex::new(Auth0JwksCache::new(Auth0Config::from_env().unwrap()))),
        };

        #[derive(Serialize)]
        struct ClaimsForSign { sub: String, iss: String, aud: String, exp: usize, iat: usize }
        let now = chrono::Utc::now().timestamp() as usize;
        let claims = ClaimsForSign {
            sub: "user-x".to_string(),
            iss: "https://wrong.local/".to_string(),
            aud: verifier.config.audience.clone(),
            exp: now + 3600,
            iat: now,
        };

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(kid.to_string());
        let token = encode(&header, &claims, &make_encoding_key(&private_key)).unwrap();

        let res = verify_auth0_jwt_token(&token, &verifier).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn wrong_audience_err() {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let public_key = RsaPublicKey::from(&private_key);
        let kid = "kid3";
        let jwk = make_jwk(&public_key, kid);
        let (jwks_uri, server) = start_jwks_server(Jwks { keys: vec![jwk] }).await;
        let _ = tokio::spawn(server);

        std::env::set_var("AUTH0_DOMAIN", "test.local");
        std::env::set_var("AUTH0_ISSUER", "https://test.local/");
        std::env::set_var("AUTH0_AUDIENCE", "https://api.conhub.dev");
        std::env::set_var("AUTH0_JWKS_URI", jwks_uri);

        let verifier = Auth0Verifier {
            config: Auth0Config::from_env().unwrap(),
            jwks_cache: Arc::new(tokio::sync::Mutex::new(Auth0JwksCache::new(Auth0Config::from_env().unwrap()))),
        };

        #[derive(Serialize)]
        struct ClaimsForSign { sub: String, iss: String, aud: String, exp: usize, iat: usize }
        let now = chrono::Utc::now().timestamp() as usize;
        let claims = ClaimsForSign {
            sub: "user-x".to_string(),
            iss: verifier.config.issuer.clone(),
            aud: "https://wrong-aud".to_string(),
            exp: now + 3600,
            iat: now,
        };

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(kid.to_string());
        let token = encode(&header, &claims, &make_encoding_key(&private_key)).unwrap();

        let res = verify_auth0_jwt_token(&token, &verifier).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn expired_token_err() {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let public_key = RsaPublicKey::from(&private_key);
        let kid = "kid4";
        let jwk = make_jwk(&public_key, kid);
        let (jwks_uri, server) = start_jwks_server(Jwks { keys: vec![jwk] }).await;
        let _ = tokio::spawn(server);

        std::env::set_var("AUTH0_DOMAIN", "test.local");
        std::env::set_var("AUTH0_ISSUER", "https://test.local/");
        std::env::set_var("AUTH0_AUDIENCE", "https://api.conhub.dev");
        std::env::set_var("AUTH0_JWKS_URI", jwks_uri);

        let verifier = Auth0Verifier {
            config: Auth0Config::from_env().unwrap(),
            jwks_cache: Arc::new(tokio::sync::Mutex::new(Auth0JwksCache::new(Auth0Config::from_env().unwrap()))),
        };

        #[derive(Serialize)]
        struct ClaimsForSign { sub: String, iss: String, aud: String, exp: usize, iat: usize }
        let now = chrono::Utc::now().timestamp() as usize;
        let claims = ClaimsForSign {
            sub: "user-x".to_string(),
            iss: verifier.config.issuer.clone(),
            aud: verifier.config.audience.clone(),
            exp: now - 10,
            iat: now - 20,
        };

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(kid.to_string());
        let token = encode(&header, &claims, &make_encoding_key(&private_key)).unwrap();

        let res = verify_auth0_jwt_token(&token, &verifier).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn missing_required_scope_err() {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let public_key = RsaPublicKey::from(&private_key);
        let kid = "kid5";
        let jwk = make_jwk(&public_key, kid);
        let (jwks_uri, server) = start_jwks_server(Jwks { keys: vec![jwk] }).await;
        let _ = tokio::spawn(server);

        std::env::set_var("AUTH0_DOMAIN", "test.local");
        std::env::set_var("AUTH0_ISSUER", "https://test.local/");
        std::env::set_var("AUTH0_AUDIENCE", "https://api.conhub.dev");
        std::env::set_var("AUTH0_JWKS_URI", jwks_uri);
        std::env::set_var("AUTH0_REQUIRED_SCOPE", "read:conhub");

        let verifier = Auth0Verifier {
            config: Auth0Config::from_env().unwrap(),
            jwks_cache: Arc::new(tokio::sync::Mutex::new(Auth0JwksCache::new(Auth0Config::from_env().unwrap()))),
        };

        #[derive(Serialize)]
        struct ClaimsForSign { sub: String, iss: String, aud: String, exp: usize, iat: usize, scope: String }
        let now = chrono::Utc::now().timestamp() as usize;
        let claims = ClaimsForSign {
            sub: "user-x".to_string(),
            iss: verifier.config.issuer.clone(),
            aud: verifier.config.audience.clone(),
            exp: now + 3600,
            iat: now,
            scope: "write:conhub".to_string(),
        };

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(kid.to_string());
        let token = encode(&header, &claims, &make_encoding_key(&private_key)).unwrap();

        let res = verify_auth0_jwt_token(&token, &verifier).await;
        assert!(res.is_err());
    }
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
