// Security module - library only (no HTTP server)
pub mod zero_trust;

pub use zero_trust::{
    ZeroTrustService, AccessPolicy, AccessRequest, AccessDecision,
    AccessContext, AccessCondition, Permission, ResourceType,
};

pub struct SecurityModule {
    pub zero_trust: ZeroTrustService,
}

impl SecurityModule {
    pub fn new() -> Self {
        Self {
            zero_trust: ZeroTrustService::new(),
        }
    }
}

impl Default for SecurityModule {
    fn default() -> Self {
        Self::new()
    }
}
