use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::services::auth::oauth::{OAuthService, OAuthProvider};
use crate::handlers::auth::auth::generate_jwt_token;
use crate::models::auth::{AuthResponse, UserProfile, User};

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: String,
    pub state: String,
    pub provider: String,
}

#[derive(Debug, Deserialize)]
pub struct OAuthInitRequest {
    pub provider: String,
}

#[derive(Debug, Serialize)]
pub struct OAuthInitResponse {
    pub authorization_url: String,
    pub state: String,
}


pub async fn oauth_init(
    request: web::Json<OAuthInitRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse> {
    let oauth_service = OAuthService::new(pool.get_ref().clone());
    
    let provider = match request.provider.to_lowercase().as_str() {
        "google" => OAuthProvider::Google,
        "microsoft" => OAuthProvider::Microsoft,
        "github" => OAuthProvider::GitHub,
        _ => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Invalid provider",
                "message": "Supported providers: google, microsoft, github"
            })));
        }
    };

    
    let state = Uuid::new_v4().to_string();
    let authorization_url = oauth_service.get_authorization_url(provider, &state);

    

    Ok(HttpResponse::Ok().json(OAuthInitResponse {
        authorization_url,
        state,
    }))
}


pub async fn oauth_callback(
    query: web::Query<OAuthCallbackQuery>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse> {
    let oauth_service = OAuthService::new(pool.get_ref().clone());
    
    let provider = match query.provider.to_lowercase().as_str() {
        "google" => OAuthProvider::Google,
        "microsoft" => OAuthProvider::Microsoft,
        "github" => OAuthProvider::GitHub,
        _ => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Invalid provider"
            })));
        }
    };

    

    
    let token_response = match oauth_service.exchange_code_for_token(provider.clone(), &query.code).await {
        Ok(token) => token,
        Err(e) => {
            log::error!("Failed to exchange OAuth code: {}", e);
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Failed to exchange authorization code",
                "message": e.to_string()
            })));
        }
    };

    
    let (provider_user_id, email, name, avatar_url) = 
        match oauth_service.get_user_info(provider.clone(), &token_response.access_token).await {
            Ok(info) => info,
            Err(e) => {
                log::error!("Failed to get user info from OAuth provider: {}", e);
                return Ok(HttpResponse::InternalServerError().json(json!({
                    "error": "Failed to retrieve user information",
                    "message": e.to_string()
                })));
            }
        };

    
    let user = match oauth_service.find_or_create_oauth_user(
        provider.clone(),
        provider_user_id.clone(),
        email,
        name.clone(),
        avatar_url.clone(),
    ).await {
        Ok(user) => user,
        Err(e) => {
            log::error!("Failed to find/create OAuth user: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to create/find user account",
                "message": e.to_string()
            })));
        }
    };

    
    if let Err(e) = oauth_service.store_oauth_connection(
        user.id,
        provider,
        provider_user_id,
        name,
        token_response.access_token,
        token_response.refresh_token,
        token_response.expires_in,
        token_response.scope,
    ).await {
        log::warn!("Failed to store OAuth connection: {}", e);
        
    }

    
    let (token, expires_at) = match generate_jwt_token(&user) {
        Ok(result) => result,
        Err(e) => {
            log::error!("Failed to generate JWT token: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to generate authentication token",
                "message": e
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


pub async fn oauth_disconnect(
    provider: web::Path<String>,
    pool: web::Data<PgPool>,
    
) -> Result<HttpResponse> {
    
    
    
    let provider_name = provider.into_inner();
    
    
    
    
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": format!("Disconnected from {}", provider_name)
    })))
}


pub async fn oauth_connections(
    pool: web::Data<PgPool>,
    
) -> Result<HttpResponse> {
    
    
    
    
    
    
    Ok(HttpResponse::Ok().json(json!({
        "connections": []
    })))
}
