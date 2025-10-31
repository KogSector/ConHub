use actix_web::{web, HttpResponse, Result};
use crate::state::AppState;
use crate::models::auth_dto::{LoginRequest, RegisterRequest, AuthResponse, UserResponse};
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
            .route("/oauth/google", web::post().to(google_oauth))
            .route("/oauth/github", web::post().to(github_oauth))
            .route("/oauth/microsoft", web::post().to(microsoft_oauth))
            .route("/reset-password", web::post().to(request_password_reset))
            .route("/reset-password/confirm", web::post().to(confirm_password_reset))
    );
}
