use actix_web::{web, App, HttpServer, HttpResponse};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use uuid::Uuid;
use conhub_observability::{init_tracing, TracingConfig, observability, info, warn};

mod handlers;
mod services;
mod models;

use models::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize observability with structured logging
    init_tracing(TracingConfig::for_service("chunker-service"));

    let port = env::var("CHUNKER_PORT")
        .unwrap_or_else(|_| "3017".to_string())
        .parse::<u16>()
        .expect("Invalid port number");

    let host = env::var("CHUNKER_HOST")
        .unwrap_or_else(|_| "0.0.0.0".to_string());

    info!("🚀 [Chunker Service] Starting on {}:{}", host, port);

    // Initialize downstream service clients
    let embedding_url = env::var("EMBEDDING_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:8082".to_string());
    
    let graph_url = env::var("GRAPH_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:8006".to_string());

    let redis_url = env::var("REDIS_URL").ok();

    let max_concurrent_jobs = env::var("MAX_CONCURRENT_JOBS")
        .unwrap_or_else(|_| "10".to_string())
        .parse::<usize>()
        .unwrap_or(10);

    info!("📡 Embedding service: {}", embedding_url);
    info!("🔗 Graph service: {}", graph_url);
    if let Some(ref redis) = redis_url {
        info!("💾 Redis cache: {}", redis);
    } else {
        info!("💾 Redis cache: disabled (no REDIS_URL)");
    }
    info!("⚙️  Max concurrent jobs: {}", max_concurrent_jobs);

    // Initialize cache
    let cache = services::cache::ChunkCache::new(redis_url).await;

    // Initialize profile manager with optional profile from env
    let mut profile_manager = services::profiles::ProfileManager::new();
    if let Ok(profile_name) = env::var("CHUNKER_PROFILE") {
        if profile_manager.set_active(&profile_name) {
            info!("📋 Using chunker profile: {}", profile_name);
        } else {
            warn!("⚠️ Unknown chunker profile '{}', using standard", profile_name);
        }
    } else {
        info!("📋 Using chunker profile: standard");
    }

    // Initialize cost policy manager with optional policy from env
    let mut cost_policy_manager = services::cost_policy::CostPolicyManager::new();
    if let Ok(policy_name) = env::var("COST_POLICY") {
        if cost_policy_manager.set_active(&policy_name) {
            info!("💰 Using cost policy: {}", policy_name);
        } else {
            warn!("⚠️ Unknown cost policy '{}', using balanced", policy_name);
        }
    } else {
        info!("💰 Using cost policy: balanced");
    }

    // Create app state
    let state = Arc::new(AppState {
        embedding_client: services::embedding_client::EmbeddingClient::new(embedding_url),
        graph_client: services::graph_client::GraphClient::new(graph_url),
        cache: RwLock::new(cache),
        max_concurrent_jobs,
        jobs: RwLock::new(HashMap::new()),
        profiles: RwLock::new(profile_manager),
        cost_policies: RwLock::new(cost_policy_manager),
    });

    HttpServer::new(move || {
        App::new()
            .wrap(observability("chunker-service"))
            .app_data(web::Data::new(state.clone()))
            .route("/health", web::get().to(health_check))
            .route("/chunk/jobs", web::post().to(handlers::jobs::start_chunk_job))
            .route("/chunk/jobs/{job_id}", web::get().to(handlers::jobs::get_chunk_job_status))
            // Profile management endpoints
            .route("/chunk/profiles", web::get().to(list_profiles))
            .route("/chunk/profiles/active", web::get().to(get_active_profile))
            .route("/chunk/profiles/active/{name}", web::put().to(set_active_profile))
            // Cost policy management endpoints
            .route("/chunk/policies", web::get().to(list_cost_policies))
            .route("/chunk/policies/active", web::get().to(get_active_cost_policy))
            .route("/chunk/policies/active/{name}", web::put().to(set_active_cost_policy))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "chunker",
        "version": "0.1.0"
    }))
}

/// List available chunker profiles
async fn list_profiles(state: web::Data<Arc<AppState>>) -> HttpResponse {
    let profiles = state.profiles.read().await;
    let names = profiles.list_profiles();
    let active = profiles.active().name.clone();
    
    HttpResponse::Ok().json(serde_json::json!({
        "profiles": names,
        "active": active
    }))
}

/// Get the currently active chunker profile
async fn get_active_profile(state: web::Data<Arc<AppState>>) -> HttpResponse {
    let profiles = state.profiles.read().await;
    let active = profiles.active();
    
    HttpResponse::Ok().json(active)
}

/// Set the active chunker profile
async fn set_active_profile(
    state: web::Data<Arc<AppState>>,
    path: web::Path<String>,
) -> HttpResponse {
    let name = path.into_inner();
    let mut profiles = state.profiles.write().await;
    
    if profiles.set_active(&name) {
        info!("📋 Switched to chunker profile: {}", name);
        HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "active": name
        }))
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "error": format!("Unknown profile: {}", name),
            "available": profiles.list_profiles()
        }))
    }
}

/// List available cost policies
async fn list_cost_policies(state: web::Data<Arc<AppState>>) -> HttpResponse {
    let policies = state.cost_policies.read().await;
    let names = policies.list_policies();
    let active = policies.active().name.clone();
    
    HttpResponse::Ok().json(serde_json::json!({
        "policies": names,
        "active": active
    }))
}

/// Get the currently active cost policy
async fn get_active_cost_policy(state: web::Data<Arc<AppState>>) -> HttpResponse {
    let policies = state.cost_policies.read().await;
    let active = policies.active();
    
    HttpResponse::Ok().json(active)
}

/// Set the active cost policy
async fn set_active_cost_policy(
    state: web::Data<Arc<AppState>>,
    path: web::Path<String>,
) -> HttpResponse {
    let name = path.into_inner();
    let mut policies = state.cost_policies.write().await;
    
    if policies.set_active(&name) {
        info!("💰 Switched to cost policy: {}", name);
        HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "active": name
        }))
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "error": format!("Unknown policy: {}", name),
            "available": policies.list_policies()
        }))
    }
}
