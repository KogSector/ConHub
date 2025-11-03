use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use sqlx::PgPool;
use redis::AsyncCommands;
use serde_json::json;

use conhub_models::auth::*;
use super::security::SecurityService;

pub struct SessionService {
    pool: PgPool,
    redis_client: redis::Client,
    security_service: std::sync::Arc<SecurityService>,
}

impl SessionService {
    pub fn new(pool: PgPool, redis_client: redis::Client, security_service: std::sync::Arc<SecurityService>) -> Self {
        Self {
            pool,
            redis_client,
            security_service,
        }
    }
    
    pub async fn create_session(&self, user: &User, device_info: Option<DeviceInfo>, ip_address: Option<String>, user_agent: Option<String>, remember_me: bool) -> Result<(UserSession, String, String), Box<dyn std::error::Error>> {
        let session_id = Uuid::new_v4();
        
        // Generate JWT tokens
        let (access_token, refresh_token, token_expires, refresh_expires) = self.security_service
            .generate_jwt_token(user, session_id, remember_me)
            .await?;
        
        // Create session record
        let device_info_json = device_info.as_ref().map(|d| serde_json::to_value(d)).transpose()?;
        
        let session = sqlx::query_as::<_, UserSession>(
            r#"
            INSERT INTO user_sessions (
                id, user_id, session_token, refresh_token, device_info, 
                ip_address, user_agent, expires_at, refresh_expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#
        )
        .bind(session_id)
        .bind(user.id)
        .bind(&access_token[..50]) // Store only first 50 chars for identification
        .bind(&refresh_token)
        .bind(device_info_json)
        .bind(ip_address.clone())
        .bind(user_agent.as_deref())
        .bind(token_expires)
        .bind(refresh_expires)
        .fetch_one(&self.pool)
        .await?;
        
        // Cache session in Redis for fast lookup
        let mut redis_conn = self.redis_client.get_async_connection().await?;
        let session_key = format!("session:{}", session_id);
        let session_data = json!({
            "user_id": user.id,
            "session_id": session_id,
            "expires_at": token_expires,
            "refresh_expires_at": refresh_expires
        });
        
        redis_conn.set_ex(
            &session_key,
            session_data.to_string(),
            if remember_me { 30 * 24 * 3600 } else { 24 * 3600 }, // 30 days or 24 hours
        ).await?;
        
        // Log successful session creation
        self.security_service.log_security_event(
            Some(user.id),
            AuditEventType::LoginSuccess,
            ip_address,
            user_agent,
            Some(json!({
                "session_id": session_id,
                "device_info": device_info,
                "remember_me": remember_me
            })),
            Some(0),
            Some(session_id)
        ).await?;
        
        Ok((session, access_token, refresh_token))
    }
    
    pub async fn validate_session(&self, session_id: Uuid) -> Result<Option<UserSession>, Box<dyn std::error::Error>> {
        // Check Redis cache first
        let mut redis_conn = self.redis_client.get_async_connection().await?;
        let session_key = format!("session:{}", session_id);
        
        if let Ok(cached_data) = redis_conn.get::<_, String>(&session_key).await {
            if let Ok(session_data) = serde_json::from_str::<serde_json::Value>(&cached_data) {
                let expires_at = session_data["expires_at"].as_str()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc));
                
                if let Some(expires_at) = expires_at {
                    if Utc::now() < expires_at {
                        // Session is valid, fetch from database
                        return self.get_session_from_db(session_id).await;
                    }
                }
            }
        }
        
        // Check database if not in cache or expired
        self.get_session_from_db(session_id).await
    }
    
    async fn get_session_from_db(&self, session_id: Uuid) -> Result<Option<UserSession>, Box<dyn std::error::Error>> {
        let session = sqlx::query_as::<_, UserSession>(
            "SELECT * FROM user_sessions WHERE id = $1 AND status = 'active' AND expires_at > NOW()"
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(ref session) = session {
            // Update last used timestamp
            sqlx::query(
                "UPDATE user_sessions SET last_used_at = NOW(), updated_at = NOW() WHERE id = $1"
            )
            .bind(session_id)
            .execute(&self.pool)
            .await?;
        }
        
        Ok(session)
    }
    
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<(String, DateTime<Utc>), Box<dyn std::error::Error>> {
        // Find session by refresh token
        let session = sqlx::query_as::<_, UserSession>(
            "SELECT * FROM user_sessions WHERE refresh_token = $1 AND status = 'active' AND refresh_expires_at > NOW()"
        )
        .bind(refresh_token)
        .fetch_optional(&self.pool)
        .await?;
        
        let session = session.ok_or("Invalid or expired refresh token")?;
        
        // Get user details
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1 AND is_active = true"
        )
        .bind(session.user_id)
        .fetch_optional(&self.pool)
        .await?;
        
        let user = user.ok_or("User not found or inactive")?;
        
        // Generate new access token
        let (new_access_token, _, token_expires, _) = self.security_service
            .generate_jwt_token(&user, session.id, false)
            .await?;
        
        // Update session with new token
        sqlx::query(
            "UPDATE user_sessions SET session_token = $1, expires_at = $2, last_used_at = NOW(), updated_at = NOW() WHERE id = $3"
        )
        .bind(&new_access_token[..50])
        .bind(token_expires)
        .bind(session.id)
        .execute(&self.pool)
        .await?;
        
        // Update Redis cache
        let mut redis_conn = self.redis_client.get_async_connection().await?;
        let session_key = format!("session:{}", session.id);
        let session_data = json!({
            "user_id": user.id,
            "session_id": session.id,
            "expires_at": token_expires,
            "refresh_expires_at": session.refresh_expires_at
        });
        
        redis_conn.set_ex(
            &session_key,
            session_data.to_string(),
            24 * 3600, // 24 hours
        ).await?;
        
        // Log token refresh
        self.security_service.log_security_event(
            Some(user.id),
            AuditEventType::TokenRefresh,
            session.ip_address,
            session.user_agent.clone(),
            Some(json!({
                "session_id": session.id,
                "old_expires_at": session.expires_at,
                "new_expires_at": token_expires
            })),
            Some(0),
            Some(session.id)
        ).await?;
        
        Ok((new_access_token, token_expires))
    }
    
    pub async fn revoke_session(&self, session_id: Uuid, user_id: Option<Uuid>) -> Result<(), Box<dyn std::error::Error>> {
        // Update session status in database
        let mut query = sqlx::query(
            "UPDATE user_sessions SET status = 'revoked', updated_at = NOW() WHERE id = $1"
        );
        
        if let Some(uid) = user_id {
            query = sqlx::query(
                "UPDATE user_sessions SET status = 'revoked', updated_at = NOW() WHERE id = $1 AND user_id = $2"
            ).bind(uid);
        }
        
        query.bind(session_id).execute(&self.pool).await?;
        
        // Remove from Redis cache
        let mut redis_conn = self.redis_client.get_async_connection().await?;
        let session_key = format!("session:{}", session_id);
        redis_conn.del(&session_key).await?;
        
        // Log logout
        self.security_service.log_security_event(
            user_id,
            AuditEventType::Logout,
            None,
            None,
            Some(json!({
                "session_id": session_id,
                "revoked_at": Utc::now()
            })),
            Some(0),
            Some(session_id)
        ).await?;
        
        Ok(())
    }
    
    pub async fn revoke_all_user_sessions(&self, user_id: Uuid, except_session: Option<Uuid>) -> Result<i32, Box<dyn std::error::Error>> {
        let mut query_str = "UPDATE user_sessions SET status = 'revoked', updated_at = NOW() WHERE user_id = $1 AND status = 'active'";
        let mut query = sqlx::query(query_str).bind(user_id);
        
        if let Some(except_id) = except_session {
            query_str = "UPDATE user_sessions SET status = 'revoked', updated_at = NOW() WHERE user_id = $1 AND status = 'active' AND id != $2";
            query = sqlx::query(query_str).bind(user_id).bind(except_id);
        }
        
        let result = query.execute(&self.pool).await?;
        let revoked_count = result.rows_affected() as i32;
        
        // Remove all user sessions from Redis cache
        let mut redis_conn = self.redis_client.get_async_connection().await?;
        let pattern = format!("session:*");
        let keys: Vec<String> = redis_conn.keys(&pattern).await?;
        
        for key in keys {
            if let Ok(cached_data) = redis_conn.get::<_, String>(&key).await {
                if let Ok(session_data) = serde_json::from_str::<serde_json::Value>(&cached_data) {
                    if let Some(cached_user_id) = session_data["user_id"].as_str() {
                        if cached_user_id == user_id.to_string() {
                            if let Some(except_id) = except_session {
                                if let Some(cached_session_id) = session_data["session_id"].as_str() {
                                    if cached_session_id != except_id.to_string() {
                                        redis_conn.del(&key).await?;
                                    }
                                }
                            } else {
                                redis_conn.del(&key).await?;
                            }
                        }
                    }
                }
            }
        }
        
        // Log mass logout
        self.security_service.log_security_event(
            Some(user_id),
            AuditEventType::Logout,
            None,
            None,
            Some(json!({
                "revoked_sessions_count": revoked_count,
                "except_session": except_session,
                "revoked_at": Utc::now()
            })),
            Some(10),
            None
        ).await?;
        
        Ok(revoked_count)
    }
    
    pub async fn get_user_sessions(&self, user_id: Uuid, current_session_id: Option<Uuid>) -> Result<Vec<SessionInfo>, Box<dyn std::error::Error>> {
        let sessions = sqlx::query_as::<_, UserSession>(
            "SELECT * FROM user_sessions WHERE user_id = $1 AND status = 'active' ORDER BY last_used_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        
        let session_infos = sessions.into_iter().map(|session| {
            SessionInfo {
                id: session.id,
                device_info: session.device_info,
                ip_address: session.ip_address,
                location: session.location,
                created_at: session.created_at,
                last_used_at: session.last_used_at,
                is_current: current_session_id.map_or(false, |id| id == session.id),
            }
        }).collect();
        
        Ok(session_infos)
    }
    
    pub async fn cleanup_expired_sessions(&self) -> Result<i32, Box<dyn std::error::Error>> {
        // Update expired sessions in database
        let result = sqlx::query(
            "UPDATE user_sessions SET status = 'expired', updated_at = NOW() 
             WHERE status = 'active' AND (expires_at < NOW() OR refresh_expires_at < NOW())"
        )
        .execute(&self.pool)
        .await?;
        
        let expired_count = result.rows_affected() as i32;
        
        // Clean up Redis cache
        let mut redis_conn = self.redis_client.get_async_connection().await?;
        let pattern = format!("session:*");
        let keys: Vec<String> = redis_conn.keys(&pattern).await?;
        
        for key in keys {
            if let Ok(cached_data) = redis_conn.get::<_, String>(&key).await {
                if let Ok(session_data) = serde_json::from_str::<serde_json::Value>(&cached_data) {
                    let expires_at = session_data["expires_at"].as_str()
                        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.with_timezone(&Utc));
                    
                    if let Some(expires_at) = expires_at {
                        if Utc::now() >= expires_at {
                            redis_conn.del(&key).await?;
                        }
                    }
                }
            }
        }
        
        tracing::info!("Cleaned up {} expired sessions", expired_count);
        Ok(expired_count)
    }
    
    pub async fn invalidate_session(&self, session_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        self.revoke_session(session_id, None).await
    }
    
    pub async fn invalidate_all_user_sessions(&self, user_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        self.revoke_all_user_sessions(user_id, None).await?;
        Ok(())
    }

    pub async fn get_session_by_token(&self, token_prefix: &str) -> Result<Option<UserSession>, Box<dyn std::error::Error>> {
        let session = sqlx::query_as::<_, UserSession>(
            "SELECT * FROM user_sessions WHERE session_token = $1 AND status = 'active' AND expires_at > NOW()"
        )
        .bind(token_prefix)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(session)
    }
}


pub async fn session_cleanup_task(session_service: std::sync::Arc<SessionService>) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); 
    
    loop {
        interval.tick().await;
        if let Err(e) = session_service.cleanup_expired_sessions().await {
            tracing::error!("Failed to cleanup expired sessions: {}", e);
        }
    }
}