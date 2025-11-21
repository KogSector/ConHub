use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

use crate::services::{Auth0Service, Auth0Config, UserService, SecurityService};
use conhub_models::auth::{AuthResponse, UserProfile, RegisterRequest};

/// Extract Bearer token from Authorization header
fn extract_bearer_token(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(|s| s.to_string())
}

/// POST /api/auth/auth0/exchange
/// 
/// Exchange Auth0 access token for ConHub JWT
/// 
/// Headers:
///   Authorization: Bearer <auth0_access_token>
/// 
/// Response:
///   AuthResponse with ConHub JWT, refresh token, and user profile
pub async fn auth0_exchange(
    req: HttpRequest,
    pool_opt: web::Data<Option<PgPool>>,
) -> Result<HttpResponse> {
    // Check if database pool is available
    let pool = match pool_opt.get_ref() {
        Some(p) => p,
        None => {
            tracing::error!("Database pool not available for Auth0 exchange");
            return Ok(HttpResponse::ServiceUnavailable().json(json!({
                "error": "Database service unavailable"
            })));
        }
    };

    // Extract Auth0 token from Authorization header
    let auth0_token = match extract_bearer_token(&req) {
        Some(token) => token,
        None => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Missing or invalid Authorization header",
                "message": "Expected 'Authorization: Bearer <auth0_access_token>'"
            })));
        }
    };

    // Initialize Auth0 service
    let auth0_config = match Auth0Config::from_env() {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("Failed to load Auth0 configuration: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Auth0 configuration error",
                "message": "Auth0 is not properly configured on the server"
            })));
        }
    };

    let auth0_service = Auth0Service::new(auth0_config);

    // Verify Auth0 token
    let claims = match auth0_service.verify_token(&auth0_token).await {
        Ok(claims) => claims,
        Err(e) => {
            tracing::warn!("Auth0 token verification failed: {}", e);
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Invalid Auth0 token",
                "message": format!("{}", e)
            })));
        }
    };

    // Extract user info from claims
    let (auth0_sub, email_opt, name_opt, picture_opt) = auth0_service.extract_user_info(&claims);

    // Email is required
    let email = match email_opt {
        Some(e) => e,
        None => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Email not provided",
                "message": "Auth0 token must include email claim"
            })));
        }
    };

    let name = name_opt.unwrap_or_else(|| {
        email.split('@').next().unwrap_or("User").to_string()
    });

    // Initialize services
    let user_service = match UserService::new(pool.clone()).await {
        Ok(service) => service,
        Err(e) => {
            tracing::error!("Failed to initialize user service: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Service initialization failed"
            })));
        }
    };

    let security_service = match SecurityService::new(pool.clone()).await {
        Ok(service) => service,
        Err(e) => {
            tracing::error!("Failed to initialize security service: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Service initialization failed"
            })));
        }
    };

    // Find or create user based on auth0_sub
    let user = match find_or_create_auth0_user(
        pool,
        &user_service,
        &auth0_sub,
        &email,
        &name,
        picture_opt.as_deref(),
    ).await {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("Failed to find/create Auth0 user: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to create or retrieve user",
                "message": format!("{}", e)
            })));
        }
    };

    // Update last login
    if let Err(e) = user_service.update_last_login(user.id).await {
        tracing::warn!("Failed to update last login for user {}: {}", user.id, e);
    }

    // Generate ConHub JWT tokens
    let session_id = Uuid::new_v4();
    let remember_me = false;
    let (token, refresh_token, expires_at, _refresh_expires) = match security_service
        .generate_jwt_token(&user, session_id, remember_me)
        .await
    {
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

    tracing::info!("Auth0 exchange successful for sub: {}", auth0_sub);
    Ok(HttpResponse::Ok().json(auth_response))
}

/// Find existing user by auth0_sub or create new user
async fn find_or_create_auth0_user(
    pool: &PgPool,
    user_service: &UserService,
    auth0_sub: &str,
    email: &str,
    name: &str,
    avatar_url: Option<&str>,
) -> anyhow::Result<conhub_models::auth::User> {
    // First, try to find user by auth0_sub
    let existing_by_sub = sqlx::query_as::<_, (Uuid,)>(
        "SELECT id FROM users WHERE auth0_sub = $1"
    )
    .bind(auth0_sub)
    .fetch_optional(pool)
    .await?;

    if let Some((user_id,)) = existing_by_sub {
        tracing::info!("Found existing user by auth0_sub: {}", user_id);
        return user_service.get_user_by_id(user_id).await?
            .ok_or_else(|| anyhow::anyhow!("User not found after lookup"));
    }

    // Try to find by email (user might exist from email/password registration)
    let existing_by_email = user_service.find_by_email(email).await;

    match existing_by_email {
        Ok(mut user) => {
            // Link this user to Auth0
            tracing::info!("Linking existing user {} to Auth0 sub: {}", user.id, auth0_sub);
            
            sqlx::query(
                "UPDATE users SET auth0_sub = $1, updated_at = $2 WHERE id = $3"
            )
            .bind(auth0_sub)
            .bind(Utc::now())
            .bind(user.id)
            .execute(pool)
            .await?;

            // Update the user struct
            user.updated_at = Utc::now();
            Ok(user)
        }
        Err(_) => {
            // Create new user
            tracing::info!("Creating new user for Auth0 sub: {}", auth0_sub);
            
            let register_request = RegisterRequest {
                email: email.to_string(),
                password: Uuid::new_v4().to_string(), // Random password (won't be used)
                name: name.to_string(),
                avatar_url: avatar_url.map(|s| s.to_string()),
                organization: None,
            };

            let mut new_user = user_service.create_user(&register_request).await?;

            // Link to Auth0
            sqlx::query(
                "UPDATE users SET auth0_sub = $1, is_verified = true, email_verified_at = $2, updated_at = $3 WHERE id = $4"
            )
            .bind(auth0_sub)
            .bind(Utc::now())
            .bind(Utc::now())
            .bind(new_user.id)
            .execute(pool)
            .await?;

            // Update the user struct
            new_user.is_verified = true;
            new_user.email_verified_at = Some(Utc::now());
            new_user.updated_at = Utc::now();

            Ok(new_user)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bearer_token() {
        use actix_web::test::TestRequest;

        let req = TestRequest::default()
            .insert_header(("Authorization", "Bearer test_token_123"))
            .to_http_request();

        let token = extract_bearer_token(&req);
        assert_eq!(token, Some("test_token_123".to_string()));

        let req_no_bearer = TestRequest::default()
            .insert_header(("Authorization", "test_token_123"))
            .to_http_request();

        let token_no_bearer = extract_bearer_token(&req_no_bearer);
        assert_eq!(token_no_bearer, None);
    }
}
