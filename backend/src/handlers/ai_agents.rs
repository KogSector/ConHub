use crate::services::ai_service::AIAgentManager;
use actix_web::{web, HttpResponse, Responder};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct AppState {
    pub agent_manager: Mutex<AIAgentManager>,
}

pub async fn create_agent(
    agent_type: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let manager = data.agent_manager.lock().unwrap();
    match manager.create_agent(&agent_type) {
        Ok(agent) => HttpResponse::Ok().json(agent),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

pub async fn list_agents(data: web::Data<AppState>) -> impl Responder {
    let manager = data.agent_manager.lock().unwrap();
    let agents = manager.list_agents();
    HttpResponse::Ok().json(agents)
}

pub async fn connect_agent(
    agent_id: web::Path<String>,
    credentials: web::Json<HashMap<String, String>>,
    data: web::Data<AppState>,
) -> impl Responder {
    let manager = data.agent_manager.lock().unwrap();
    match manager.connect_agent(&agent_id, &credentials).await {
        Ok(success) => HttpResponse::Ok().json(success),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

pub async fn query_agent(
    agent_id: web::Path<String>,
    query: web::Json<HashMap<String, String>>,
    data: web::Data<AppState>,
) -> impl Responder {
    let manager = data.agent_manager.lock().unwrap();
    let prompt = query.get("prompt").cloned().unwrap_or_default();
    let context = query.get("context").cloned();
    match manager.query_agent(&agent_id, &prompt, context.as_deref()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/agents")
            .route("/create/{agent_type}", web::post().to(create_agent))
            .route("/list", web::get().to(list_agents))
            .route("/connect/{agent_id}", web::post().to(connect_agent))
            .route("/query/{agent_id}", web::post().to(query_agent)),
    );
}
