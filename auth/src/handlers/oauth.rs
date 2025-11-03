use actix_web::{web, HttpResponse, Result};
use actix_session::Session;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::services::{oauth::{OAuthService, OAuthProvider}, security::SecurityService};
use conhub_models::auth::{AuthResponse, UserProfile, User};

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

pub async fn oauth_login(
    provider: web::Path<String>,
    pool: web::Data<PgPool>,
    session: Session,
) -> Result<HttpResponse> {
    let oauth_service = OAuthService::new(pool.get_ref().clone());
    
    let provider_enum = match provider.to_lowercase().as_str() {
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
    session.insert("oauth_state", state.clone())?;
    let authorization_url = oauth_service.get_authorization_url(provider_enum, &state);

    Ok(HttpResponse::Ok().json(json!({
        "authorization_url": authorization_url,
        "state": state
    })))
}


pub async fn oauth_callback(
    query: web::Query<OAuthCallbackQuery>,
    pool: web::Data<PgPool>,
    session: Session,
) -> Result<HttpResponse> {
    let session_state = match session.get::<String>("oauth_state")? {
        Some(state) => state,
        None => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Invalid state"
            })));
        }
    };

    if session_state != query.state {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Invalid state"
        })));
    }

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
            tracing::error!("Failed to exchange OAuth code: {}", e);
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Failed to exchange authorization code",
                "message": format!("{}", e)
            })));
        }
    };

    
    let (provider_user_id, email, name, avatar_url): (String, String, String, Option<String>) = 
        match oauth_service.get_user_info(provider.clone(), &token_response.access_token).await {
            Ok(info) => info,
            Err(e) => {
                tracing::error!("Failed to get user info from OAuth provider: {}", e);
                return Ok(HttpResponse::InternalServerError().json(json!({
                    "error": "Failed to retrieve user information",
                    "message": format!("{}", e)
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
            tracing::error!("Failed to find/create OAuth user: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to create/find user account",
                "message": format!("{}", e)
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
        tracing::warn!("Failed to store OAuth connection: {}", e);
        
    }

    // Initialize SecurityService for token generation
    let security_service = SecurityService::new(pool.get_ref().clone()).await.map_err(|e| {
        tracing::error!("Failed to initialize SecurityService: {}", e);
        actix_web::error::ErrorInternalServerError("Service initialization failed")
    })?;

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


pub async fn oauth_disconnect(
    provider: web::Path<String>,
    _pool: web::Data<PgPool>,
    
) -> Result<HttpResponse> {
    
    
    
    let provider_name = provider.into_inner();
    
    
    
    
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": format!("Disconnected from {}", provider_name)
    })))
}


pub async fn oauth_connections(
    _pool: web::Data<PgPool>,
    
) -> Result<HttpResponse> {
    
    
    
    
    
    
    Ok(HttpResponse::Ok().json(json!({
        "connections": []
    })))
}
