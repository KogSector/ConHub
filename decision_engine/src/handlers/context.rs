use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use tracing::{info, error};

use crate::models::{AppState, ContextQueryRequest, ContextQueryResponse, StatsResponse};
use crate::services::DecisionService;

/// Query context with strategy selection
pub async fn query_context(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ContextQueryRequest>,
) -> Result<Json<ContextQueryResponse>, (StatusCode, String)> {
    info!(
        "📥 Context query from tenant={}, user={}, strategy={:?}",
        request.tenant_id, request.user_id, request.strategy
    );

    // Try cache first
    let cached_response = {
        let mut cache = state.cache.write().await;
        cache.get(&request).await.ok().flatten()
    };

    if let Some(cached) = cached_response {
        info!("🎯 Returning cached response ({} blocks)", cached.blocks.len());
        return Ok(Json(cached));
    }

    // Execute query
    match DecisionService::query_context(
        &state.vector_client,
        &state.graph_client,
        &request,
    ).await {
        Ok(response) => {
            info!(
                "✅ Query complete: {} blocks, {} ms, strategy={:?}",
                response.blocks.len(),
                response.processing_time_ms,
                response.strategy_used
            );

            // Cache the response
            {
                let mut cache = state.cache.write().await;
                if let Err(e) = cache.set(&request, &response).await {
                    error!("⚠️  Failed to cache response: {}", e);
                }
            }

            Ok(Json(response))
        }
        Err(e) => {
            error!("❌ Context query failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Context query failed: {}", e),
            ))
        }
    }
}

/// Get cache and query statistics
pub async fn get_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<StatsResponse>, (StatusCode, String)> {
    let mut cache = state.cache.write().await;
    let cache_healthy = cache.health_check().await;

    Ok(Json(StatsResponse {
        cache_enabled: cache_healthy,
        cache_hit_rate: 0.0, // TODO: track metrics
        total_queries: 0,
        avg_latency_ms: 0.0,
        strategy_distribution: std::collections::HashMap::new(),
    }))
}
