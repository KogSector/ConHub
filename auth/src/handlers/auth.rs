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
};

pub async fn login(
    request: web::Json<LoginRequest>,
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    // Initialize SecurityService for rate limiting
    let security_service = match crate::services::security::SecurityService::new(pool.get_ref().clone()).await {
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

    let user_service = match UserService::new(pool.get_ref().clone()).await {
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
    pool: web::Data<PgPool>,
) -> Result<HttpResponse> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    let email = &request.email;
    tracing::info!("Password reset requested for email: {}", email);
    
    let user_service = match UserService::new(pool.get_ref().clone()).await {
        Ok(service) => service,
        Err(e) => {
            tracing::error!("Failed to initialize user service: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({"error": "Service initialization failed"})));
        }
    };
    let password_reset_service = crate::services::password_reset::PasswordResetService::new(pool.get_ref().clone());
    
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
    pool: web::Data<PgPool>,
) -> Result<HttpResponse> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    let token = &request.token;
    let new_password = &request.new_password;
    
    tracing::info!("Password reset attempted for token: {}", token);
    
    let password_reset_service = crate::services::password_reset::PasswordResetService::new(pool.get_ref().clone());
    
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
    let security_service = match crate::services::security::SecurityService::new(pool.get_ref().clone()).await {
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

    let user_service = match UserService::new(pool.get_ref().clone()).await {
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
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    // Initialize SecurityService for rate limiting
    let security_service = match crate::services::security::SecurityService::new(pool.get_ref().clone()).await {
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

    // Check rate limit for registration attempts (3 attempts per minute)
    if !security_service.check_rate_limit(&client_ip, "register", 3, 1).await
        .unwrap_or(false) {
        tracing::warn!("Rate limit exceeded for registration attempt from IP: {}", client_ip);
        return Ok(HttpResponse::TooManyRequests().json(json!({
            "error": "Too many registration attempts. Please try again later."
        })));
    }

    let user_service = match UserService::new(pool.get_ref().clone()).await {
        Ok(service) => service,
        Err(e) => {
            tracing::error!("Failed to initialize user service: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({"error": "Service initialization failed"})));
        }
    };
    
    
    let new_user = match user_service.create_user(&request).await {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("Failed to create user: {}", e);
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Failed to create user",
                "details": format!("{}", e)
            })));
        }
    };

    // Generate secure tokens using SecurityService
    let session_id = Uuid::new_v4();
    let remember_me = false; // Could be extracted from request if needed
    let (token, refresh_token, expires_at, _refresh_expires) = match security_service.generate_jwt_token(&new_user, session_id, remember_me).await {
        Ok(tokens) => tokens,
        Err(e) => {
            tracing::error!("Failed to generate JWT tokens for user {}: {}", new_user.id, e);
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

    Ok(HttpResponse::Created().json(auth_response))
}

pub async fn verify_token() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "valid": true
    })))
}

pub async fn logout(
    req: HttpRequest,
    request: web::Json<LogoutRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse> {
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

    let security_service = match SecurityService::new(pool.get_ref().clone()).await {
        Ok(service) => std::sync::Arc::new(service),
        Err(e) => {
            tracing::error!("Failed to initialize security service: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Service initialization failed"
            })));
        }
    };

    let session_service = SessionService::new(
        pool.get_ref().clone(),
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
    pool: web::Data<PgPool>,
) -> Result<HttpResponse> {
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
    let user_service = match UserService::new(pool.get_ref().clone()).await {
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
    pool: web::Data<PgPool>,
    redis_client: web::Data<redis::Client>,
) -> Result<HttpResponse> {
    // Initialize services
    let security_service = match SecurityService::new(pool.get_ref().clone()).await {
        Ok(service) => std::sync::Arc::new(service),
        Err(e) => {
            tracing::error!("Failed to initialize security service: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Service initialization failed"
            })));
        }
    };

    let session_service = SessionService::new(
        pool.get_ref().clone(),
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

pub async fn get_profile(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    
    let user_service = match UserService::new(pool.get_ref().clone()).await {
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

pub async fn list_users(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    let user_service = match UserService::new(pool.get_ref().clone()).await {
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
    pool: web::Data<PgPool>,
) -> Result<HttpResponse> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    let user_service = match UserService::new(pool.get_ref().clone()).await {
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
    .fetch_one(pool.get_ref())
    .await;

    let final_connection_id = match result {
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
    );
}