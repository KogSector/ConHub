use std::sync::Arc;
use dashmap::DashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use crate::models::auth::Claims;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl UserSession {
    pub fn new(user_id: Uuid, email: String, ip_address: Option<String>, user_agent: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            user_id,
            email,
            created_at: now,
            expires_at: now + Duration::hours(24), // 24 hour session
            last_accessed: now,
            ip_address,
            user_agent,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn refresh(&mut self) {
        self.last_accessed = Utc::now();
        self.expires_at = Utc::now() + Duration::hours(24);
    }

    pub fn to_claims(&self) -> Claims {
        Claims {
            sub: self.user_id.to_string(),
            email: self.email.clone(),
            name: "User".to_string(), // You may want to store the actual name in the session
            role: crate::models::auth::UserRole::User, // Default role, you may want to store this in session
            subscription_tier: crate::models::auth::SubscriptionTier::Free, // Default tier
            exp: self.expires_at.timestamp() as usize,
            iat: self.created_at.timestamp() as usize,
        }
    }
}

#[derive(Clone)]
pub struct SessionService {
    // High-performance concurrent session store using DashMap
    sessions: Arc<DashMap<String, UserSession>>,
}

impl SessionService {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
        }
    }

    /// Create a new session and return session ID
    pub async fn create_session(&self, user_id: Uuid, email: String, ip_address: Option<String>, user_agent: Option<String>) -> String {
        let session_id = Uuid::new_v4().to_string();
        let session = UserSession::new(user_id, email, ip_address, user_agent);
        
        self.sessions.insert(session_id.clone(), session);
        
        session_id
    }

    /// Get session by ID
    #[allow(dead_code)]
    pub async fn get_session(&self, session_id: &str) -> Option<UserSession> {
        self.sessions.get(session_id).map(|entry| entry.value().clone())
    }

    /// Validate and refresh session
    pub async fn validate_session(&self, session_id: &str) -> Option<UserSession> {        
        if let Some(mut entry) = self.sessions.get_mut(session_id) {
            let session = entry.value_mut();
            if session.is_expired() {
                // Remove expired session
                drop(entry);
                self.sessions.remove(session_id);
                return None;
            }
            
            session.refresh();
            Some(session.clone())
        } else {
            None
        }
    }

    /// Remove session (logout)
    pub async fn remove_session(&self, session_id: &str) -> bool {
        self.sessions.remove(session_id).is_some()
    }

    /// Remove all sessions for a user
    #[allow(dead_code)]
    pub async fn remove_user_sessions(&self, user_id: Uuid) {
        self.sessions.retain(|_, session| session.user_id != user_id);
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) {
        self.sessions.retain(|_, session| !session.is_expired());
    }

    /// Get all active sessions for a user
    #[allow(dead_code)]
    pub async fn get_user_sessions(&self, user_id: Uuid) -> Vec<UserSession> {
        self.sessions.iter()
            .filter_map(|entry| {
                let session = entry.value();
                if session.user_id == user_id && !session.is_expired() {
                    Some(session.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get session count
    #[allow(dead_code)]
    pub async fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

impl Default for SessionService {
    fn default() -> Self {
        Self::new()
    }
}

// Session cleanup task - run this periodically
pub async fn session_cleanup_task(session_service: Arc<SessionService>) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // Every hour
    
    loop {
        interval.tick().await;
        session_service.cleanup_expired_sessions().await;
    }
}