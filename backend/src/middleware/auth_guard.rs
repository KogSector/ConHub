use actix_web::{dev::ServiceRequest, Error, HttpMessage, HttpResponse};
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use uuid::Uuid;
use std::collections::HashSet;

use crate::models::auth::{Claims, UserRole, SubscriptionTier};

pub struct AuthGuard {
    pub required_roles: Option<Vec<UserRole>>,
    pub required_subscription: Option<SubscriptionTier>,
    pub check_subscription_active: bool,
}

impl AuthGuard {
    pub fn new() -> Self {
        Self {
            required_roles: None,
            required_subscription: None,
            check_subscription_active: false,
        }
    }

    pub fn require_roles(mut self, roles: Vec<UserRole>) -> Self {
        self.required_roles = Some(roles);
        self
    }

    pub fn require_subscription(mut self, tier: SubscriptionTier) -> Self {
        self.required_subscription = Some(tier);
        self
    }

    pub fn require_active_subscription(mut self) -> Self {
        self.check_subscription_active = true;
        self
    }

    pub fn admin_only() -> Self {
        Self::new().require_roles(vec![UserRole::Admin])
    }

    pub fn paid_subscription() -> Self {
        Self::new().require_subscription(SubscriptionTier::Personal)
    }

    pub fn team_subscription() -> Self {
        Self::new().require_subscription(SubscriptionTier::Team)
    }

    pub fn enterprise_subscription() -> Self {
        Self::new().require_subscription(SubscriptionTier::Enterprise)
    }
}

pub async fn bearer_auth_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());
    
    match validate_token(credentials.token(), &jwt_secret) {
        Ok(claims) => {
            req.extensions_mut().insert(claims);
            Ok(req)
        }
        Err(_) => {
            let config = req.app_data::<Config>().cloned().unwrap_or_default();
            Err((AuthenticationError::from(config).into(), req))
        }
    }
}

pub fn validate_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let key = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::new(Algorithm::HS256);
    
    let token_data = decode::<Claims>(token, &key, &validation)?;
    Ok(token_data.claims)
}

pub fn check_permissions(
    claims: &Claims,
    guard: &AuthGuard,
) -> Result<(), String> {
    // Check roles
    if let Some(required_roles) = &guard.required_roles {
        let user_roles: HashSet<String> = claims.roles.iter().cloned().collect();
        let required_role_strings: HashSet<String> = required_roles
            .iter()
            .map(|r| format!("{:?}", r).to_lowercase())
            .collect();

        if !required_role_strings.iter().any(|role| user_roles.contains(role)) {
            return Err("Insufficient role permissions".to_string());
        }
    }

    // Note: Subscription checks would need to be done with database access
    // This is a simplified version for demonstration
    
    Ok(())
}

// Middleware for protecting routes with specific requirements
pub fn create_auth_middleware(
    guard: AuthGuard,
) -> impl Fn(ServiceRequest, &mut actix_web::dev::ServiceResponse) -> Result<(), actix_web::Error> {
    move |req: ServiceRequest, _res: &mut actix_web::dev::ServiceResponse| {
        if let Some(claims) = req.extensions().get::<Claims>() {
            if let Err(error) = check_permissions(claims, &guard) {
                log::warn!("Permission denied: {}", error);
                return Err(actix_web::error::ErrorForbidden(error));
            }
        } else {
            return Err(actix_web::error::ErrorUnauthorized("Authentication required"));
        }
        Ok(())
    }
}

// Helper macros for common auth patterns
#[macro_export]
macro_rules! require_auth {
    () => {
        actix_web_httpauth::middleware::HttpAuthentication::bearer(
            crate::middleware::auth_guard::bearer_auth_validator
        )
    };
}

#[macro_export]
macro_rules! require_admin {
    () => {
        actix_web_httpauth::middleware::HttpAuthentication::bearer(
            crate::middleware::auth_guard::bearer_auth_validator
        )
        // Additional admin check would be added here
    };
}

#[macro_export]
macro_rules! require_subscription {
    ($tier:expr) => {
        actix_web_httpauth::middleware::HttpAuthentication::bearer(
            crate::middleware::auth_guard::bearer_auth_validator
        )
        // Additional subscription check would be added here
    };
}