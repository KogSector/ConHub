use actix_web::{web, HttpResponse, Result};
use serde_json::json;
use uuid::Uuid;
use chrono::{Utc, Duration, DateTime};
use validator::Validate;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, Header, EncodingKey};
use sqlx::{PgPool, Row};

use conhub_models::auth::*;
use crate::services::password_reset::PASSWORD_RESET_SERVICE;
use crate::services::users::UserService;


pub fn generate_jwt_token(user: &User) -> Result<(String, DateTime<Utc>), String> {
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
    tracing::info!("Password reset requested for email: {}", email);
    
    let user_service = UserService::new(pool.get_ref().clone());
    
    
    let user_exists = match user_service.find_by_email(email).await {
        Ok(_) => true,
        Err(_) => false,
    };
    
    
    match PASSWORD_RESET_SERVICE.generate_reset_token(email) {
        Ok(_) => {},
        Err(_) => {}
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
    
    tracing::info!("Password reset attempted for token: {}", token);
    
    
    let email = match PASSWORD_RESET_SERVICE.validate_token(token) {
        Ok(email) => email,
        Err(e) => {
            tracing::warn!("Invalid password reset token: {}", e);
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Invalid or expired reset token",
                "details": e
            })));
        }
    };
    
    
    let new_password_hash = match hash(new_password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(e) => {
            tracing::error!("Failed to hash password: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to process password reset"
            })));
        }
    };

    let user_service = UserService::new(pool.get_ref().clone());
    
    
    let user = match user_service.find_by_email(&email).await {
        Ok(user) => user,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "User not found"
            })));
        }
    };
    
    
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
) -> Result<HttpResponse> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    let user_service = UserService::new(pool.get_ref().clone());
    
    
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

    
    let (token, expires_at) = match generate_jwt_token(&new_user) {
        Ok((token, expires_at)) => (token, expires_at),
        Err(e) => {
            tracing::error!("Failed to generate JWT token for user {}: {}", new_user.id, e);
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

pub async fn logout() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "message": "Logged out successfully"
    })))
}

pub async fn get_current_user() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "message": "Current user endpoint"
    })))
}

pub async fn refresh_token() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "message": "Token refreshed"
    })))
}

pub async fn get_profile(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    
    let user_service = UserService::new(pool.get_ref().clone());
    
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
    let user_service = UserService::new(pool.get_ref().clone());
    
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

    let user_service = UserService::new(pool.get_ref().clone());
    
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