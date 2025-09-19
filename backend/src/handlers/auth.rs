use actix_web::{web, HttpRequest, HttpResponse, Result, HttpMessage, cookie::{Cookie, SameSite}};
use validator::Validate;
use crate::models::auth::*;
use crate::services::auth_service::{AuthService, AuthError};
use crate::services::session_service::SessionService;
use crate::middleware::auth::{extract_claims_from_http_request, extract_user_id_from_http_request};
use uuid::Uuid;

pub async fn register(
    auth_service: web::Data<AuthService>,
    request: web::Json<RegisterRequest>,
) -> Result<HttpResponse> {
    // Validate request
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    match auth_service.register(request.into_inner()).await {
        Ok(response) => Ok(HttpResponse::Created().json(response)),
        Err(AuthError::UserAlreadyExists) => {
            Ok(HttpResponse::Conflict().json(serde_json::json!({
                "error": "User with this email already exists"
            })))
        }
        Err(AuthError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": msg
            })))
        }
        Err(err) => {
            log::error!("Registration error: {:?}", err);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error"
            })))
        }
    }
}

pub async fn login(
    auth_service: web::Data<AuthService>,
    session_service: web::Data<SessionService>,
    request: web::Json<LoginRequest>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // Validate request
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    match auth_service.login(request.into_inner()).await {
        Ok(response) => {
            // Extract user info from the response
            let user_id = response.user.id;
            let email = response.user.email.clone();
            
            // Get client info for session
            let ip_address = req.connection_info().peer_addr().map(|s| s.to_string());
            let user_agent = req.headers().get("User-Agent")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string());
            
            // Create session
            let session_id = session_service.create_session(user_id, email, ip_address, user_agent).await;
            
            // Create secure cookie
            let cookie = Cookie::build("session_id", session_id)
                .path("/")
                .max_age(actix_web::cookie::time::Duration::hours(24))
                .http_only(true)
                .secure(false) // Set to true in production with HTTPS
                .same_site(SameSite::Lax)
                .finish();
            
            Ok(HttpResponse::Ok()
                .cookie(cookie)
                .json(response))
        }
        Err(AuthError::InvalidCredentials) => {
            Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid email or password"
            })))
        }
        Err(err) => {
            log::error!("Login error: {:?}", err);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error"
            })))
        }
    }
}

pub async fn get_profile(
    auth_service: web::Data<AuthService>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // For protected routes, claims are already validated by middleware
    let user_id = match req.extensions().get::<Claims>() {
        Some(claims) => match Uuid::parse_str(&claims.sub) {
            Ok(id) => id,
            Err(_) => return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid user ID in token"
            }))),
        },
        None => return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Authentication required"
        }))),
    };

    match auth_service.get_user_by_id(user_id).await {
        Ok(user) => {
            let profile: UserProfile = user.into();
            Ok(HttpResponse::Ok().json(profile))
        }
        Err(AuthError::UserNotFound) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "User not found"
            })))
        }
        Err(err) => {
            log::error!("Get profile error: {:?}", err);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error"
            })))
        }
    }
}

pub async fn update_profile(
    auth_service: web::Data<AuthService>,
    req: HttpRequest,
    request: web::Json<UpdateProfileRequest>,
) -> Result<HttpResponse> {
    let user_id = match extract_user_id_from_http_request(&req) {
        Some(id) => id,
        None => return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Authentication required"
        }))),
    };

    // Validate request
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    match auth_service.update_profile(user_id, request.into_inner()).await {
        Ok(profile) => Ok(HttpResponse::Ok().json(profile)),
        Err(AuthError::UserNotFound) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "User not found"
            })))
        }
        Err(err) => {
            log::error!("Update profile error: {:?}", err);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error"
            })))
        }
    }
}

pub async fn change_password(
    auth_service: web::Data<AuthService>,
    req: HttpRequest,
    request: web::Json<ChangePasswordRequest>,
) -> Result<HttpResponse> {
    let user_id = match extract_user_id_from_http_request(&req) {
        Some(id) => id,
        None => return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Authentication required"
        }))),
    };

    // Validate request
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    match auth_service.change_password(user_id, request.into_inner()).await {
        Ok(()) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Password changed successfully"
        }))),
        Err(AuthError::InvalidCredentials) => {
            Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Current password is incorrect"
            })))
        }
        Err(AuthError::UserNotFound) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "User not found"
            })))
        }
        Err(err) => {
            log::error!("Change password error: {:?}", err);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error"
            })))
        }
    }
}

pub async fn verify_token(
    _auth_service: web::Data<AuthService>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    match extract_claims_from_http_request(&req) {
        Some(claims) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "valid": true,
            "claims": claims
        }))),
        None => Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "valid": false,
            "error": "Authentication required"
        }))),
    }
}

pub async fn logout(
    session_service: web::Data<SessionService>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // Try to get session ID from cookie
    if let Some(cookie_header) = req.headers().get("Cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if cookie.starts_with("session_id=") {
                    let session_id = &cookie[11..];
                    session_service.remove_session(session_id).await;
                    break;
                }
            }
        }
    }
    
    // Clear the session cookie
    let cookie = Cookie::build("session_id", "")
        .path("/")
        .max_age(actix_web::cookie::time::Duration::seconds(0))
        .http_only(true)
        .secure(false) // Set to true in production with HTTPS
        .same_site(SameSite::Lax)
        .finish();
    
    Ok(HttpResponse::Ok()
        .cookie(cookie)
        .json(serde_json::json!({
            "message": "Logged out successfully"
        })))
}

pub async fn forgot_password(
    auth_service: web::Data<AuthService>,
    request: web::Json<ForgotPasswordRequest>,
) -> Result<HttpResponse> {
    // Validate request
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    // For security reasons, always return success even if email doesn't exist
    // This prevents email enumeration attacks
    match auth_service.initiate_password_reset(&request.email).await {
        Ok(_) => {
            log::info!("Password reset initiated for email: {}", request.email);
        }
        Err(AuthError::UserNotFound) => {
            log::info!("Password reset attempted for non-existent email: {}", request.email);
        }
        Err(err) => {
            log::error!("Forgot password error: {:?}", err);
        }
    }

    // Always return success response for security
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "If an account with that email exists, we've sent a password reset link."
    })))
}

/// Reset password using reset token
pub async fn reset_password(
    auth_service: web::Data<AuthService>,
    request: web::Json<ResetPasswordRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    // Validate request
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    match auth_service.reset_password(&request.token, &request.new_password).await {
        Ok(()) => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Password has been successfully reset."
            })))
        }
        Err(AuthError::InvalidCredentials) => {
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid or expired reset token."
            })))
        }
        Err(AuthError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": msg
            })))
        }
        Err(err) => {
            log::error!("Reset password error: {:?}", err);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error"
            })))
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/logout", web::post().to(logout))
            .route("/profile", web::get().to(get_profile))
            .route("/profile", web::put().to(update_profile))
            .route("/change-password", web::post().to(change_password))
            .route("/verify", web::post().to(verify_token))
            .route("/forgot-password", web::post().to(forgot_password))
            .route("/reset-password", web::post().to(reset_password))
    );
}