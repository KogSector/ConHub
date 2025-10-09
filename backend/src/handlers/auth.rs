use actix_web::{web, HttpResponse, Result};
use serde_json::json;
use uuid::Uuid;
use chrono::{Utc, Duration};
use validator::Validate;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, Header, EncodingKey};

use crate::models::auth::*;
use crate::services::email_service::EmailService;
use crate::services::password_reset_service::PASSWORD_RESET_SERVICE;

// Mock user for development - replace with database calls
fn get_mock_user() -> User {
    User {
        id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        email: "admin@conhub.dev".to_string(),
        password_hash: hash("password123", DEFAULT_COST).unwrap(),
        name: "ConHub Admin".to_string(),
        avatar_url: None,
        organization: Some("ConHub Development".to_string()),
        role: UserRole::Admin,
        subscription_tier: SubscriptionTier::Enterprise,
        is_verified: true,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login_at: Some(Utc::now()),
    }
}

pub async fn login(request: web::Json<LoginRequest>) -> Result<HttpResponse> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    // Mock authentication - replace with database lookup
    let mock_user = get_mock_user();
    
    if request.email != mock_user.email {
        return Ok(HttpResponse::Unauthorized().json(json!({
            "error": "Invalid credentials"
        })));
    }

    if !verify(&request.password, &mock_user.password_hash).unwrap_or(false) {
        return Ok(HttpResponse::Unauthorized().json(json!({
            "error": "Invalid credentials"
        })));
    }

    // Generate JWT token
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "conhub_super_secret_jwt_key_2024_development_only".to_string());
    
    let expires_at = Utc::now() + Duration::hours(24);
    let claims = Claims {
        sub: mock_user.id.to_string(),
        email: mock_user.email.clone(),
        roles: vec![format!("{:?}", mock_user.role).to_lowercase()],
        exp: expires_at.timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
        iss: "conhub".to_string(),
        aud: "conhub-frontend".to_string(),
        session_id: Uuid::new_v4().to_string(),
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    ) {
        Ok(token) => token,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to generate token"
            })));
        }
    };

    let user_profile = UserProfile::from(mock_user);
    let auth_response = AuthResponse {
        user: user_profile,
        token,
        expires_at,
    };

    Ok(HttpResponse::Ok().json(auth_response))
}

pub async fn forgot_password(request: web::Json<ForgotPasswordRequest>) -> Result<HttpResponse> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    let email = &request.email;
    log::info!("Password reset requested for email: {}", email);
    
    // For security reasons, always return success regardless of whether email exists
    // This prevents email enumeration attacks
    
    // Generate reset token
    match PASSWORD_RESET_SERVICE.generate_reset_token(email) {
        Ok(token) => {
            // Send email asynchronously
            let email_clone = email.clone();
            let token_clone = token.clone();
            
            tokio::spawn(async move {
                match EmailService::new() {
                    Ok(email_service) => {
                        if let Err(e) = email_service.send_password_reset_email(&email_clone, &token_clone).await {
                            log::error!("Failed to send password reset email to {}: {}", email_clone, e);
                        } else {
                            log::info!("Password reset email sent successfully to: {}", email_clone);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to initialize email service: {}", e);
                    }
                }
            });
        }
        Err(e) => {
            log::error!("Failed to generate reset token for {}: {}", email, e);
            // Still return success for security reasons
        }
    }

    Ok(HttpResponse::Ok().json(json!({
        "message": "If an account with that email exists, we've sent a password reset link.",
        "success": true
    })))
}

pub async fn reset_password(request: web::Json<ResetPasswordRequest>) -> Result<HttpResponse> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    let token = &request.token;
    let new_password = &request.new_password;
    
    log::info!("Password reset attempted for token: {}", token);
    
    // Validate the reset token
    let email = match PASSWORD_RESET_SERVICE.validate_token(token) {
        Ok(email) => email,
        Err(e) => {
            log::warn!("Invalid password reset token: {}", e);
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Invalid or expired reset token",
                "details": e
            })));
        }
    };
    
    // Hash the new password
    let new_password_hash = match hash(new_password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(e) => {
            log::error!("Failed to hash password: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to process password reset"
            })));
        }
    };

    // In a real implementation, update the user's password in the database
    // For now, we'll just log the successful reset
    log::info!("Password successfully reset for email: {}", email);
    
    // TODO: Update user password in database
    // let updated_user = update_user_password(&email, &new_password_hash).await?;

    Ok(HttpResponse::Ok().json(json!({
        "message": "Password has been reset successfully. You can now log in with your new password.",
        "success": true
    })))
}

pub async fn register(request: web::Json<RegisterRequest>) -> Result<HttpResponse> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    // For development, create a new user with the provided details
    let password_hash = match hash(&request.password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to hash password"
            })));
        }
    };

    let new_user = User {
        id: Uuid::new_v4(),
        email: request.email.clone(),
        password_hash,
        name: request.name.clone(),
        avatar_url: request.avatar_url.clone(),
        organization: request.organization.clone(),
        role: UserRole::User,
        subscription_tier: SubscriptionTier::Free,
        is_verified: false,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login_at: None,
    };

    // Generate JWT token
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "conhub_super_secret_jwt_key_2024_development_only".to_string());
    
    let expires_at = Utc::now() + Duration::hours(24);
    let claims = Claims {
        sub: new_user.id.to_string(),
        email: new_user.email.clone(),
        roles: vec![format!("{:?}", new_user.role).to_lowercase()],
        exp: expires_at.timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
        iss: "conhub".to_string(),
        aud: "conhub-frontend".to_string(),
        session_id: Uuid::new_v4().to_string(),
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    ) {
        Ok(token) => token,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to generate token"
            })));
        }
    };

    let user_profile = UserProfile::from(new_user);
    let auth_response = AuthResponse {
        user: user_profile,
        token,
        expires_at,
    };

    Ok(HttpResponse::Created().json(auth_response))
}

pub async fn verify_token() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "valid": true
    })))
}

pub async fn get_profile() -> Result<HttpResponse> {
    let mock_user = get_mock_user();
    let user_profile = UserProfile::from(mock_user);
    Ok(HttpResponse::Ok().json(user_profile))
}

pub async fn test_email(request: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let email = request.get("email")
        .and_then(|e| e.as_str())
        .unwrap_or("test@conhub.dev");
    
    log::info!("Testing email service for: {}", email);
    
    match EmailService::new() {
        Ok(email_service) => {
            let test_token = "test-token-123";
            match email_service.send_password_reset_email(email, test_token).await {
                Ok(_) => {
                    Ok(HttpResponse::Ok().json(json!({
                        "success": true,
                        "message": "Test email sent successfully",
                        "email": email
                    })))
                }
                Err(e) => {
                    log::error!("Failed to send test email: {}", e);
                    Ok(HttpResponse::InternalServerError().json(json!({
                        "success": false,
                        "error": "Failed to send test email",
                        "details": e
                    })))
                }
            }
        }
        Err(e) => {
            log::error!("Failed to initialize email service: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Failed to initialize email service",
                "details": e
            })))
        }
    }
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
            .route("/test-email", web::post().to(test_email))
    );
}