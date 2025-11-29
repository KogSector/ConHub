//! Memory Search Handlers
//! 
//! HTTP endpoints for memory/knowledge retrieval:
//! - POST /api/memory/search - General memory search
//! - POST /api/robots/{robot_id}/memory/search - Robot-specific memory search
//! - GET /api/robots/{robot_id}/context/latest - Latest robot context snapshot

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use tracing::{info, error};
use uuid::Uuid;

use crate::models::{AppState, MemoryQuery, MemorySearchResponse, RobotMemoryQuery};
use crate::services::{MemorySearchService, RobotMemorySearchService};
use crate::services::memory_search::RobotContextSnapshot;

/// POST /api/memory/search
/// 
/// General memory search endpoint for AI agents.
/// Analyzes the query, selects optimal retrieval strategy, and returns context blocks.
pub async fn memory_search(
    State(state): State<Arc<AppState>>,
    Json(query): Json<MemoryQuery>,
) -> Result<Json<MemorySearchResponse>, (StatusCode, String)> {
    info!(
        "📥 Memory search from tenant={}, user={}: '{}'",
        query.tenant_id, query.user_id, query.query
    );
    
    // Create the memory search service
    let service = MemorySearchService::new(
        state.vector_client.clone(),
        state.graph_client.clone(),
    );
    
    match service.search(query).await {
        Ok(response) => {
            info!(
                "✅ Memory search complete: {} blocks, {}ms, kind={:?}",
                response.blocks.len(),
                response.took_ms,
                response.query_kind
            );
            Ok(Json(response))
        }
        Err(e) => {
            error!("❌ Memory search failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Memory search failed: {}", e),
            ))
        }
    }
}

/// POST /api/robots/{robot_id}/memory/search
/// 
/// Robot-specific memory search.
/// Searches episodic and semantic memory for a specific robot.
pub async fn robot_memory_search(
    State(state): State<Arc<AppState>>,
    Path(robot_id): Path<Uuid>,
    Json(mut query): Json<RobotMemoryQuery>,
) -> Result<Json<MemorySearchResponse>, (StatusCode, String)> {
    // Ensure robot_id matches
    query.robot_id = robot_id;
    
    info!(
        "🤖 Robot memory search for robot={}: '{}'",
        robot_id, query.query
    );
    
    let service = RobotMemorySearchService::new(
        state.vector_client.clone(),
        state.graph_client.clone(),
    );
    
    match service.search(query).await {
        Ok(response) => {
            info!(
                "✅ Robot memory search complete: {} blocks, {}ms",
                response.blocks.len(),
                response.took_ms
            );
            Ok(Json(response))
        }
        Err(e) => {
            error!("❌ Robot memory search failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Robot memory search failed: {}", e),
            ))
        }
    }
}

/// GET /api/robots/{robot_id}/context/latest
/// 
/// Get the latest context snapshot for a robot.
/// Includes recent episodes, relevant facts, and current state.
pub async fn robot_context_latest(
    State(state): State<Arc<AppState>>,
    Path(robot_id): Path<Uuid>,
) -> Result<Json<RobotContextSnapshot>, (StatusCode, String)> {
    info!("🤖 Getting latest context for robot={}", robot_id);
    
    let service = RobotMemorySearchService::new(
        state.vector_client.clone(),
        state.graph_client.clone(),
    );
    
    // Use a default tenant_id for now (should come from auth)
    let tenant_id = Uuid::nil();
    
    match service.get_latest_context(robot_id, tenant_id).await {
        Ok(snapshot) => {
            info!(
                "✅ Robot context snapshot: {} episodes, {} facts",
                snapshot.recent_episodes.len(),
                snapshot.relevant_facts.len()
            );
            Ok(Json(snapshot))
        }
        Err(e) => {
            error!("❌ Failed to get robot context: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get robot context: {}", e),
            ))
        }
    }
}

/// Simple health check for memory search
pub async fn memory_health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "memory_search",
        "features": {
            "general_search": true,
            "robot_memory": true,
            "query_analysis": true,
            "strategy_selection": true
        }
    }))
}
