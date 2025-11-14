use uuid::Uuid;
use crate::protocol::SharedContext;
use crate::error::MCPError;

/// Service for managing shared context between agents
pub struct ContextService;

impl ContextService {
    pub fn new() -> Self {
        Self
    }
    
    /// Validate context data
    pub fn validate_context(&self, context: &SharedContext) -> Result<(), MCPError> {
        if context.context_type.is_empty() {
            return Err(MCPError::InvalidRequest(
                "Context type cannot be empty".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Check if context has expired
    pub fn is_expired(&self, context: &SharedContext) -> bool {
        if let Some(expires_at) = context.expires_at {
            chrono::Utc::now() > expires_at
        } else {
            false
        }
    }
}
