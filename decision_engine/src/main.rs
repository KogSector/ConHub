use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use conhub_observability::{init_tracing, TracingConfig, info, warn};

mod models;
mod handlers;
mod services;

use models::AppState;
use services::{VectorRagClient, GraphRagClient, QueryCache};
use handlers::{memory, context};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize observability with structured logging
    init_tracing(TracingConfig::for_service("decision-engine").json());

    let port = std::env::var("DECISION_ENGINE_PORT")
        .unwrap_or_else(|_| "3016".to_string())
        .parse::<u16>()?;

    let host = std::env::var("DECISION_ENGINE_HOST")
        .unwrap_or_else(|_| "0.0.0.0".to_string());

    info!("🚀 [Decision Engine] Starting on {}:{}", host, port);

    // Service URLs
    let vector_rag_url = std::env::var("VECTOR_RAG_URL")
        .unwrap_or_else(|_| "http://localhost:8082".to_string());
    
    let graph_rag_url = std::env::var("GRAPH_RAG_URL")
        .unwrap_or_else(|_| "http://localhost:8006".to_string());

    let redis_url = std::env::var("REDIS_URL").ok();

    info!("📡 Vector RAG service: {}", vector_rag_url);
    info!("🔗 Graph RAG service: {}", graph_rag_url);
    if let Some(ref redis) = redis_url {
        info!("💾 Redis cache: {}", redis);
    } else {
        info!("💾 Redis cache: disabled");
    }

    // Initialize clients
    let vector_client = VectorRagClient::new(vector_rag_url);
    let graph_client = GraphRagClient::new(graph_rag_url);
    let cache = QueryCache::new(redis_url).await;

    // Create app state
    let state = Arc::new(AppState {
        vector_client,
        graph_client,
        cache: RwLock::new(cache),
    });

    // Build router
    let app = Router::new()
        // Health checks
        .route("/health", get(health_check))
        .route("/api/memory/health", get(memory::memory_health))
        
        // Legacy context endpoints
        .route("/context/query", post(context::query_context))
        .route("/context/stats", get(context::get_stats))
        
        // New memory search endpoints (for AI agents)
        .route("/api/memory/search", post(memory::memory_search))
        
        // Robot memory endpoints
        .route("/api/robots/:robot_id/memory/search", post(memory::robot_memory_search))
        .route("/api/robots/:robot_id/context/latest", get(memory::robot_context_latest))
        
        .with_state(state)
        .layer(
            tower_http::cors::CorsLayer::permissive()
        )
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("✅ Decision Engine listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "service": "decision_engine",
        "version": "0.1.0",
        "features": {
            "memory_search": true,
            "robot_memory": true,
            "query_analysis": true,
            "strategy_selection": true,
            "context_building": true,
            "vector_rag": true,
            "graph_rag": true,
            "hybrid_retrieval": true
        },
        "endpoints": {
            "general": ["/api/memory/search"],
            "robot": ["/api/robots/:robot_id/memory/search", "/api/robots/:robot_id/context/latest"],
            "legacy": ["/context/query", "/context/stats"]
        }
    }))
}
