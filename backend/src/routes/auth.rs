use actix_web::{web, HttpResponse, Result};
use crate::state::AppState;
use conhub_middleware::auth::extract_claims_from_http_request;
use crate::models::auth_dto::{LoginRequest, RegisterRequest, AuthResponse, UserResponse};
use conhub_models::auth::{default_dev_user_profile, UserProfile, UserRole, SubscriptionTier};
use serde::{Deserialize, Serialize};
use validator::Validate;

pub async fn login(
    state: web::Data<AppState>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    // Validate request
    body.validate()
        .map_err(|e| actix_web::error::ErrorBadRequest(e))?;

    // Call auth service
    match state.auth_service.authenticate_user(&body.email, &body.password).await {
        Ok(result) => {
            let response = AuthResponse {
                token: result.token,
                user: UserResponse {
                    id: result.user.id,
                    email: result.user.email,
                    name: result.user.name,
                },
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            log::error!("Login failed: {}", e);
            Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid credentials"
            })))
        }
    }
}

pub async fn register(
    state: web::Data<AppState>,
    body: web::Json<RegisterRequest>,
) -> Result<HttpResponse> {
    // Validate request
    body.validate()
        .map_err(|e| actix_web::error::ErrorBadRequest(e))?;

    // Call auth service
    match state.auth_service.register_user(&body.email, &body.password, &body.name).await {
        Ok(result) => {
            let response = AuthResponse {
                token: result.token,
                user: UserResponse {
                    id: result.user.id,
                    email: result.user.email,
                    name: result.user.name,
                },
            };
            Ok(HttpResponse::Created().json(response))
        }
        Err(e) => {
            log::error!("Registration failed: {}", e);
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("{}", e)
            })))
        }
    }
}

pub async fn logout(
    state: web::Data<AppState>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse> {
    // Extract token from Authorization header
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                let _ = state.auth_service.logout_user(token).await;
            }
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}

pub async fn get_current_user(
    state: web::Data<AppState>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse> {
    // First, try to read injected claims (present when Auth is disabled)
    if let Some(claims) = extract_claims_from_http_request(&req) {
        return Ok(HttpResponse::Ok().json(UserResponse {
            id: claims.sub,
            email: claims.email,
            name: "Development User".to_string(),
        }));
    }

    // Extract token from Authorization header
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                match state.auth_service.validate_token(token).await {
                    Ok(user) => {
                        return Ok(HttpResponse::Ok().json(UserResponse {
                            id: user.id,
                            email: user.email,
                            name: user.name,
                        }));
                    }
                    Err(e) => {
                        log::error!("Token validation failed: {}", e);
                    }
                }
            }
        }
    }

    Ok(HttpResponse::Unauthorized().json(serde_json::json!({
        "error": "Invalid or missing token"
    })))
}

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize, Clone)]
struct FrontendUser {
    id: String,
    email: String,
    name: String,
    avatar_url: Option<String>,
    organization: Option<String>,
    role: String,                 // 'admin' | 'user' | 'moderator'
    subscription_tier: String,    // 'free' | 'personal' | 'team' | 'enterprise'
    is_verified: bool,
    created_at: String,
    last_login_at: Option<String>,
}

impl From<UserProfile> for FrontendUser {
    fn from(p: UserProfile) -> Self {
        let role = match p.role { UserRole::Admin => "admin".into(), UserRole::User => "user".into() };
        let tier = match p.subscription_tier {
            SubscriptionTier::Free => "free".into(),
            SubscriptionTier::Personal => "personal".into(),
            SubscriptionTier::Team => "team".into(),
            SubscriptionTier::Enterprise => "enterprise".into(),
        };
        FrontendUser {
            id: p.id.to_string(),
            email: p.email,
            name: p.name,
            avatar_url: p.avatar_url,
            organization: p.organization,
            role,
            subscription_tier: tier,
            is_verified: p.is_verified,
            created_at: p.created_at.to_rfc3339(),
            last_login_at: p.last_login_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

#[derive(Deserialize)]
struct UpdateProfileRequest {
    name: Option<String>,
    avatar_url: Option<String>,
    organization: Option<String>,
}

pub async fn get_profile(
    _state: web::Data<AppState>,
    req: actix_web::HttpRequest,
)
-> Result<HttpResponse> {
    if let Some(claims) = extract_claims_from_http_request(&req) {
        let mut profile: UserProfile = default_dev_user_profile();
        // ensure id/email consistency with injected claims
        profile.id = uuid::Uuid::parse_str(&claims.sub).unwrap_or(profile.id);
        profile.email = claims.email.clone();
        let frontend_user: FrontendUser = profile.into();
        let resp = ApiResponse {
            success: true,
            message: "Profile fetched".to_string(),
            data: Some(frontend_user),
            error: None,
        };
        return Ok(HttpResponse::Ok().json(resp));
    }

    Ok(HttpResponse::Unauthorized().json(ApiResponse::<()> {
        success: false,
        message: "Unauthorized".to_string(),
        data: None,
        error: Some("Invalid or missing token".to_string()),
    }))
}

pub async fn update_profile(
    _state: web::Data<AppState>,
    req: actix_web::HttpRequest,
    body: web::Json<UpdateProfileRequest>,
) -> Result<HttpResponse> {
    if let Some(claims) = extract_claims_from_http_request(&req) {
        let mut profile: UserProfile = default_dev_user_profile();
        profile.id = uuid::Uuid::parse_str(&claims.sub).unwrap_or(profile.id);
        profile.email = claims.email.clone();

        // Merge updates for allowed fields
        if let Some(name) = &body.name { profile.name = name.clone(); }
        if let Some(avatar_url) = &body.avatar_url { profile.avatar_url = Some(avatar_url.clone()); }
        if let Some(org) = &body.organization { profile.organization = Some(org.clone()); }

        let frontend_user: FrontendUser = profile.into();
        let resp = ApiResponse {
            success: true,
            message: "Profile updated".to_string(),
            data: Some(frontend_user),
            error: None,
        };
        return Ok(HttpResponse::Ok().json(resp));
    }

    Ok(HttpResponse::Unauthorized().json(ApiResponse::<()> {
        success: false,
        message: "Unauthorized".to_string(),
        data: None,
        error: Some("Invalid or missing token".to_string()),
    }))
}

#[derive(Serialize)]
struct VerifyResponse { valid: bool }

pub async fn verify_token(
    _state: web::Data<AppState>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse> {
    // When auth is disabled, claims are injected and all requests are considered valid
    if extract_claims_from_http_request(&req).is_some() {
        return Ok(HttpResponse::Ok().json(VerifyResponse { valid: true }));
    }

    // Otherwise, signal not implemented for strict auth mode in backend
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Token verification not implemented in backend service"
    })))
}

pub async fn refresh_token(
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    // TODO: Implement token refresh logic
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Not implemented yet"
    })))
}

pub async fn google_oauth(
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    // TODO: Implement Google OAuth
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Not implemented yet"
    })))
}

pub async fn github_oauth(
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    // TODO: Implement GitHub OAuth
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Not implemented yet"
    })))
}

pub async fn microsoft_oauth(
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    // TODO: Implement Microsoft OAuth
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Not implemented yet"
    })))
}

pub async fn request_password_reset(
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    // TODO: Implement password reset request
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Not implemented yet"
    })))
}

pub async fn confirm_password_reset(
    _state: web::Data<AppState>,
) -> Result<HttpResponse> {
    // TODO: Implement password reset confirmation
    Ok(HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Not implemented yet"
    })))
}

pub fn configure_auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/logout", web::post().to(logout))
            .route("/refresh", web::post().to(refresh_token))
            .route("/me", web::get().to(get_current_user))
            .route("/profile", web::get().to(get_profile))
            .route("/profile", web::put().to(update_profile))
            .route("/verify", web::post().to(verify_token))
            .route("/oauth/google", web::post().to(google_oauth))
            .route("/oauth/github", web::post().to(github_oauth))
            .route("/oauth/microsoft", web::post().to(microsoft_oauth))
            .route("/reset-password", web::post().to(request_password_reset))
            .route("/reset-password/confirm", web::post().to(confirm_password_reset))
    );
}
