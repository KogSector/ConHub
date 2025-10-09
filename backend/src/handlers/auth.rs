use actix_web::{web, HttpResponse, Result};
use serde_json::json;
use uuid::Uuid;
use chrono::{Utc, Duration, DateTime};
use validator::Validate;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, Header, EncodingKey};
use sqlx::{PgPool, Row};

use crate::models::auth::*;
// use crate::services::email_service::EmailService;
use crate::services::password_reset_service::PASSWORD_RESET_SERVICE;
use crate::services::user_service::UserService;

// Helper function to generate JWT token
fn generate_jwt_token(user: &User) -> Result<(String, DateTime<Utc>), String> {
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "conhub_super_secret_jwt_key_2024_development_only".to_string());
    
    let expires_at = Utc::now() + Duration::hours(24);
    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        roles: vec![format!("{:?}", user.role).to_lowercase()],
        exp: expires_at.timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
        iss: "conhub".to_string(),
        aud: "conhub-frontend".to_string(),
        session_id: Uuid::new_v4().to_string(),
    };

    match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    ) {
        Ok(token) => Ok((token, expires_at)),
        Err(_) => Err("Failed to generate token".to_string()),
    }
}

pub async fn login(
    request: web::Json<LoginRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    let user_service = UserService::new(pool.get_ref().clone());
    
    // Authenticate user with database
    let user = match user_service.verify_password(&request.email, &request.password).await {
        Ok(user) => user,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Invalid credentials"
            })));
        }
    };

    // Update last login timestamp
    if let Err(e) = user_service.update_last_login(user.id).await {
        log::warn!("Failed to update last login for user {}: {}", user.id, e);
    }

    // Generate JWT token
    let (token, expires_at) = match generate_jwt_token(&user) {
        Ok((token, expires_at)) => (token, expires_at),
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": e
            })));
        }
    };

    let user_profile = UserProfile::from(user);
    let auth_response = AuthResponse {
        user: user_profile,
        token,
        expires_at,
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
    log::info!("Password reset requested for email: {}", email);
    
    let user_service = UserService::new(pool.get_ref().clone());
    
    // Check if user exists in database
    let user_exists = match user_service.find_by_email(email).await {
        Ok(_) => true,
        Err(_) => false,
    };
    
    // Generate reset token
    match PASSWORD_RESET_SERVICE.generate_reset_token(email) {
        Ok(token) => {
            // Email functionality commented out for now
            log::info!("Password reset token generated: {} (email sending disabled)", token);
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

    let user_service = UserService::new(pool.get_ref().clone());
    
    // Find user by email and update password
    let user = match user_service.find_by_email(&email).await {
        Ok(user) => user,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "User not found"
            })));
        }
    };
    
    // Update user password in database
    if let Err(e) = user_service.update_password(user.id, new_password).await {
        log::error!("Failed to update password for user {}: {}", user.id, e);
        return Ok(HttpResponse::InternalServerError().json(json!({
            "error": "Failed to update password"
        })));
    }
    
    log::info!("Password successfully reset for email: {}", email);

    Ok(HttpResponse::Ok().json(json!({
        "message": "Password has been reset successfully. You can now log in with your new password.",
        "success": true
    })))
}

pub async fn register(
    request: web::Json<RegisterRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    let user_service = UserService::new(pool.get_ref().clone());
    
    // Create user in database
    let new_user = match user_service.create_user(&request).await {
        Ok(user) => user,
        Err(e) => {
            log::error!("Failed to create user: {}", e);
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Failed to create user",
                "details": e.to_string()
            })));
        }
    };

    // Generate JWT token
    let (token, expires_at) = match generate_jwt_token(&new_user) {
        Ok((token, expires_at)) => (token, expires_at),
        Err(e) => {
            log::error!("Failed to generate JWT token for user {}: {}", new_user.id, e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": e
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

pub async fn get_profile(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    // For now, return the first user in the database as a test
    let user_service = UserService::new(pool.get_ref().clone());
    
    match user_service.list_users(1, 0).await {
        Ok(users) => {
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
            log::error!("Failed to get user profile: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to get profile"
            })))
        }
    }
}

pub async fn list_users(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    let user_service = UserService::new(pool.get_ref().clone());
    
    match user_service.list_users(10, 0).await {
        Ok(users) => {
            let user_profiles: Vec<UserProfile> = users.into_iter()
                .map(UserProfile::from)
                .collect();
            
            Ok(HttpResponse::Ok().json(json!({
                "users": user_profiles,
                "count": user_profiles.len()
            })))
        }
        Err(e) => {
            log::error!("Failed to list users: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to list users"
            })))
        }
    }
}

pub async fn test_database(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    log::info!("Testing database connection and operations...");
    
    // Test 1: Basic connectivity
    let connectivity_test = match sqlx::query("SELECT 1 as test, NOW() as current_time")
        .fetch_one(pool.get_ref())
        .await 
    {
        Ok(row) => {
            let test_val: i32 = row.get("test");
            let current_time: chrono::DateTime<chrono::Utc> = row.get("current_time");
            json!({
                "status": "success",
                "test_value": test_val,
                "db_time": current_time
            })
        }
        Err(e) => {
            log::error!("Database connectivity test failed: {}", e);
            json!({
                "status": "failed",
                "error": e.to_string()
            })
        }
    };

    // Test 2: Check if users table exists and get structure
    let table_test = match sqlx::query(
        "SELECT column_name, data_type FROM information_schema.columns WHERE table_name = 'users' ORDER BY ordinal_position"
    ).fetch_all(pool.get_ref()).await {
        Ok(rows) => {
            let columns: Vec<serde_json::Value> = rows.into_iter().map(|row| {
                json!({
                    "column": row.get::<String, _>("column_name"),
                    "type": row.get::<String, _>("data_type")
                })
            }).collect();
            json!({
                "status": "success",
                "columns": columns
            })
        }
        Err(e) => {
            log::error!("Users table structure test failed: {}", e);
            json!({
                "status": "failed",
                "error": e.to_string()
            })
        }
    };

    // Test 3: Count existing users
    let user_count_test = match sqlx::query("SELECT COUNT(*) as count FROM users")
        .fetch_one(pool.get_ref())
        .await 
    {
        Ok(row) => {
            let count: i64 = row.get("count");
            json!({
                "status": "success",
                "user_count": count
            })
        }
        Err(e) => {
            log::error!("User count test failed: {}", e);
            json!({
                "status": "failed",
                "error": e.to_string()
            })
        }
    };

    Ok(HttpResponse::Ok().json(json!({
        "database_tests": {
            "connectivity": connectivity_test,
            "users_table": table_test,
            "user_count": user_count_test
        }
    })))
}

pub async fn test_email(request: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let email = request.get("email")
        .and_then(|e| e.as_str())
        .unwrap_or("test@conhub.dev");
    
    log::info!("Testing email service for: {}", email);
    
    // Email service commented out for now
    log::info!("Email test requested for: {} (email service disabled)", email);
    
    Ok(HttpResponse::Ok().json(json!({
        "message": "Email service is currently disabled",
        "email": email,
        "status": "disabled"
    })))
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
            .route("/test-db", web::get().to(test_database))
            .route("/test-email", web::post().to(test_email))
    );
}