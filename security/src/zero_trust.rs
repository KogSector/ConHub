use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use tracing::{info, warn};

/// Access policy for zero-trust model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPolicy {
    pub resource_id: Uuid,
    pub resource_type: ResourceType,
    pub user_id: Uuid,
    pub permissions: Vec<Permission>,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub conditions: Vec<AccessCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceType {
    Repository,
    Document,
    Folder,
    Channel,
    DataSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
    Read,
    Write,
    Delete,
    Share,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessCondition {
    TimeWindow {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
    IpWhitelist {
        ips: Vec<String>,
    },
    MfaRequired,
    DeviceVerified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRequest {
    pub user_id: Uuid,
    pub resource_id: Uuid,
    pub resource_type: ResourceType,
    pub permission: Permission,
    pub context: AccessContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessContext {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_id: Option<String>,
    pub mfa_verified: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessDecision {
    pub allowed: bool,
    pub reason: Option<String>,
    pub policy_id: Option<Uuid>,
}

/// Zero-trust security service
pub struct ZeroTrustService {
    policies: std::sync::Arc<tokio::sync::RwLock<Vec<AccessPolicy>>>,
}

impl ZeroTrustService {
    pub fn new() -> Self {
        Self {
            policies: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
    
    /// Grant access to a resource with specific permissions
    pub async fn grant_access(
        &self,
        user_id: Uuid,
        resource_id: Uuid,
        resource_type: ResourceType,
        permissions: Vec<Permission>,
        duration: Option<Duration>,
        conditions: Vec<AccessCondition>,
    ) -> Result<Uuid, String> {
        let policy_id = Uuid::new_v4();
        let expires_at = duration.map(|d| Utc::now() + d);
        
        let policy = AccessPolicy {
            resource_id,
            resource_type: resource_type.clone(),
            user_id,
            permissions,
            granted_at: Utc::now(),
            expires_at,
            conditions,
        };
        
        let mut policies = self.policies.write().await;
        policies.push(policy);
        
        info!(
            "✓ Access granted to user {} for resource {} (type: {:?})",
            user_id, resource_id, resource_type
        );
        
        Ok(policy_id)
    }
    
    /// Revoke access to a resource
    pub async fn revoke_access(
        &self,
        user_id: Uuid,
        resource_id: Uuid,
    ) -> Result<(), String> {
        let mut policies = self.policies.write().await;
        policies.retain(|p| !(p.user_id == user_id && p.resource_id == resource_id));
        
        info!(
            "✓ Access revoked for user {} on resource {}",
            user_id, resource_id
        );
        
        Ok(())
    }
    
    /// Check if access is allowed
    pub async fn check_access(&self, request: &AccessRequest) -> AccessDecision {
        let policies = self.policies.read().await;
        
        // Find applicable policies
        let applicable_policies: Vec<&AccessPolicy> = policies
            .iter()
            .filter(|p| {
                p.user_id == request.user_id
                    && p.resource_id == request.resource_id
                    && p.resource_type == request.resource_type
            })
            .collect();
        
        if applicable_policies.is_empty() {
            return AccessDecision {
                allowed: false,
                reason: Some("No access policy found for this resource".to_string()),
                policy_id: None,
            };
        }
        
        // Check each policy
        for policy in applicable_policies {
            // Check if policy has expired
            if let Some(expires_at) = policy.expires_at {
                if Utc::now() > expires_at {
                    continue;
                }
            }
            
            // Check if user has the requested permission
            if !policy.permissions.contains(&request.permission) {
                continue;
            }
            
            // Verify all conditions
            let conditions_met = self.verify_conditions(&policy.conditions, &request.context);
            
            if conditions_met {
                return AccessDecision {
                    allowed: true,
                    reason: None,
                    policy_id: None,
                };
            } else {
                return AccessDecision {
                    allowed: false,
                    reason: Some("Access conditions not met".to_string()),
                    policy_id: None,
                };
            }
        }
        
        AccessDecision {
            allowed: false,
            reason: Some("Insufficient permissions".to_string()),
            policy_id: None,
        }
    }
    
    /// Verify access conditions
    fn verify_conditions(&self, conditions: &[AccessCondition], context: &AccessContext) -> bool {
        for condition in conditions {
            match condition {
                AccessCondition::TimeWindow { start, end } => {
                    let now = Utc::now();
                    if now < *start || now > *end {
                        warn!("Access denied: Outside time window");
                        return false;
                    }
                }
                AccessCondition::IpWhitelist { ips } => {
                    if let Some(ref ip) = context.ip_address {
                        if !ips.contains(ip) {
                            warn!("Access denied: IP not whitelisted");
                            return false;
                        }
                    } else {
                        warn!("Access denied: No IP address provided");
                        return false;
                    }
                }
                AccessCondition::MfaRequired => {
                    if !context.mfa_verified {
                        warn!("Access denied: MFA required but not verified");
                        return false;
                    }
                }
                AccessCondition::DeviceVerified => {
                    if context.device_id.is_none() {
                        warn!("Access denied: Device not verified");
                        return false;
                    }
                }
            }
        }
        
        true
    }
    
    /// List all policies for a user
    pub async fn list_user_policies(&self, user_id: Uuid) -> Vec<AccessPolicy> {
        let policies = self.policies.read().await;
        policies
            .iter()
            .filter(|p| p.user_id == user_id)
            .cloned()
            .collect()
    }
    
    /// List all policies for a resource
    pub async fn list_resource_policies(&self, resource_id: Uuid) -> Vec<AccessPolicy> {
        let policies = self.policies.read().await;
        policies
            .iter()
            .filter(|p| p.resource_id == resource_id)
            .cloned()
            .collect()
    }
    
    /// Clean up expired policies
    pub async fn cleanup_expired_policies(&self) -> usize {
        let mut policies = self.policies.write().await;
        let before_count = policies.len();
        let now = Utc::now();
        
        policies.retain(|p| {
            if let Some(expires_at) = p.expires_at {
                expires_at > now
            } else {
                true
            }
        });
        
        let removed = before_count - policies.len();
        if removed > 0 {
            info!("✓ Cleaned up {} expired policies", removed);
        }
        
        removed
    }
}

impl Default for ZeroTrustService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_grant_and_check_access() {
        let service = ZeroTrustService::new();
        let user_id = Uuid::new_v4();
        let resource_id = Uuid::new_v4();
        
        // Grant read access
        service.grant_access(
            user_id,
            resource_id,
            ResourceType::Document,
            vec![Permission::Read],
            None,
            vec![],
        ).await.unwrap();
        
        // Check access
        let request = AccessRequest {
            user_id,
            resource_id,
            resource_type: ResourceType::Document,
            permission: Permission::Read,
            context: AccessContext {
                ip_address: None,
                user_agent: None,
                device_id: None,
                mfa_verified: false,
                timestamp: Utc::now(),
            },
        };
        
        let decision = service.check_access(&request).await;
        assert!(decision.allowed);
        
        // Check write access (should fail)
        let write_request = AccessRequest {
            permission: Permission::Write,
            ..request
        };
        
        let write_decision = service.check_access(&write_request).await;
        assert!(!write_decision.allowed);
    }
    
    #[tokio::test]
    async fn test_expired_policy() {
        let service = ZeroTrustService::new();
        let user_id = Uuid::new_v4();
        let resource_id = Uuid::new_v4();
        
        // Grant access that expires immediately
        service.grant_access(
            user_id,
            resource_id,
            ResourceType::Document,
            vec![Permission::Read],
            Some(Duration::milliseconds(-1)),
            vec![],
        ).await.unwrap();
        
        let request = AccessRequest {
            user_id,
            resource_id,
            resource_type: ResourceType::Document,
            permission: Permission::Read,
            context: AccessContext {
                ip_address: None,
                user_agent: None,
                device_id: None,
                mfa_verified: false,
                timestamp: Utc::now(),
            },
        };
        
        let decision = service.check_access(&request).await;
        assert!(!decision.allowed);
    }
    
    #[tokio::test]
    async fn test_mfa_condition() {
        let service = ZeroTrustService::new();
        let user_id = Uuid::new_v4();
        let resource_id = Uuid::new_v4();
        
        // Grant access with MFA requirement
        service.grant_access(
            user_id,
            resource_id,
            ResourceType::Document,
            vec![Permission::Read],
            None,
            vec![AccessCondition::MfaRequired],
        ).await.unwrap();
        
        // Without MFA
        let request_no_mfa = AccessRequest {
            user_id,
            resource_id,
            resource_type: ResourceType::Document,
            permission: Permission::Read,
            context: AccessContext {
                ip_address: None,
                user_agent: None,
                device_id: None,
                mfa_verified: false,
                timestamp: Utc::now(),
            },
        };
        
        let decision = service.check_access(&request_no_mfa).await;
        assert!(!decision.allowed);
        
        // With MFA
        let request_with_mfa = AccessRequest {
            context: AccessContext {
                mfa_verified: true,
                ..request_no_mfa.context
            },
            ..request_no_mfa
        };
        
        let decision_with_mfa = service.check_access(&request_with_mfa).await;
        assert!(decision_with_mfa.allowed);
    }
}
