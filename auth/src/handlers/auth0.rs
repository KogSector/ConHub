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

    // Email is strongly preferred, but some Auth0 access tokens for custom APIs
    // may not include a standard or namespaced email claim. In that case, we
    // synthesize a stable, valid email from the Auth0 subject so users can
    // still log in.
    let email = match email_opt {
        Some(e) => e,
        None => {
            let sanitized_sub = auth0_sub.replace('|', ".");
            let synthetic_email = format!("{}@auth0.local", sanitized_sub);
            tracing::warn!(
                "Auth0 token missing email claim for sub {}; using synthetic email {}",
                auth0_sub,
                synthetic_email,
            );
            synthetic_email
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

    // Find or create user based on auth0_sub using the dedicated helper on
    // UserService. This path is specific to SSO/Auth0 users and does not
    // enforce local password strength rules, since Auth0 is the identity
    // provider of record.
    let user = match user_service
        .find_or_create_by_auth0(
            &auth0_sub,
            &email,
            Some(&name),
            picture_opt.as_deref(),
        )
        .await
    {
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
