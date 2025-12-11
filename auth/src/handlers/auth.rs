use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde_json::json;
use uuid::Uuid;

use chrono::{DateTime, Utc, Duration};
use jsonwebtoken::{encode, Header, EncodingKey};
use sqlx::{PgPool, Row};
use validator::Validate;
use bcrypt;

use conhub_models::auth::*;
use crate::services::{
    users::UserService,
    sessions::SessionService,
    security::SecurityService,
    dev_user::get_dev_user,
};
use reqwest::{Client, Url};
use conhub_middleware::auth::extract_claims_from_http_request;

// Disabled-mode handler: responds consistently when auth is turned off
pub async fn disabled() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "disabled": true,
        "message": "Authentication is disabled via feature toggles."
    })))
}

/// Get current user in dev mode (Auth disabled)
/// Returns the dev user profile, either from DB or in-memory fallback
pub async fn get_dev_current_user(
    pool_opt: web::Data<Option<PgPool>>,
) -> Result<HttpResponse> {
    let pool_ref = pool_opt.get_ref().as_ref();
    let profile = get_dev_user(pool_ref).await;
    
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": profile,
        "dev_mode": true
    })))
}

/// Get profile in dev mode (Auth disabled)
/// Returns the dev user profile, either from DB or in-memory fallback
pub async fn get_dev_profile(
    pool_opt: web::Data<Option<PgPool>>,
) -> Result<HttpResponse> {
    let pool_ref = pool_opt.get_ref().as_ref();
    let profile = get_dev_user(pool_ref).await;
    
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": profile,
        "dev_mode": true
    })))
}

pub async fn login(
    request: web::Json<LoginRequest>,
    pool_opt: web::Data<Option<PgPool>>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // Check if database pool is available
    let pool = match pool_opt.get_ref() {
        Some(p) => p,
        None => {
            tracing::error!("Database pool not available for login");
            return Ok(HttpResponse::ServiceUnavailable().json(json!({
                "error": "Database service unavailable. Please ensure the database is connected."
            })));
        }
    };

    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    // Initialize SecurityService for rate limiting
    let security_service = match crate::services::security::SecurityService::new(pool.clone()).await {
        Ok(service) => service,
        Err(e) => {
            tracing::error!("Failed to initialize security service: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({"error": "Service initialization failed"})));
        }
    };

    // Get client IP for rate limiting
    let client_ip = req.connection_info().realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();

    // Check rate limit for login attempts (5 attempts per minute)
    if !security_service.check_rate_limit(&client_ip, "login", 5, 1).await
        .unwrap_or(false) {
        tracing::warn!("Rate limit exceeded for login attempt from IP: {}", client_ip);
        return Ok(HttpResponse::TooManyRequests().json(json!({
            "error": "Too many login attempts. Please try again later."
        })));
    }

    let user_service = match UserService::new(pool.clone()).await {
        Ok(service) => service,
        Err(e) => {
            tracing::error!("Failed to initialize user service: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({"error": "Service initialization failed"})));
        }
    };
    
    
    let user = match user_service.verify_password(&request.email, &request.password).await {
        Ok(user) => user,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Invalid credentials"
            })));
        }
    };

    
    if let Err(e) = user_service.update_last_login(user.id).await {
        tracing::warn!("Failed to update last login for user {}: {}", user.id, e);
    }

    // Generate secure tokens using SecurityService
    let session_id = Uuid::new_v4();
    let remember_me = false; // Could be extracted from request if needed
    let (token, refresh_token, expires_at, _refresh_expires) = match security_service.generate_jwt_token(&user, session_id, remember_me).await {
        Ok(tokens) => tokens,
        Err(e) => {
            tracing::error!("Failed to generate JWT tokens: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to generate authentication tokens"
            })));
        }
    };

    let user_profile = UserProfile::from(user);
    
    let auth_response = AuthResponse {
        user: user_profile,
        token,
        refresh_token,
        expires_at,
        session_id,
    };

    Ok(HttpResponse::Ok().json(auth_response))
}

pub async fn forgot_password(
    request: web::Json<ForgotPasswordRequest>,
    pool_opt: web::Data<Option<PgPool>>,
) -> Result<HttpResponse> {
    let pool = match pool_opt.get_ref() {
        Some(p) => p,
        None => {
            return Ok(HttpResponse::ServiceUnavailable().json(json!({
                "error": "Database service unavailable"
            })));
        }
    };
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    let email = &request.email;
    tracing::info!("Password reset requested for email: {}", email);
    
    let user_service = match UserService::new(pool.clone()).await {
        Ok(service) => service,
        Err(e) => {
            tracing::error!("Failed to initialize user service: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({"error": "Service initialization failed"})));
        }
    };
    let password_reset_service = crate::services::password_reset::PasswordResetService::new(pool.clone());
    
    // Check if user exists
    let user_exists = match user_service.find_by_email(email).await {
        Ok(_) => true,
        Err(_) => false,
    };
    
    // Generate reset token only if user exists
    if user_exists {
        match password_reset_service.generate_reset_token(email).await {
            Ok(_) => {
                tracing::info!("Password reset token generated for email: {}", email);
            },
            Err(e) => {
                tracing::error!("Failed to generate reset token for {}: {}", email, e);
            }
        }
    }

    // Always return success to prevent email enumeration
    Ok(HttpResponse::Ok().json(json!({
        "message": "If an account with that email exists, we've sent a password reset link.",
        "success": true
    })))
}

pub async fn reset_password(
    request: web::Json<ResetPasswordRequest>,
    pool_opt: web::Data<Option<PgPool>>,
) -> Result<HttpResponse> {
    let pool = match pool_opt.get_ref() {
        Some(p) => p,
        None => {
            return Ok(HttpResponse::ServiceUnavailable().json(json!({
                "error": "Database service unavailable"
            })));
        }
    };
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    let token = &request.token;
    let new_password = &request.new_password;
    
    tracing::info!("Password reset attempted for token: {}", token);
    
    let password_reset_service = crate::services::password_reset::PasswordResetService::new(pool.clone());
    
    // Validate the reset token and get the email
    let email = match password_reset_service.validate_token(token).await {
        Ok(email) => email,
        Err(e) => {
            tracing::warn!("Invalid password reset token: {}", e);
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Invalid or expired reset token",
                "details": format!("{}", e)
            })));
        }
    };
    
    // Initialize SecurityService for password operations
    let security_service = match crate::services::security::SecurityService::new(pool.clone()).await {
        Ok(service) => service,
        Err(e) => {
            tracing::error!("Failed to initialize security service: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Service initialization failed"
            })));
        }
    };

    // Validate password strength
    if let Err(validation_error) = security_service.validate_password_strength(new_password) {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Password validation failed",
            "details": validation_error
        })));
    }

    // Hash the new password using Argon2
    let new_password_hash = match security_service.hash_password(new_password) {
        Ok(hash) => hash,
        Err(e) => {
            tracing::error!("Failed to hash password: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to process password reset"
            })));
        }
    };

    let user_service = match UserService::new(pool.clone()).await {
        Ok(service) => service,
        Err(e) => {
            tracing::error!("Failed to initialize user service: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({"error": "Service initialization failed"})));
        }
    };
    
    // Find the user by email
    let user = match user_service.find_by_email(&email).await {
        Ok(user) => user,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "User not found"
            })));
        }
    };
    
    // Update the user's password
    if let Err(e) = user_service.update_password(user.id, &new_password_hash).await {
        tracing::error!("Failed to update password for user {}: {}", user.id, e);
        return Ok(HttpResponse::InternalServerError().json(json!({
            "error": "Failed to update password"
        })));
    }
    
    tracing::info!("Password successfully reset for email: {}", email);

    Ok(HttpResponse::Ok().json(json!({
        "message": "Password has been reset successfully. You can now log in with your new password.",
        "success": true
    })))
}

pub async fn register(
    request: web::Json<RegisterRequest>,
    pool_opt: web::Data<Option<PgPool>>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    tracing::info!("📝 [Register] Registration request received for email: {}", request.email);
    
    // Check if database pool is available
    let pool = match pool_opt.get_ref() {
        Some(p) => {
            tracing::info!("✅ [Register] Database pool is available");
            p
        },
        None => {
            tracing::error!("❌ [Register] Database pool not available for registration");
            return Ok(HttpResponse::ServiceUnavailable().json(json!({
                "error": "Database service unavailable. Please ensure the database is connected."
            })));
        }
    };

    if let Err(validation_errors) = request.validate() {
        tracing::warn!("⚠️  [Register] Validation failed: {:?}", validation_errors);
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    tracing::info!("✅ [Register] Validation passed");

    // Initialize SecurityService for rate limiting
    tracing::info!("🔐 [Register] Initializing SecurityService...");
    let security_service = match crate::services::security::SecurityService::new(pool.clone()).await {
        Ok(service) => {
            tracing::info!("✅ [Register] SecurityService initialized");
            service
        },
        Err(e) => {
            tracing::error!("❌ [Register] Failed to initialize security service: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({"error": "Service initialization failed"})));
        }
    };

    // Get client IP for rate limiting
    let client_ip = req.connection_info().realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();
    
    tracing::info!("🌐 [Register] Client IP: {}", client_ip);

    // Check rate limit for registration attempts (3 attempts per minute)
    tracing::info!("⏱️  [Register] Checking rate limit...");
    if !security_service.check_rate_limit(&client_ip, "register", 3, 1).await
        .unwrap_or(false) {
        tracing::warn!("⚠️  [Register] Rate limit exceeded for registration attempt from IP: {}", client_ip);
        return Ok(HttpResponse::TooManyRequests().json(json!({
            "error": "Too many registration attempts. Please try again later."
        })));
    }

    tracing::info!("✅ [Register] Rate limit check passed");
    tracing::info!("👤 [Register] Initializing UserService...");
    
    let user_service = match UserService::new(pool.clone()).await {
        Ok(service) => {
            tracing::info!("✅ [Register] UserService initialized");
            service
        },
        Err(e) => {
            tracing::error!("❌ [Register] Failed to initialize user service: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({"error": "Service initialization failed"})));
        }
    };
    
    tracing::info!("💾 [Register] Creating user in database...");
    let new_user = match user_service.create_user(&request).await {
        Ok(user) => {
            tracing::info!("✅ [Register] User created successfully: {} ({})", user.email, user.id);
            user
        },
        Err(e) => {
            let msg = e.to_string();
            tracing::error!("❌ [Register] Failed to create user: {}", msg);
            if msg.contains("already exists") {
                return Ok(HttpResponse::Conflict().json(json!({
                    "error": "User already exists",
                    "details": msg
                })));
            }
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Failed to create user",
                "details": msg
            })));
        }
    };

    // Generate secure tokens using SecurityService
    tracing::info!("🔑 [Register] Generating JWT tokens...");
    let session_id = Uuid::new_v4();
    let remember_me = false; // Could be extracted from request if needed
    let (token, refresh_token, expires_at, _refresh_expires) = match security_service.generate_jwt_token(&new_user, session_id, remember_me).await {
        Ok(tokens) => {
            tracing::info!("✅ [Register] JWT tokens generated successfully");
            tokens
        },
        Err(e) => {
            tracing::error!("❌ [Register] Failed to generate JWT tokens for user {}: {}", new_user.id, e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to generate authentication tokens"
            })));
        }
    };

    let user_profile = UserProfile::from(new_user);
    
    let auth_response = AuthResponse {
        user: user_profile,
        token,
        refresh_token,
        expires_at,
        session_id,
    };

    tracing::info!("🎉 [Register] Registration completed successfully!");
    Ok(HttpResponse::Created().json(auth_response))
}

pub async fn verify_token(req: HttpRequest) -> Result<HttpResponse> {
    if let Some(claims) = extract_claims_from_http_request(&req) {
        Ok(HttpResponse::Ok().json(json!({
            "valid": true,
            "sub": claims.sub,
            "email": claims.email,
            "roles": claims.roles,
            "exp": claims.exp,
        })))
    } else {
        Ok(HttpResponse::Unauthorized().json(json!({
            "valid": false,
            "error": "Invalid or missing token"
        })))
    }
}

pub async fn dev_reset(pool_opt: web::Data<Option<PgPool>>) -> Result<HttpResponse> {
    let pool = match pool_opt.get_ref() {
        Some(p) => p,
        None => {
            return Ok(HttpResponse::ServiceUnavailable().json(json!({
                "error": "Database service unavailable"
            })));
        }
    };

    let env = std::env::var("NODE_ENV").unwrap_or_default();
    if env != "development" {
        return Ok(HttpResponse::Forbidden().json(json!({
            "error": "Reset is only allowed in development"
        })));
    }

    if let Err(e) = sqlx::query("TRUNCATE TABLE users RESTART IDENTITY CASCADE").execute(pool).await {
        tracing::error!("Failed to truncate users table: {}", e);
        return Ok(HttpResponse::InternalServerError().json(json!({
            "error": "Failed to reset database",
            "details": format!("{}", e)
        })));
    }

    Ok(HttpResponse::Ok().json(json!({
        "message": "Development database reset: users cleared"
    })))
}

pub async fn logout(
    req: HttpRequest,
    request: web::Json<LogoutRequest>,
    pool_opt: web::Data<Option<PgPool>>,
) -> Result<HttpResponse> {
    let pool = match pool_opt.get_ref() {
        Some(p) => p,
        None => {
            return Ok(HttpResponse::ServiceUnavailable().json(json!({
                "error": "Database service unavailable"
            })));
        }
    };
    use crate::services::middleware::extract_claims_from_request;
    
    // Get user claims from the request
    let claims = match extract_claims_from_request(&req) {
        Some(claims) => claims,
        None => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Authentication required"
            })));
        }
    };

    let user_id: uuid::Uuid = match claims.sub.parse() {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Invalid user ID in token"
            })));
        }
    };

    let session_id: uuid::Uuid = match claims.session_id.parse() {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Invalid session ID in token"
            })));
        }
    };

    // Initialize session service
    let redis_client = match req.app_data::<web::Data<redis::Client>>() {
        Some(client) => client.get_ref().clone(),
        None => {
            tracing::error!("Redis client not found in app data");
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Session service unavailable"
            })));
        }
    };

    let security_service = match SecurityService::new(pool.clone()).await {
        Ok(service) => std::sync::Arc::new(service),
        Err(e) => {
            tracing::error!("Failed to initialize security service: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Service initialization failed"
            })));
        }
    };

    let session_service = SessionService::new(
        pool.clone(),
        redis_client,
        security_service,
    );

    // Invalidate session(s)
    if request.logout_all.unwrap_or(false) {
        // Logout from all sessions
        if let Err(e) = session_service.invalidate_all_user_sessions(user_id).await {
            tracing::error!("Failed to invalidate all sessions for user {}: {}", user_id, e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to logout from all sessions"
            })));
        }
        
        tracing::info!("User {} logged out from all sessions", user_id);
        Ok(HttpResponse::Ok().json(json!({
            "message": "Logged out from all sessions successfully"
        })))
    } else {
        // Logout from current session only
        let target_session_id = request.session_id.unwrap_or(session_id);
        
        if let Err(e) = session_service.invalidate_session(target_session_id).await {
            tracing::error!("Failed to invalidate session {} for user {}: {}", target_session_id, user_id, e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to logout"
            })));
        }
        
        tracing::info!("User {} logged out from session {}", user_id, target_session_id);
        Ok(HttpResponse::Ok().json(json!({
            "message": "Logged out successfully"
        })))
    }
}

pub async fn get_current_user(
    req: HttpRequest,
    pool_opt: web::Data<Option<PgPool>>,
) -> Result<HttpResponse> {
    let pool = match pool_opt.get_ref() {
        Some(p) => p,
        None => {
            return Ok(HttpResponse::ServiceUnavailable().json(json!({
                "error": "Database service unavailable"
            })));
        }
    };
    use crate::services::middleware::extract_claims_from_request;
    
    // Get user claims from the request
    let claims = match extract_claims_from_request(&req) {
        Some(claims) => claims,
        None => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Authentication required"
            })));
        }
    };

    let user_id: uuid::Uuid = match claims.sub.parse() {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Invalid user ID in token"
            })));
        }
    };

    // Initialize user service
    let user_service = match UserService::new(pool.clone()).await {
        Ok(service) => service,
        Err(e) => {
            tracing::error!("Failed to initialize user service: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Service initialization failed"
            })));
        }
    };

    // Get user by ID
    match user_service.get_user_by_id(user_id).await {
        Ok(Some(user)) => {
            let user_profile = UserProfile::from(user);
            Ok(HttpResponse::Ok().json(user_profile))
        }
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(json!({
                "error": "User not found"
            })))
        }
        Err(e) => {
            tracing::error!("Failed to get user {}: {}", user_id, e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to retrieve user"
            })))
        }
    }
}

pub async fn refresh_token(
    request: web::Json<RefreshTokenRequest>,
    pool_opt: web::Data<Option<PgPool>>,
    redis_client: web::Data<redis::Client>,
) -> Result<HttpResponse> {
    let pool = match pool_opt.get_ref() {
        Some(p) => p,
        None => {
            return Ok(HttpResponse::ServiceUnavailable().json(json!({
                "error": "Database service unavailable"
            })));
        }
    };
    // Initialize services
    let security_service = match SecurityService::new(pool.clone()).await {
        Ok(service) => std::sync::Arc::new(service),
        Err(e) => {
            tracing::error!("Failed to initialize security service: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Service initialization failed"
            })));
        }
    };

    let session_service = SessionService::new(
        pool.clone(),
        redis_client.get_ref().clone(),
        security_service,
    );

    // Refresh the token
    match session_service.refresh_token(&request.refresh_token).await {
        Ok((new_access_token, expires_at)) => {
            let response = RefreshTokenResponse {
                token: new_access_token,
                expires_at,
            };
            
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            tracing::warn!("Token refresh failed: {}", e);
            Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Invalid or expired refresh token"
            })))
        }
    }
}

pub async fn get_profile(pool_opt: web::Data<Option<PgPool>>) -> Result<HttpResponse> {
    let pool = match pool_opt.get_ref() {
        Some(p) => p,
        None => {
            return Ok(HttpResponse::ServiceUnavailable().json(json!({
                "error": "Database service unavailable"
            })));
        }
    };
    
    let user_service = match UserService::new(pool.clone()).await {
        Ok(service) => service,
        Err(e) => {
            tracing::error!("Failed to initialize user service: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({"error": "Service initialization failed"})));
        }
    };
    
    match user_service.list_users(1, 0).await {
        Ok(users) => {
            let users: Vec<User> = users;
            if let Some(user) = users.first() {
                let user_profile = UserProfile::from(user.clone());
                Ok(HttpResponse::Ok().json(user_profile))
            } else {
                Ok(HttpResponse::NotFound().json(json!({
                    "error": "No users found"
                })))
            }
        }
        Err(e) => {
            tracing::error!("Failed to get user profile: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to get profile"
            })))
        }
    }
}

pub async fn list_users(pool_opt: web::Data<Option<PgPool>>) -> Result<HttpResponse> {
    let pool = match pool_opt.get_ref() {
        Some(p) => p,
        None => {
            return Ok(HttpResponse::ServiceUnavailable().json(json!({
                "error": "Database service unavailable"
            })));
        }
    };
    let user_service = match UserService::new(pool.clone()).await {
        Ok(service) => service,
        Err(e) => {
            tracing::error!("Failed to initialize user service: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({"error": "Service initialization failed"})));
        }
    };
    
    match user_service.list_users(10, 0).await {
        Ok(users) => {
            let users: Vec<User> = users;
            let user_profiles: Vec<UserProfile> = users.into_iter()
                .map(UserProfile::from)
                .collect();
            
            Ok(HttpResponse::Ok().json(json!({
                "users": user_profiles,
                "count": user_profiles.len()
            })))
        }
        Err(e) => {
            tracing::error!("Failed to list users: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to list users"
            })))
        }
    }
}

pub async fn oauth_callback(
    request: web::Json<OAuthCallbackRequest>,
    pool_opt: web::Data<Option<PgPool>>,
) -> Result<HttpResponse> {
    let pool = match pool_opt.get_ref() {
        Some(p) => p,
        None => {
            return Ok(HttpResponse::ServiceUnavailable().json(json!({
                "error": "Database service unavailable"
            })));
        }
    };
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    let user_service = match UserService::new(pool.clone()).await {
        Ok(service) => service,
        Err(e) => {
            tracing::error!("Failed to initialize user service: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({"error": "Service initialization failed"})));
        }
    };
    
    // Try to find existing user by email
    let (user, is_new_user) = match user_service.find_by_email(&request.email).await {
        Ok(existing_user) => (existing_user, false),
        Err(_) => {
            // User doesn't exist, create new one
            // Generate a random password for OAuth users (they won't use it)
            let random_password = Uuid::new_v4().to_string();
            let register_request = RegisterRequest {
                email: request.email.clone(),
                password: random_password,
                name: request.name.clone().unwrap_or_else(|| request.email.split('@').next().unwrap_or("User").to_string()),
                avatar_url: request.avatar_url.clone(),
                organization: None,
            };
            
            match user_service.create_user(&register_request).await {
                Ok(new_user) => {
                    tracing::info!("Created new user via OAuth: {} ({})", new_user.email, new_user.id);
                    (new_user, true)
                },
                Err(e) => {
                    tracing::error!("Failed to create OAuth user: {}", e);
                    return Ok(HttpResponse::InternalServerError().json(json!({
                        "error": "Failed to create user",
                        "details": format!("{}", e)
                    })));
                }
            }
        }
    };

    // Update last login
    if let Err(e) = user_service.update_last_login(user.id).await {
        tracing::warn!("Failed to update last login for OAuth user {}: {}", user.id, e);
    }

    // Calculate token expiration datetime
    let token_expires_at = request.expires_at.map(|ts| {
        chrono::DateTime::<Utc>::from_timestamp(ts, 0)
            .unwrap_or_else(|| Utc::now() + Duration::hours(1))
    });

    // Create or update social connection
    let connection_id = Uuid::new_v4();
    let now = Utc::now();
    
    let result = sqlx::query(
        r#"
        INSERT INTO social_connections (
            id, user_id, platform, platform_user_id, username,
            access_token, refresh_token, token_expires_at, scope,
            is_active, created_at, updated_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12
        )
        ON CONFLICT (user_id, platform, platform_user_id) 
        DO UPDATE SET
            access_token = EXCLUDED.access_token,
            refresh_token = EXCLUDED.refresh_token,
            token_expires_at = EXCLUDED.token_expires_at,
            scope = EXCLUDED.scope,
            is_active = true,
            updated_at = EXCLUDED.updated_at
        RETURNING id
        "#
    )
    .bind(connection_id)
    .bind(user.id)
    .bind(request.provider.to_lowercase())
    .bind(&request.provider_user_id)
    .bind(request.email.split('@').next().unwrap_or("user"))
    .bind(&request.access_token)
    .bind(request.refresh_token.as_ref())
    .bind(token_expires_at)
    .bind(request.scope.as_ref().unwrap_or(&"".to_string()))
    .bind(true)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await;

    let final_connection_id: Uuid = match result {
        Ok(row) => row.get("id"),
        Err(e) => {
            tracing::error!("Failed to create/update social connection: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to store social connection"
            })));
        }
    };

    tracing::info!("OAuth callback successful for user {} with provider {}", user.id, request.provider);

    Ok(HttpResponse::Ok().json(OAuthCallbackResponse {
        user_id: user.id,
        is_new_user,
        connection_id: final_connection_id,
    }))
}

pub fn configure_auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/auth")
            .route("/login", web::post().to(login))
            .route("/register", web::post().to(register))
            .route("/forgot-password", web::post().to(forgot_password))
            .route("/reset-password", web::post().to(reset_password))
            .route("/verify", web::post().to(verify_token))
            .route("/profile", web::get().to(get_profile))
            .route("/users", web::get().to(list_users))
            .route("/oauth/callback", web::post().to(oauth_callback))
            .route("/oauth/url", web::get().to(oauth_url))
            .route("/oauth/exchange", web::post().to(oauth_exchange))
            .route("/connections", web::get().to(list_auth_connections))
            .route("/connections/{id}", web::delete().to(disconnect_auth_connection))
            .route("/repos/github", web::get().to(list_github_repos))
            .route("/repos/github/branches", web::get().to(list_github_branches))
            .route("/repos/bitbucket", web::get().to(list_bitbucket_repos))
            .route("/repos/bitbucket/branches", web::get().to(list_bitbucket_branches))
            .route("/repos/check", web::post().to(check_repo))
            .route("/dev/reset", web::post().to(dev_reset))
    );
}

#[derive(serde::Serialize)]
struct SocialConnectionDto {
    id: uuid::Uuid,
    platform: String,
    username: String,
    is_active: bool,
    connected_at: chrono::DateTime<chrono::Utc>,
    last_sync: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn list_auth_connections(
    req: HttpRequest,
    pool_opt: web::Data<Option<PgPool>>,
) -> Result<HttpResponse> {
    let pool = match pool_opt.get_ref() { Some(p) => p, None => return Ok(HttpResponse::ServiceUnavailable().json(json!({"error": "Database service unavailable"}))) };
    use crate::services::middleware::extract_user_id_from_request_async;
    let user_id = match extract_user_id_from_request_async(&req, pool).await {
        Some(id) => id,
        None => return Ok(HttpResponse::Unauthorized().json(json!({"error": "Authentication required or user not found"})))
    };

    let rows = sqlx::query(
        r#"
        SELECT id, platform, username, is_active, created_at AS connected_at, last_sync
        FROM social_connections
        WHERE user_id = $1
        ORDER BY updated_at DESC
        "#
    )
    .bind(user_id)
    .fetch_all(pool)
    .await;

    match rows {
        Ok(list) => {
            let data: Vec<SocialConnectionDto> = list.into_iter().map(|row| SocialConnectionDto {
                id: row.get("id"),
                platform: row.get("platform"),
                username: row.get("username"),
                is_active: row.get("is_active"),
                connected_at: row.get("connected_at"),
                last_sync: row.get("last_sync"),
            }).collect();
            Ok(HttpResponse::Ok().json(json!({"success": true, "data": data})))
        },
        Err(e) => Ok(HttpResponse::InternalServerError().json(json!({"error": format!("Failed to list connections: {}", e)})))
    }
}

pub async fn disconnect_auth_connection(
    req: HttpRequest,
    path: web::Path<String>,
    pool_opt: web::Data<Option<PgPool>>,
) -> Result<HttpResponse> {
    let pool = match pool_opt.get_ref() { Some(p) => p, None => return Ok(HttpResponse::ServiceUnavailable().json(json!({"error": "Database service unavailable"}))) };
    use crate::services::middleware::extract_user_id_from_request_async;
    let user_id = match extract_user_id_from_request_async(&req, pool).await {
        Some(id) => id,
        None => return Ok(HttpResponse::Unauthorized().json(json!({"error": "Authentication required or user not found"})))
    };
    let conn_id_str = path.into_inner();
    let conn_id = uuid::Uuid::parse_str(&conn_id_str).map_err(|_| actix_web::error::ErrorBadRequest("Invalid connection id"))?;

    let res = sqlx::query(
        r#"
        UPDATE social_connections SET is_active = false, updated_at = NOW()
        WHERE id = $1 AND user_id = $2
        "#
    )
    .bind(conn_id)
    .bind(user_id)
    .execute(pool)
    .await;

    match res {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({"success": true}))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(json!({"error": format!("Failed to disconnect: {}", e)})))
    }
}

pub async fn oauth_url(
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse> {
    use std::env;

    let provider_raw = query.get("provider").cloned().unwrap_or_default();
    let provider = provider_raw.to_lowercase();

    if provider.is_empty() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Missing provider",
            "message": "Query parameter 'provider' is required",
        })));
    }

    let state = Uuid::new_v4().to_string();

    let redirect_base = env::var("OAUTH_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:3000/auth/callback".to_string());
    let redirect_with_provider = format!("{}?provider={}", redirect_base, provider);

    let auth_url = match provider.as_str() {
        "google" => {
            let client_id = env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
            format!(
                "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=openid%20email%20profile&state={}",
                client_id,
                urlencoding::encode(&redirect_with_provider),
                state
            )
        }
        "microsoft" => {
            let client_id = env::var("MICROSOFT_CLIENT_ID").unwrap_or_default();
            let tenant = env::var("MICROSOFT_TENANT_ID").unwrap_or_else(|_| "common".to_string());
            format!(
                "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize?client_id={}&redirect_uri={}&response_type=code&scope=openid%20email%20profile&state={}",
                tenant,
                client_id,
                urlencoding::encode(&redirect_with_provider),
                state
            )
        }
        "github" => {
            let client_id = env::var("GITHUB_CLIENT_ID").unwrap_or_default();
            let scopes = "repo read:user user:email";
            format!(
                "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope={}&state={}",
                client_id,
                urlencoding::encode(&redirect_with_provider),
                urlencoding::encode(scopes),
                state
            )
        }
        "bitbucket" => {
            let client_id = env::var("BITBUCKET_CLIENT_ID").unwrap_or_default();
            let scopes = ["repository:read", "account", "email"].join(" ");
            format!(
                "https://bitbucket.org/site/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
                client_id,
                urlencoding::encode(&redirect_with_provider),
                urlencoding::encode(&scopes),
                state
            )
        }
        "gitlab" => {
            let client_id = env::var("GITLAB_CLIENT_ID").unwrap_or_default();
            let scopes = "read_repository read_user";
            format!(
                "https://gitlab.com/oauth/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
                client_id,
                urlencoding::encode(&redirect_with_provider),
                urlencoding::encode(scopes),
                state
            )
        }
        _ => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Unsupported provider",
                "provider": provider_raw,
            })));
        }
    };

    Ok(HttpResponse::Ok().json(json!({ "url": auth_url, "state": state })))
}

async fn get_bearer_token_for_provider(pool: &PgPool, user_id: uuid::Uuid, provider: &str) -> Result<String, anyhow::Error> {
    let row = sqlx::query(
        "SELECT access_token FROM social_connections WHERE user_id = $1 AND platform = $2 AND is_active = true ORDER BY updated_at DESC LIMIT 1"
    )
    .bind(user_id)
    .bind(provider)
    .fetch_optional(pool)
    .await?
    ;

    match row {
        Some(row) => Ok(row.get::<String, _>("access_token")),
        None => Err(anyhow::anyhow!("No active connection for provider")),
    }
}

pub async fn list_github_repos(
    req: HttpRequest,
    pool_opt: web::Data<Option<PgPool>>,
) -> Result<HttpResponse> {
    let pool = match pool_opt.get_ref() { Some(p) => p, None => return Ok(HttpResponse::ServiceUnavailable().json(json!({"error": "Database service unavailable"}))) };
    use crate::services::middleware::extract_claims_from_request;
    let claims = match extract_claims_from_request(&req) { Some(c) => c, None => return Ok(HttpResponse::Unauthorized().json(json!({"error": "Authentication required"}))) };
    let user_id: uuid::Uuid = claims.sub.parse().map_err(|_| actix_web::error::ErrorBadRequest("Invalid user id"))?;

    let token = match get_bearer_token_for_provider(pool, user_id, "github").await {
        Ok(t) => t,
        Err(e) => return Ok(HttpResponse::BadRequest().json(json!({"error": e.to_string()}))),
    };

    let client = Client::new();
    let resp = client
        .get("https://api.github.com/user/repos?per_page=100")
        .header("User-Agent", "ConHub")
        .bearer_auth(&token)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let repos: serde_json::Value = r.json().await.unwrap_or(json!([]));
            let simplified: Vec<serde_json::Value> = repos.as_array().unwrap_or(&vec![]).iter().map(|repo| json!({
                "name": repo["name"].as_str().unwrap_or_default(),
                "full_name": repo["full_name"].as_str().unwrap_or_default(),
                "default_branch": repo["default_branch"].as_str().unwrap_or("main")
            })).collect();
            Ok(HttpResponse::Ok().json(json!({"repos": simplified})))
        }
        Ok(r) => Ok(HttpResponse::BadRequest().json(json!({"error": format!("GitHub API error: {}", r.status())}))),
        Err(e) => Ok(HttpResponse::BadRequest().json(json!({"error": format!("GitHub request failed: {}", e)}))),
    }
}

pub async fn list_github_branches(
    req: HttpRequest,
    pool_opt: web::Data<Option<PgPool>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse> {
    let pool = match pool_opt.get_ref() { Some(p) => p, None => return Ok(HttpResponse::ServiceUnavailable().json(json!({"error": "Database service unavailable"}))) };
    use crate::services::middleware::extract_claims_from_request;
    let claims = match extract_claims_from_request(&req) { Some(c) => c, None => return Ok(HttpResponse::Unauthorized().json(json!({"error": "Authentication required"}))) };
    let user_id: uuid::Uuid = claims.sub.parse().map_err(|_| actix_web::error::ErrorBadRequest("Invalid user id"))?;
    let repo_full_name = query.get("repo").cloned().unwrap_or_default();
    if repo_full_name.is_empty() { return Ok(HttpResponse::BadRequest().json(json!({"error": "Missing repo query parameter"}))); }

    let token = match get_bearer_token_for_provider(pool, user_id, "github").await {
        Ok(t) => t,
        Err(e) => return Ok(HttpResponse::BadRequest().json(json!({"error": e.to_string()}))),
    };

    let client = Client::new();
    let url = format!("https://api.github.com/repos/{}/branches?per_page=200", repo_full_name);
    let resp = client
        .get(&url)
        .header("User-Agent", "ConHub")
        .bearer_auth(&token)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let branches: serde_json::Value = r.json().await.unwrap_or(json!([]));
            let names: Vec<String> = branches.as_array().unwrap_or(&vec![]).iter().map(|b| b["name"].as_str().unwrap_or("").to_string()).collect();
            Ok(HttpResponse::Ok().json(json!({"branches": names})))
        }
        Ok(r) => Ok(HttpResponse::BadRequest().json(json!({"error": format!("GitHub API error: {}", r.status())}))),
        Err(e) => Ok(HttpResponse::BadRequest().json(json!({"error": format!("GitHub request failed: {}", e)}))),
    }
}

pub async fn list_bitbucket_repos(
    req: HttpRequest,
    pool_opt: web::Data<Option<PgPool>>,
) -> Result<HttpResponse> {
    let pool = match pool_opt.get_ref() { Some(p) => p, None => return Ok(HttpResponse::ServiceUnavailable().json(json!({"error": "Database service unavailable"}))) };
    use crate::services::middleware::extract_claims_from_request;
    let claims = match extract_claims_from_request(&req) { Some(c) => c, None => return Ok(HttpResponse::Unauthorized().json(json!({"error": "Authentication required"}))) };
    let user_id: uuid::Uuid = claims.sub.parse().map_err(|_| actix_web::error::ErrorBadRequest("Invalid user id"))?;
    let token = match get_bearer_token_for_provider(pool, user_id, "bitbucket").await { Ok(t) => t, Err(e) => return Ok(HttpResponse::BadRequest().json(json!({"error": e.to_string()}))) };

    let client = Client::new();
    let resp = client
        .get("https://api.bitbucket.org/2.0/repositories?role=member")
        .bearer_auth(&token)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let data: serde_json::Value = r.json().await.unwrap_or(json!({"values": []}));
            let simplified: Vec<serde_json::Value> = data["values"].as_array().unwrap_or(&vec![]).iter().map(|repo| json!({
                "name": repo["name"].as_str().unwrap_or_default(),
                "full_name": repo["full_name"].as_str().unwrap_or_default(),
                "slug": repo["slug"].as_str().unwrap_or_default(),
            })).collect();
            Ok(HttpResponse::Ok().json(json!({"repos": simplified})))
        }
        Ok(r) => Ok(HttpResponse::BadRequest().json(json!({"error": format!("Bitbucket API error: {}", r.status())}))),
        Err(e) => Ok(HttpResponse::BadRequest().json(json!({"error": format!("Bitbucket request failed: {}", e)}))),
    }
}

pub async fn list_bitbucket_branches(
    req: HttpRequest,
    pool_opt: web::Data<Option<PgPool>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse> {
    let pool = match pool_opt.get_ref() { Some(p) => p, None => return Ok(HttpResponse::ServiceUnavailable().json(json!({"error": "Database service unavailable"}))) };
    use crate::services::middleware::extract_claims_from_request;
    let claims = match extract_claims_from_request(&req) { Some(c) => c, None => return Ok(HttpResponse::Unauthorized().json(json!({"error": "Authentication required"}))) };
    let user_id: uuid::Uuid = claims.sub.parse().map_err(|_| actix_web::error::ErrorBadRequest("Invalid user id"))?;
    let full_name = query.get("repo").cloned().unwrap_or_default();
    if full_name.is_empty() { return Ok(HttpResponse::BadRequest().json(json!({"error": "Missing repo query parameter"}))); }
    let token = match get_bearer_token_for_provider(pool, user_id, "bitbucket").await { Ok(t) => t, Err(e) => return Ok(HttpResponse::BadRequest().json(json!({"error": e.to_string()}))) };

    let client = Client::new();
    let url = format!("https://api.bitbucket.org/2.0/repositories/{}/refs/branches?pagelen=100", full_name);
    let resp = client.get(&url).bearer_auth(&token).send().await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let data: serde_json::Value = r.json().await.unwrap_or(json!({"values": []}));
            let names: Vec<String> = data["values"].as_array().unwrap_or(&vec![]).iter().map(|b| b["name"].as_str().unwrap_or("").to_string()).collect();
            Ok(HttpResponse::Ok().json(json!({"branches": names})))
        }
        Ok(r) => Ok(HttpResponse::BadRequest().json(json!({"error": format!("Bitbucket API error: {}", r.status())}))),
        Err(e) => Ok(HttpResponse::BadRequest().json(json!({"error": format!("Bitbucket request failed: {}", e)}))),
    }
}

#[derive(serde::Deserialize)]
pub struct RepoCheckRequest {
    provider: Option<String>,
    repo_url: String,
    access_token: Option<String>,
}

pub async fn check_repo(
    req: HttpRequest,
    payload: web::Json<RepoCheckRequest>,
    pool_opt: web::Data<Option<PgPool>>,
) -> Result<HttpResponse> {
    let pool = match pool_opt.get_ref() { Some(p) => p, None => return Ok(HttpResponse::ServiceUnavailable().json(json!({"error": "Database service unavailable"}))) };
    use crate::services::middleware::extract_claims_from_request;
    let claims = match extract_claims_from_request(&req) { Some(c) => c, None => return Ok(HttpResponse::Unauthorized().json(json!({"error": "Authentication required"}))) };
    let user_id: uuid::Uuid = claims.sub.parse().map_err(|_| actix_web::error::ErrorBadRequest("Invalid user id"))?;

    let url = &payload.repo_url;
    let host = Url::parse(url).map(|u| u.host_str().unwrap_or("").to_string()).unwrap_or_default();
    let provider = payload.provider.clone().unwrap_or_else(|| {
        if host.contains("github.com") { "github".to_string() } else if host.contains("bitbucket.org") { "bitbucket".to_string() } else { "".to_string() }
    });
    if provider.is_empty() { return Ok(HttpResponse::BadRequest().json(json!({"error": "Unsupported repo URL"}))); }

    let token = if let Some(t) = &payload.access_token { t.clone() } else {
        match get_bearer_token_for_provider(pool, user_id, &provider).await { Ok(t) => t, Err(e) => return Ok(HttpResponse::BadRequest().json(json!({"error": e.to_string()}))) }
    };

    let client = Client::new();
    if provider == "github" {
        let parts: Vec<&str> = url.trim_end_matches('/').split('/').collect();
        let (owner, repo) = (parts.get(parts.len()-2).unwrap_or(&""), parts.get(parts.len()-1).unwrap_or(&""));
        let api_url = format!("https://api.github.com/repos/{}/{}", owner, repo);
        let resp = client.get(&api_url).header("User-Agent", "ConHub").bearer_auth(&token).send().await;
        match resp {
            Ok(r) if r.status().is_success() => {
                let v: serde_json::Value = r.json().await.unwrap_or(json!({}));
                let name = v["name"].as_str().unwrap_or("");
                let full_name = v["full_name"].as_str().unwrap_or("");
                return Ok(HttpResponse::Ok().json(json!({"provider": "github", "name": name, "full_name": full_name})));
            }
            Ok(r) => return Ok(HttpResponse::BadRequest().json(json!({"error": format!("GitHub API error: {}", r.status())}))),
            Err(e) => return Ok(HttpResponse::BadRequest().json(json!({"error": format!("GitHub request failed: {}", e)}))),
        }
    } else {
        let parts: Vec<&str> = url.trim_end_matches('/').split('/').collect();
        let (workspace, repo) = (parts.get(parts.len()-2).unwrap_or(&""), parts.get(parts.len()-1).unwrap_or(&""));
        let api_url = format!("https://api.bitbucket.org/2.0/repositories/{}/{}", workspace, repo);
        let resp = client.get(&api_url).bearer_auth(&token).send().await;
        match resp {
            Ok(r) if r.status().is_success() => {
                let v: serde_json::Value = r.json().await.unwrap_or(json!({}));
                let name = v["name"].as_str().unwrap_or("");
                let full_name = v["full_name"].as_str().unwrap_or("");
                return Ok(HttpResponse::Ok().json(json!({"provider": "bitbucket", "name": name, "full_name": full_name})));
            }
            Ok(r) => return Ok(HttpResponse::BadRequest().json(json!({"error": format!("Bitbucket API error: {}", r.status())}))),
            Err(e) => return Ok(HttpResponse::BadRequest().json(json!({"error": format!("Bitbucket request failed: {}", e)}))),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct OAuthExchangeRequest { provider: String, code: String }

pub async fn oauth_exchange(
    req: HttpRequest,
    request: web::Json<OAuthExchangeRequest>,
    pool_opt: web::Data<Option<PgPool>>,
) -> Result<HttpResponse> {
    use crate::services::oauth::token_debug;
    
    // Generate debug_id for correlation
    let debug_id = Uuid::new_v4().to_string()[..8].to_string();
    
    // Extract correlation ID from header if present
    let correlation_id = req.headers()
        .get("x-correlation-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| debug_id.clone());
    
    let pool = match pool_opt.get_ref() { 
        Some(p) => p, 
        None => {
            tracing::error!("[OAuth Exchange][{}] Database pool unavailable", correlation_id);
            return Ok(HttpResponse::ServiceUnavailable().json(json!({"error": "Database service unavailable"})));
        }
    };
    
    use crate::services::middleware::extract_claims_from_request;
    let claims = match extract_claims_from_request(&req) { 
        Some(c) => c, 
        None => {
            tracing::warn!("[OAuth Exchange][{}] No JWT claims found - authentication required", correlation_id);
            return Ok(HttpResponse::Unauthorized().json(json!({"error": "Authentication required"})));
        }
    };
    let user_id: uuid::Uuid = claims.sub.parse().map_err(|_| actix_web::error::ErrorBadRequest("Invalid user id"))?;
    
    let code_preview = if request.code.len() > 10 { &request.code[..10] } else { &request.code };
    tracing::info!(
        "[OAuth Exchange][{}] 🔄 Starting exchange: provider={}, user_id={}, code_prefix={}...",
        correlation_id, request.provider, user_id, code_preview
    );

    let oauth_service = crate::services::oauth::OAuthService::new(pool.clone());
    let provider_enum = match request.provider.to_lowercase().as_str() {
        "google" => crate::services::oauth::OAuthProvider::Google,
        "microsoft" => crate::services::oauth::OAuthProvider::Microsoft,
        "github" => crate::services::oauth::OAuthProvider::GitHub,
        "bitbucket" => crate::services::oauth::OAuthProvider::Bitbucket,
        "gitlab" => crate::services::oauth::OAuthProvider::GitLab,
        _ => {
            tracing::warn!("[OAuth Exchange][{}] Unsupported provider: {}", correlation_id, request.provider);
            return Ok(HttpResponse::BadRequest().json(json!({"error": "Unsupported provider"})));
        }
    };

    let token = match oauth_service.exchange_code_for_token(provider_enum.clone(), &request.code).await {
        Ok(t) => {
            tracing::info!(
                "[OAuth Exchange][{}] ✅ Token exchange successful: provider={}, user_id={}, token_type={}, scope={:?}, has_refresh={}, token_debug={}",
                correlation_id, request.provider, user_id, t.token_type, t.scope, t.refresh_token.is_some(), token_debug(&t.access_token)
            );
            t
        },
        Err(e) => {
            tracing::error!(
                "[OAuth Exchange][{}] ❌ Token exchange FAILED: provider={}, user_id={}, error={}",
                correlation_id, request.provider, user_id, e
            );
            return Ok(HttpResponse::BadRequest().json(json!({"error": format!("Token exchange failed: {}", e)})));
        }
    };

    let (platform_user_id, email, name, avatar_url) = match oauth_service.get_user_info(provider_enum.clone(), &token.access_token).await {
        Ok(info) => {
            tracing::info!(
                "[OAuth Exchange][{}] ✅ User info fetched: provider={}, platform_user_id={}, email={}",
                correlation_id, request.provider, info.0, info.1
            );
            info
        },
        Err(e) => {
            tracing::error!(
                "[OAuth Exchange][{}] ❌ Failed to fetch user info: provider={}, error={}",
                correlation_id, request.provider, e
            );
            return Ok(HttpResponse::BadRequest().json(json!({"error": format!("Failed to fetch user info: {}", e)})));
        }
    };

    let scope = token.scope.clone().unwrap_or_default();
    let expires_at = token.expires_in.and_then(|s| chrono::Duration::from_std(std::time::Duration::from_secs(s as u64)).ok()).map(|d| (Utc::now() + d).timestamp());

    let connection_id = Uuid::new_v4();
    let now = Utc::now();
    let provider_str = format!("{}", provider_enum);
    
    tracing::info!(
        "[OAuth Exchange][{}] 💾 Storing connection: connection_id={}, user_id={}, platform={}, platform_user_id={}, scope={}, expires_at={:?}, token_debug={}",
        correlation_id, connection_id, user_id, provider_str, platform_user_id, scope, 
        expires_at.map(|ts| chrono::DateTime::<Utc>::from_timestamp(ts, 0)),
        token_debug(&token.access_token)
    );

    let result = sqlx::query(
        r#"
        INSERT INTO social_connections (
            id, user_id, platform, platform_user_id, username,
            access_token, refresh_token, token_expires_at, scope,
            is_active, created_at, updated_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12
        )
        ON CONFLICT (user_id, platform, platform_user_id) 
        DO UPDATE SET
            access_token = EXCLUDED.access_token,
            refresh_token = EXCLUDED.refresh_token,
            token_expires_at = EXCLUDED.token_expires_at,
            scope = EXCLUDED.scope,
            is_active = true,
            updated_at = EXCLUDED.updated_at
        RETURNING id
        "#
    )
    .bind(connection_id)
    .bind(user_id)
    .bind(provider_str)
    .bind(&platform_user_id)
    .bind(email.split('@').next().unwrap_or("user"))
    .bind(&token.access_token)
    .bind(token.refresh_token.as_ref())
    .bind(expires_at.map(|ts| chrono::DateTime::<Utc>::from_timestamp(ts, 0)))
    .bind(&scope)
    .bind(true)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await;

    let final_connection_id: Uuid = match result {
        Ok(row) => row.get("id"),
        Err(e) => {
            tracing::error!(
                "[OAuth Exchange][{}] ❌ DB INSERT FAILED: provider={}, user_id={}, error={}",
                correlation_id, request.provider, user_id, e
            );
            return Ok(HttpResponse::InternalServerError().json(json!({"error": "Failed to store social connection"})));
        }
    };

    tracing::info!(
        "[OAuth Exchange][{}] ✅✅ CONNECTION STORED SUCCESSFULLY: provider={}, user_id={}, connection_id={}, platform_user_id={}, scope={}, expires_at={:?}, token_debug={}",
        correlation_id, request.provider, user_id, final_connection_id, platform_user_id, scope,
        expires_at.map(|ts| chrono::DateTime::<Utc>::from_timestamp(ts, 0)),
        token_debug(&token.access_token)
    );

    Ok(HttpResponse::Ok().json(json!({
        "user_id": user_id,
        "is_new_user": false,
        "connection_id": final_connection_id,
        "debug_id": correlation_id
    })))
}

// ============================================================================
// INTERNAL SERVICE-TO-SERVICE ENDPOINTS
// These endpoints are used by other microservices to access user data securely
// ============================================================================

#[derive(serde::Serialize)]
pub struct InternalTokenResponse {
    pub access_token: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub refresh_token: Option<String>,
}

/// Internal endpoint for service-to-service token resolution
/// Used by data service to get GitHub tokens for repository sync
/// 
/// GET /internal/oauth/{provider}/token?user_id={uuid}
pub async fn internal_get_oauth_token(
    req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
    pool_opt: web::Data<Option<PgPool>>,
) -> Result<HttpResponse> {
    use crate::services::oauth::token_debug;
    
    // Extract correlation ID from header if present
    let correlation_id = req.headers()
        .get("x-correlation-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string()[..8].to_string());
    
    let pool = match pool_opt.get_ref() {
        Some(p) => p,
        None => {
            tracing::error!("[Internal Token][{}] Database pool unavailable", correlation_id);
            return Ok(HttpResponse::ServiceUnavailable().json(json!({
                "error": "Database service unavailable"
            })));
        }
    };

    let provider = path.into_inner().to_lowercase();
    let user_id_str = match query.get("user_id") {
        Some(id) => id,
        None => {
            tracing::warn!("[Internal Token][{}] Missing user_id query parameter", correlation_id);
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Missing user_id query parameter"
            })));
        }
    };

    let user_id: uuid::Uuid = match user_id_str.parse() {
        Ok(id) => id,
        Err(_) => {
            tracing::warn!("[Internal Token][{}] Invalid user_id format: {}", correlation_id, user_id_str);
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Invalid user_id format"
            })));
        }
    };
    
    tracing::info!(
        "[Internal Token][{}] 🔍 Looking up token: provider={}, user_id={}",
        correlation_id, provider, user_id
    );

    // Validate provider
    let valid_providers = ["github", "bitbucket", "gitlab", "google", "microsoft"];
    if !valid_providers.contains(&provider.as_str()) {
        tracing::warn!("[Internal Token][{}] Unsupported provider: {}", correlation_id, provider);
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": format!("Unsupported provider: {}", provider)
        })));
    }

    // First, let's see how many connections exist for this user/provider (for debugging)
    let count_result = sqlx::query(
        r#"SELECT COUNT(*) as cnt FROM social_connections WHERE user_id = $1 AND platform = $2"#
    )
    .bind(user_id)
    .bind(&provider)
    .fetch_one(pool)
    .await;
    
    let total_connections: i64 = count_result.map(|r| r.get::<i64, _>("cnt")).unwrap_or(0);
    
    // Query for active token - filter out expired tokens in SQL
    // Note: Some providers (like GitHub) don't have expiry, so we allow NULL token_expires_at
    let row = sqlx::query(
        r#"
        SELECT id, access_token, refresh_token, token_expires_at, scope, username, platform_user_id, updated_at, is_active
        FROM social_connections
        WHERE user_id = $1 
          AND platform = $2 
          AND is_active = true
          AND (token_expires_at IS NULL OR token_expires_at > NOW())
        ORDER BY updated_at DESC
        LIMIT 1
        "#
    )
    .bind(user_id)
    .bind(&provider)
    .fetch_optional(pool)
    .await;

    match row {
        Ok(Some(row)) => {
            let connection_id: Uuid = row.get("id");
            let access_token: String = row.get("access_token");
            let refresh_token: Option<String> = row.get("refresh_token");
            let expires_at: Option<chrono::DateTime<chrono::Utc>> = row.get("token_expires_at");
            let scope: Option<String> = row.get("scope");
            let username: String = row.get("username");
            let platform_user_id: String = row.get("platform_user_id");
            let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

            tracing::info!(
                "[Internal Token][{}] ✅ FOUND VALID TOKEN: provider={}, user_id={}, connection_id={}, platform_user_id={}, username={}, scope={:?}, expires_at={:?}, updated_at={}, total_connections={}, token_debug={}",
                correlation_id, provider, user_id, connection_id, platform_user_id, username, scope, expires_at, updated_at, total_connections, token_debug(&access_token)
            );

            Ok(HttpResponse::Ok().json(InternalTokenResponse {
                access_token,
                expires_at,
                refresh_token,
            }))
        }
        Ok(None) => {
            tracing::warn!(
                "[Internal Token][{}] ⚠️ NO VALID TOKEN FOUND: provider={}, user_id={}, total_connections={}",
                correlation_id, provider, user_id, total_connections
            );
            
            // Check if there's an expired token to give a better error message
            let expired_check = sqlx::query(
                r#"
                SELECT id, token_expires_at, scope, username, is_active, updated_at
                FROM social_connections
                WHERE user_id = $1 AND platform = $2
                ORDER BY updated_at DESC
                LIMIT 1
                "#
            )
            .bind(user_id)
            .bind(&provider)
            .fetch_optional(pool)
            .await;

            let (error_code, error_message) = match expired_check {
                Ok(Some(row)) => {
                    let connection_id: Uuid = row.get("id");
                    let expires_at: Option<chrono::DateTime<chrono::Utc>> = row.get("token_expires_at");
                    let is_active: bool = row.get("is_active");
                    let scope: Option<String> = row.get("scope");
                    let username: String = row.get("username");
                    let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");
                    
                    tracing::warn!(
                        "[Internal Token][{}] 📋 Found connection but not valid: connection_id={}, is_active={}, expires_at={:?}, scope={:?}, username={}, updated_at={}",
                        correlation_id, connection_id, is_active, expires_at, scope, username, updated_at
                    );
                    
                    if !is_active {
                        ("connection_inactive", format!("Your {} connection is inactive. Please reconnect in Social Connections.", provider))
                    } else if expires_at.map(|e| e < chrono::Utc::now()).unwrap_or(false) {
                        ("token_expired", format!("Your {} connection has expired. Please reconnect in Social Connections.", provider))
                    } else {
                        ("no_connection", format!("No active {} connection found for user", provider))
                    }
                }
                Ok(None) => {
                    tracing::warn!(
                        "[Internal Token][{}] 📋 No connection exists at all for provider={}, user_id={}",
                        correlation_id, provider, user_id
                    );
                    ("no_connection", format!("No {} connection found. Please connect in Social Connections first.", provider))
                }
                Err(e) => {
                    tracing::error!("[Internal Token][{}] DB error checking expired: {}", correlation_id, e);
                    ("no_connection", format!("No active {} connection found for user", provider))
                }
            };

            Ok(HttpResponse::NotFound().json(json!({
                "code": error_code,
                "error": error_message
            })))
        }
        Err(e) => {
            tracing::error!("[Internal Token][{}] ❌ DB QUERY FAILED: provider={}, user_id={}, error={}", correlation_id, provider, user_id, e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "code": "internal_error",
                "error": "Failed to fetch OAuth token"
            })))
        }
    }
}

/// Internal endpoint to check if a user has a valid connection for a provider
/// GET /internal/oauth/{provider}/status?user_id={uuid}
pub async fn internal_check_oauth_status(
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
    pool_opt: web::Data<Option<PgPool>>,
) -> Result<HttpResponse> {
    let pool = match pool_opt.get_ref() {
        Some(p) => p,
        None => return Ok(HttpResponse::ServiceUnavailable().json(json!({
            "error": "Database service unavailable"
        }))),
    };

    let provider = path.into_inner().to_lowercase();
    let user_id_str = match query.get("user_id") {
        Some(id) => id,
        None => return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Missing user_id query parameter"
        }))),
    };

    let user_id: uuid::Uuid = match user_id_str.parse() {
        Ok(id) => id,
        Err(_) => return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Invalid user_id format"
        }))),
    };

    let row = sqlx::query(
        r#"
        SELECT id, username, token_expires_at, updated_at
        FROM social_connections
        WHERE user_id = $1 AND platform = $2 AND is_active = true
        ORDER BY updated_at DESC
        LIMIT 1
        "#
    )
    .bind(user_id)
    .bind(&provider)
    .fetch_optional(pool)
    .await;

    match row {
        Ok(Some(row)) => {
            let connection_id: uuid::Uuid = row.get("id");
            let username: String = row.get("username");
            let expires_at: Option<chrono::DateTime<chrono::Utc>> = row.get("token_expires_at");
            let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

            let is_expired = expires_at
                .map(|exp| exp < chrono::Utc::now())
                .unwrap_or(false);

            Ok(HttpResponse::Ok().json(json!({
                "connected": true,
                "connection_id": connection_id,
                "username": username,
                "is_expired": is_expired,
                "expires_at": expires_at,
                "updated_at": updated_at
            })))
        }
        Ok(None) => Ok(HttpResponse::Ok().json(json!({
            "connected": false
        }))),
        Err(e) => {
            tracing::error!("Failed to check OAuth status: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to check OAuth status"
            })))
        }
    }
}

/// Public endpoint that returns connections for the current user (if authenticated)
/// or an empty list (if not authenticated). Used as a fallback when auth middleware
/// is not applied to this route.
pub async fn list_auth_connections_current(
    req: HttpRequest,
    pool_opt: web::Data<Option<PgPool>>,
) -> Result<HttpResponse> {
    let pool = match pool_opt.get_ref() {
        Some(p) => p,
        None => return Ok(HttpResponse::Ok().json(json!({"success": true, "data": []}))),
    };

    // Try to extract user_id from claims if present
    use crate::services::middleware::{extract_claims_from_request, extract_user_id_from_request_async};
    
    let user_id = if let Some(claims) = extract_claims_from_request(&req) {
        // Try UUID parse first, then async lookup
        if let Ok(uuid) = claims.sub.parse::<uuid::Uuid>() {
            Some(uuid)
        } else {
            extract_user_id_from_request_async(&req, pool).await
        }
    } else {
        None
    };

    let user_id = match user_id {
        Some(id) => id,
        None => {
            // No authenticated user - return empty list
            return Ok(HttpResponse::Ok().json(json!({"success": true, "data": []})));
        }
    };

    let rows = sqlx::query(
        r#"
        SELECT id, platform, username, is_active, created_at AS connected_at, last_sync
        FROM social_connections
        WHERE user_id = $1
        ORDER BY updated_at DESC
        "#
    )
    .bind(user_id)
    .fetch_all(pool)
    .await;

    match rows {
        Ok(list) => {
            let data: Vec<SocialConnectionDto> = list.into_iter().map(|row| SocialConnectionDto {
                id: row.get("id"),
                platform: row.get("platform"),
                username: row.get("username"),
                is_active: row.get("is_active"),
                connected_at: row.get("connected_at"),
                last_sync: row.get("last_sync"),
            }).collect();
            Ok(HttpResponse::Ok().json(json!({"success": true, "data": data})))
        },
        Err(e) => Ok(HttpResponse::InternalServerError().json(json!({"error": format!("Failed to list connections: {}", e)})))
    }
}