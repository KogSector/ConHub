use actix_web::{web, HttpResponse};
use crate::services::{AgenticOrchestrator, AgenticQueryRequest};
use std::sync::Arc;

pub async fn agentic_query(
    req: web::Json<AgenticQueryRequest>,
    orchestrator: web::Data<Arc<AgenticOrchestrator>>,
) -> HttpResponse {
    tracing::info!("🤖 Agentic query request: {}", req.query);
    
    match orchestrator.execute_query(req.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            tracing::error!("Agentic query failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Query execution failed",
                "details": e.to_string()
            }))
        }
    }
}
