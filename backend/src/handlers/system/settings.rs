use actix_web::{web, HttpResponse, Result};
use std::collections::HashMap;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use crate::models::{ApiResponse, UserSettings, ProfileSettings, NotificationSettings, SecuritySettings, UpdateSettingsRequest};


lazy_static::lazy_static! {
    static ref SETTINGS_STORE: Mutex<HashMap<String, UserSettings>> = Mutex::new(HashMap::new());
    static ref API_TOKENS_STORE: Mutex<HashMap<String, Vec<ApiToken>>> = Mutex::new(HashMap::new());
    static ref WEBHOOKS_STORE: Mutex<HashMap<String, Vec<Webhook>>> = Mutex::new(HashMap::new());
    static ref TEAM_MEMBERS_STORE: Mutex<HashMap<String, Vec<TeamMember>>> = Mutex::new(HashMap::new());
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ApiToken {
    pub id: String,
    pub name: String,
    pub token: String,
    pub permissions: Vec<String>,
    pub created_at: String,
    pub last_used: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Webhook {
    pub id: String,
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
    pub status: String,
    pub created_at: String,
    pub last_delivery: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TeamMember {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub joined_date: String,
    pub last_active: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateApiTokenRequest {
    pub name: String,
    pub permissions: Vec<String>,
}

#[derive(Deserialize)]
pub struct CreateWebhookRequest {
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
}

#[derive(Deserialize)]
pub struct InviteTeamMemberRequest {
    pub email: String,
    pub role: String,
}

pub async fn get_settings(path: web::Path<String>) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    let store = SETTINGS_STORE.lock().unwrap();
    
    match store.get(&user_id) {
        Some(settings) => Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: "Settings retrieved successfully".to_string(),
            data: Some(settings.clone()),
            error: None,
        })),
        None => {
            
            let default_settings = UserSettings {
                user_id: user_id.clone(),
                profile: ProfileSettings {
                    first_name: "John".to_string(),
                    last_name: "Doe".to_string(),
                    email: "john.doe@example.com".to_string(),
                    bio: Some("Full-stack developer passionate about building scalable applications.".to_string()),
                    location: Some("San Francisco, CA".to_string()),
                    website: None,
                    social_links: HashMap::new(),
                },
                notifications: NotificationSettings {
                    email_notifications: true,
                    push_notifications: true,
                    security_alerts: true,
                },
                security: SecuritySettings {
                    two_factor_enabled: false,
                    session_timeout: 3600,
                },
            };
            
            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: "Default settings returned".to_string(),
                data: Some(default_settings),
                error: None,
            }))
        }
    }
}

pub async fn update_settings(
    path: web::Path<String>,
    req: web::Json<UpdateSettingsRequest>
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    let mut store = SETTINGS_STORE.lock().unwrap();
    
    let mut settings = store.get(&user_id).cloned().unwrap_or_else(|| UserSettings {
        user_id: user_id.clone(),
        profile: ProfileSettings {
            first_name: "".to_string(),
            last_name: "".to_string(),
            email: "".to_string(),
            bio: None,
            location: None,
            website: None,
            social_links: HashMap::new(),
        },
        notifications: NotificationSettings {
            email_notifications: true,
            push_notifications: true,
            security_alerts: true,
        },
        security: SecuritySettings {
            two_factor_enabled: false,
            session_timeout: 3600,
        },
    });
    
    
    if let Some(profile) = &req.profile {
        settings.profile = profile.clone();
    }
    if let Some(notifications) = &req.notifications {
        settings.notifications = notifications.clone();
    }
    if let Some(security) = &req.security {
        settings.security = security.clone();
    }
    
    store.insert(user_id, settings.clone());
    
    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "Settings updated successfully".to_string(),
        data: Some(settings),
        error: None,
    }))
}


pub async fn get_api_tokens(path: web::Path<String>) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    let store = API_TOKENS_STORE.lock().unwrap();
    
    let tokens = store.get(&user_id).cloned().unwrap_or_default();
    
    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "API tokens retrieved successfully".to_string(),
        data: Some(tokens),
        error: None,
    }))
}

pub async fn create_api_token(
    path: web::Path<String>,
    req: web::Json<CreateApiTokenRequest>
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    let mut store = API_TOKENS_STORE.lock().unwrap();
    
    let token = ApiToken {
        id: uuid::Uuid::new_v4().to_string(),
        name: req.name.clone(),
        token: format!("ch_{}_{}", req.permissions.join(""), uuid::Uuid::new_v4().to_string()[..8].to_string()),
        permissions: req.permissions.clone(),
        created_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        last_used: None,
    };
    
    let mut tokens = store.get(&user_id).cloned().unwrap_or_default();
    tokens.push(token.clone());
    store.insert(user_id, tokens);
    
    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "API token created successfully".to_string(),
        data: Some(token),
        error: None,
    }))
}

pub async fn delete_api_token(
    path: web::Path<(String, String)>
) -> Result<HttpResponse> {
    let (user_id, token_id) = path.into_inner();
    let mut store = API_TOKENS_STORE.lock().unwrap();
    
    if let Some(tokens) = store.get_mut(&user_id) {
        tokens.retain(|t| t.id != token_id);
    }
    
    Ok(HttpResponse::Ok().json(ApiResponse::<()> {
        success: true,
        message: "API token deleted successfully".to_string(),
        data: None,
        error: None,
    }))
}


pub async fn get_webhooks(path: web::Path<String>) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    let store = WEBHOOKS_STORE.lock().unwrap();
    
    let webhooks = store.get(&user_id).cloned().unwrap_or_default();
    
    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "Webhooks retrieved successfully".to_string(),
        data: Some(webhooks),
        error: None,
    }))
}

pub async fn create_webhook(
    path: web::Path<String>,
    req: web::Json<CreateWebhookRequest>
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    let mut store = WEBHOOKS_STORE.lock().unwrap();
    
    let webhook = Webhook {
        id: uuid::Uuid::new_v4().to_string(),
        name: req.name.clone(),
        url: req.url.clone(),
        events: req.events.clone(),
        status: "active".to_string(),
        created_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        last_delivery: None,
    };
    
    let mut webhooks = store.get(&user_id).cloned().unwrap_or_default();
    webhooks.push(webhook.clone());
    store.insert(user_id, webhooks);
    
    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "Webhook created successfully".to_string(),
        data: Some(webhook),
        error: None,
    }))
}

pub async fn delete_webhook(
    path: web::Path<(String, String)>
) -> Result<HttpResponse> {
    let (user_id, webhook_id) = path.into_inner();
    let mut store = WEBHOOKS_STORE.lock().unwrap();
    
    if let Some(webhooks) = store.get_mut(&user_id) {
        webhooks.retain(|w| w.id != webhook_id);
    }
    
    Ok(HttpResponse::Ok().json(ApiResponse::<()> {
        success: true,
        message: "Webhook deleted successfully".to_string(),
        data: None,
        error: None,
    }))
}


pub async fn get_team_members(path: web::Path<String>) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    let store = TEAM_MEMBERS_STORE.lock().unwrap();
    
    let members = store.get(&user_id).cloned().unwrap_or_default();
    
    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "Team members retrieved successfully".to_string(),
        data: Some(members),
        error: None,
    }))
}

pub async fn invite_team_member(
    path: web::Path<String>,
    req: web::Json<InviteTeamMemberRequest>
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    let mut store = TEAM_MEMBERS_STORE.lock().unwrap();
    
    let member = TeamMember {
        id: uuid::Uuid::new_v4().to_string(),
        name: req.email.split('@').next().unwrap_or("User").to_string(),
        email: req.email.clone(),
        role: req.role.clone(),
        status: "pending".to_string(),
        joined_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        last_active: None,
    };
    
    let mut members = store.get(&user_id).cloned().unwrap_or_default();
    members.push(member.clone());
    store.insert(user_id, members);
    
    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "Team member invited successfully".to_string(),
        data: Some(member),
        error: None,
    }))
}

pub async fn remove_team_member(
    path: web::Path<(String, String)>
) -> Result<HttpResponse> {
    let (user_id, member_id) = path.into_inner();
    let mut store = TEAM_MEMBERS_STORE.lock().unwrap();
    
    if let Some(members) = store.get_mut(&user_id) {
        members.retain(|m| m.id != member_id);
    }
    
    Ok(HttpResponse::Ok().json(ApiResponse::<()> {
        success: true,
        message: "Team member removed successfully".to_string(),
        data: None,
        error: None,
    }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/settings")
            .route("/{user_id}", web::get().to(get_settings))
            .route("/{user_id}", web::put().to(update_settings))
            .route("/{user_id}/api-tokens", web::get().to(get_api_tokens))
            .route("/{user_id}/api-tokens", web::post().to(create_api_token))
            .route("/{user_id}/api-tokens/{token_id}", web::delete().to(delete_api_token))
            .route("/{user_id}/webhooks", web::get().to(get_webhooks))
            .route("/{user_id}/webhooks", web::post().to(create_webhook))
            .route("/{user_id}/webhooks/{webhook_id}", web::delete().to(delete_webhook))
            .route("/{user_id}/team", web::get().to(get_team_members))
            .route("/{user_id}/team", web::post().to(invite_team_member))
            .route("/{user_id}/team/{member_id}", web::delete().to(remove_team_member))
    );
}