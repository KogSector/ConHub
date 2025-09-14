use actix_web::{web, HttpResponse, Result};
use std::collections::HashMap;
use std::sync::Mutex;
use crate::models::{ApiResponse, UserSettings, ProfileSettings, NotificationSettings, SecuritySettings, UpdateSettingsRequest};

// In-memory storage for settings (replace with database in production)
lazy_static::lazy_static! {
    static ref SETTINGS_STORE: Mutex<HashMap<String, UserSettings>> = Mutex::new(HashMap::new());
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
            // Return default settings for new users
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
    
    // Update only provided fields
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

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/settings")
            .route("/{user_id}", web::get().to(get_settings))
            .route("/{user_id}", web::put().to(update_settings))
    );
}