use actix_web::{web, HttpResponse, Responder};
use crate::services::ai_service::{AIAgentManager};
use std::collections::HashMap;
use std::sync::Mutex;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/ai/agents")
            .route("", web::get().to(list_agents))
            .route("/create", web::post().to(create_agent))
            .route("/{agent_id}/connect", web::post().to(connect_agent))
            .route("/{agent_id}/query", web::post().to(query_agent)),
    );
}

async fn list_agents(manager: web::Data<Mutex<AIAgentManager>>) -> impl Responder {
    let manager = manager.lock().unwrap();
    let agents = manager.list_agents();
    HttpResponse::Ok().json(agents)
}

async fn create_agent(
    manager: web::Data<Mutex<AIAgentManager>>,
    body: web::Json<HashMap<String, String>>,
) -> impl Responder {
    let agent_type = body.get("agent_type").unwrap();
    let manager = manager.lock().unwrap();
    match manager.create_agent(agent_type) {
        Ok(agent) => HttpResponse::Ok().json(agent),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

async fn connect_agent(
    manager: web::Data<Mutex<AIAgentManager>>,
    agent_id: web::Path<String>,
    credentials: web::Json<HashMap<String, String>>,
) -> impl Responder {
    let manager = manager.lock().unwrap();
    match manager.connect_agent(&agent_id, &credentials).await {
        Ok(success) => HttpResponse::Ok().json(success),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn query_agent(
    manager: web::Data<Mutex<AIAgentManager>>,
    agent_id: web::Path<String>,
    body: web::Json<HashMap<String, String>>,
) -> impl Responder {
    let prompt = body.get("prompt").unwrap();
    let context = body.get("context").map(|s| s.as_str());
    let manager = manager.lock().unwrap();
    match manager.query_agent(&agent_id, prompt, context).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
