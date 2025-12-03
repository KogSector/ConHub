//! Domain event logging for ConHub services.
//!
//! Provides structured logging for business domain events with consistent schema.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Result of a domain operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OperationResult {
    Success,
    Failure,
    Partial,
    Skipped,
}

impl std::fmt::Display for OperationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Failure => write!(f, "failure"),
            Self::Partial => write!(f, "partial"),
            Self::Skipped => write!(f, "skipped"),
        }
    }
}

/// Categories of domain events for filtering and routing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventCategory {
    Auth,
    Ingestion,
    Chunking,
    Embedding,
    Search,
    Graph,
    Billing,
    Robot,
    Connector,
    Sync,
    Mcp,
    Api,
    System,
}

impl std::fmt::Display for EventCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auth => write!(f, "auth"),
            Self::Ingestion => write!(f, "ingestion"),
            Self::Chunking => write!(f, "chunking"),
            Self::Embedding => write!(f, "embedding"),
            Self::Search => write!(f, "search"),
            Self::Graph => write!(f, "graph"),
            Self::Billing => write!(f, "billing"),
            Self::Robot => write!(f, "robot"),
            Self::Connector => write!(f, "connector"),
            Self::Sync => write!(f, "sync"),
            Self::Mcp => write!(f, "mcp"),
            Self::Api => write!(f, "api"),
            Self::System => write!(f, "system"),
        }
    }
}

/// A structured domain event for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// Category of the event
    pub category: EventCategory,
    /// Specific event type (e.g., "job_started", "document_chunked")
    pub event_type: String,
    /// Entity type being operated on (e.g., "sync_job", "document", "robot")
    pub entity_type: Option<String>,
    /// Entity ID
    pub entity_id: Option<String>,
    /// Result of the operation
    pub result: OperationResult,
    /// Duration in milliseconds (if applicable)
    pub duration_ms: Option<u64>,
    /// Attempt number for retries
    pub attempt: Option<u32>,
    /// Error message if failed
    pub error: Option<String>,
    /// Trace context
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    /// Tenant and user context
    pub tenant_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    /// Service that emitted the event
    pub service: String,
    /// Additional structured metadata
    pub metadata: Option<serde_json::Value>,
}

impl DomainEvent {
    /// Create a new domain event builder
    pub fn new(service: impl Into<String>, category: EventCategory, event_type: impl Into<String>) -> DomainEventBuilder {
        DomainEventBuilder {
            service: service.into(),
            category,
            event_type: event_type.into(),
            entity_type: None,
            entity_id: None,
            result: OperationResult::Success,
            duration_ms: None,
            attempt: None,
            error: None,
            trace_id: None,
            span_id: None,
            tenant_id: None,
            user_id: None,
            metadata: None,
        }
    }
}

/// Builder for constructing domain events
pub struct DomainEventBuilder {
    service: String,
    category: EventCategory,
    event_type: String,
    entity_type: Option<String>,
    entity_id: Option<String>,
    result: OperationResult,
    duration_ms: Option<u64>,
    attempt: Option<u32>,
    error: Option<String>,
    trace_id: Option<String>,
    span_id: Option<String>,
    tenant_id: Option<Uuid>,
    user_id: Option<Uuid>,
    metadata: Option<serde_json::Value>,
}

impl DomainEventBuilder {
    pub fn entity(mut self, entity_type: impl Into<String>, entity_id: impl Into<String>) -> Self {
        self.entity_type = Some(entity_type.into());
        self.entity_id = Some(entity_id.into());
        self
    }

    pub fn result(mut self, result: OperationResult) -> Self {
        self.result = result;
        self
    }

    pub fn success(mut self) -> Self {
        self.result = OperationResult::Success;
        self
    }

    pub fn failure(mut self, error: impl Into<String>) -> Self {
        self.result = OperationResult::Failure;
        self.error = Some(error.into());
        self
    }

    pub fn duration_ms(mut self, ms: u64) -> Self {
        self.duration_ms = Some(ms);
        self
    }

    pub fn attempt(mut self, attempt: u32) -> Self {
        self.attempt = Some(attempt);
        self
    }

    pub fn trace(mut self, trace_id: impl Into<String>, span_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self.span_id = Some(span_id.into());
        self
    }

    pub fn tenant(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    pub fn user(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Build and emit the event as a log
    pub fn emit(self) {
        let event = self.build();
        let json = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());
        
        match event.result {
            OperationResult::Success => tracing::info!(
                target: "domain_event",
                category = %event.category,
                event_type = %event.event_type,
                result = "success",
                "DomainEvent: {}", json
            ),
            OperationResult::Failure => tracing::error!(
                target: "domain_event",
                category = %event.category,
                event_type = %event.event_type,
                result = "failure",
                error = ?event.error,
                "DomainEvent: {}", json
            ),
            OperationResult::Partial => tracing::warn!(
                target: "domain_event",
                category = %event.category,
                event_type = %event.event_type,
                result = "partial",
                "DomainEvent: {}", json
            ),
            OperationResult::Skipped => tracing::debug!(
                target: "domain_event",
                category = %event.category,
                event_type = %event.event_type,
                result = "skipped",
                "DomainEvent: {}", json
            ),
        }
    }

    /// Build the event without emitting
    pub fn build(self) -> DomainEvent {
        DomainEvent {
            timestamp: Utc::now(),
            category: self.category,
            event_type: self.event_type,
            entity_type: self.entity_type,
            entity_id: self.entity_id,
            result: self.result,
            duration_ms: self.duration_ms,
            attempt: self.attempt,
            error: self.error,
            trace_id: self.trace_id,
            span_id: self.span_id,
            tenant_id: self.tenant_id,
            user_id: self.user_id,
            service: self.service,
            metadata: self.metadata,
        }
    }
}

// ============================================================================
// Convenience functions for common domain events
// ============================================================================

/// Log the start of a sync job
pub fn log_sync_job_started(service: &str, job_id: Uuid, connector: &str, trace_id: Option<&str>) {
    let mut builder = DomainEvent::new(service, EventCategory::Sync, "job_started")
        .entity("sync_job", job_id.to_string())
        .metadata(serde_json::json!({ "connector": connector }));
    
    if let Some(tid) = trace_id {
        builder = builder.trace(tid, "");
    }
    
    builder.emit();
}

/// Log sync job completion
pub fn log_sync_job_completed(service: &str, job_id: Uuid, docs_processed: usize, duration_ms: u64) {
    DomainEvent::new(service, EventCategory::Sync, "job_completed")
        .entity("sync_job", job_id.to_string())
        .duration_ms(duration_ms)
        .metadata(serde_json::json!({ "documents_processed": docs_processed }))
        .success()
        .emit();
}

/// Log sync job failure
pub fn log_sync_job_failed(service: &str, job_id: Uuid, error: &str, duration_ms: u64) {
    DomainEvent::new(service, EventCategory::Sync, "job_failed")
        .entity("sync_job", job_id.to_string())
        .duration_ms(duration_ms)
        .failure(error)
        .emit();
}

/// Log document chunking
pub fn log_document_chunked(service: &str, doc_id: &str, chunks_created: usize, duration_ms: u64) {
    DomainEvent::new(service, EventCategory::Chunking, "document_chunked")
        .entity("document", doc_id)
        .duration_ms(duration_ms)
        .metadata(serde_json::json!({ "chunks_created": chunks_created }))
        .success()
        .emit();
}

/// Log embedding generation
pub fn log_embedding_generated(service: &str, chunk_id: &str, model: &str, duration_ms: u64) {
    DomainEvent::new(service, EventCategory::Embedding, "embedding_generated")
        .entity("chunk", chunk_id)
        .duration_ms(duration_ms)
        .metadata(serde_json::json!({ "model": model }))
        .success()
        .emit();
}

/// Log search query execution
pub fn log_search_executed(
    service: &str,
    query_type: &str,
    results_count: usize,
    duration_ms: u64,
    strategy: &str,
) {
    DomainEvent::new(service, EventCategory::Search, "query_executed")
        .duration_ms(duration_ms)
        .metadata(serde_json::json!({
            "query_type": query_type,
            "results_count": results_count,
            "strategy": strategy
        }))
        .success()
        .emit();
}

/// Log MCP tool invocation
pub fn log_mcp_tool_invoked(service: &str, tool_name: &str, duration_ms: u64, success: bool, error: Option<&str>) {
    let mut builder = DomainEvent::new(service, EventCategory::Mcp, "tool_invoked")
        .entity("mcp_tool", tool_name)
        .duration_ms(duration_ms);
    
    if success {
        builder = builder.success();
    } else if let Some(err) = error {
        builder = builder.failure(err);
    }
    
    builder.emit();
}

/// Log robot event received
pub fn log_robot_event_received(service: &str, robot_id: Uuid, event_type: &str) {
    DomainEvent::new(service, EventCategory::Robot, "event_received")
        .entity("robot", robot_id.to_string())
        .metadata(serde_json::json!({ "event_type": event_type }))
        .success()
        .emit();
}

/// Log authentication event
pub fn log_auth_event(service: &str, event_type: &str, user_id: Option<Uuid>, success: bool, error: Option<&str>) {
    let mut builder = DomainEvent::new(service, EventCategory::Auth, event_type);
    
    if let Some(uid) = user_id {
        builder = builder.entity("user", uid.to_string()).user(uid);
    }
    
    if success {
        builder = builder.success();
    } else if let Some(err) = error {
        builder = builder.failure(err);
    }
    
    builder.emit();
}

/// Log billing event
pub fn log_billing_event(service: &str, event_type: &str, tenant_id: Uuid, amount: Option<f64>) {
    let mut builder = DomainEvent::new(service, EventCategory::Billing, event_type)
        .entity("tenant", tenant_id.to_string())
        .tenant(tenant_id);
    
    if let Some(amt) = amount {
        builder = builder.metadata(serde_json::json!({ "amount": amt }));
    }
    
    builder.emit();
}

/// Log connector operation
pub fn log_connector_operation(
    service: &str,
    connector_type: &str,
    operation: &str,
    success: bool,
    items_count: Option<usize>,
    duration_ms: u64,
    error: Option<&str>,
) {
    let mut builder = DomainEvent::new(service, EventCategory::Connector, operation)
        .entity("connector", connector_type)
        .duration_ms(duration_ms);
    
    if let Some(count) = items_count {
        builder = builder.metadata(serde_json::json!({ "items_count": count }));
    }
    
    if success {
        builder = builder.success();
    } else if let Some(err) = error {
        builder = builder.failure(err);
    }
    
    builder.emit();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_event_builder() {
        let event = DomainEvent::new("test-service", EventCategory::Sync, "job_started")
            .entity("sync_job", "123")
            .duration_ms(100)
            .success()
            .build();

        assert_eq!(event.service, "test-service");
        assert_eq!(event.event_type, "job_started");
        assert_eq!(event.entity_id, Some("123".to_string()));
        assert_eq!(event.result, OperationResult::Success);
    }
}
