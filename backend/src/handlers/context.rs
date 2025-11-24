use actix_web::{web, HttpResponse};
use crate::services::DecisionEngineClient;
use crate::services::decision_engine_client::*;
use std::sync::Arc;
use uuid::Uuid;

/// Query context through decision engine
pub async fn query_context(
    req: web::Json<ContextQueryRequest>,
    decision_engine: web::Data<Arc<DecisionEngineClient>>,
) -> HttpResponse {
    log::info!(
        "Context query: '{}' (strategy: {:?}, top_k: {})",
        req.query,
        req.strategy,
        req.top_k
    );
    
    match decision_engine.query_context(req.into_inner()).await {
        Ok(response) => {
            log::info!(
                "Context query successful: {} results using {:?} strategy in {}ms",
                response.total_results,
                response.strategy_used,
                response.processing_time_ms
            );
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            log::error!("Context query failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Context query failed",
                "details": e.to_string()
            }))
        }
    }
}

/// Get decision engine statistics
pub async fn get_stats(
    decision_engine: web::Data<Arc<DecisionEngineClient>>,
) -> HttpResponse {
    match decision_engine.get_stats().await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => {
            log::error!("Failed to get stats: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get stats",
                "details": e.to_string()
            }))
        }
    }
}

/// Simple query endpoint for convenience (auto strategy)
#[derive(serde::Deserialize)]
pub struct SimpleQueryRequest {
    pub query: String,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

fn default_top_k() -> usize {
    20
}

pub async fn simple_query(
    req: web::Json<SimpleQueryRequest>,
    decision_engine: web::Data<Arc<DecisionEngineClient>>,
) -> HttpResponse {
    let context_request = ContextQueryRequest {
        tenant_id: req.tenant_id,
        user_id: req.user_id,
        query: req.query.clone(),
        filters: QueryFilters::default(),
        strategy: Strategy::Auto,
        top_k: req.top_k,
    };

    match decision_engine.query_context(context_request).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            log::error!("Simple query failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Query failed",
                "details": e.to_string()
            }))
        }
    }
}
