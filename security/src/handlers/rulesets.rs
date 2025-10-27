use crate::services::rulesets::{self, CreateRulesetRequest, UpdateRulesetRequest};
use crate::errors::ServiceError;
use actix_web::{web, HttpResponse, Responder};
use uuid::Uuid;
use sqlx::PgPool;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/rulesets")
            .route("", web::post().to(create_ruleset))
            .route("", web::get().to(list_rulesets))
            .route("/{ruleset_id}", web::get().to(get_ruleset))
            .route("/{ruleset_id}", web::put().to(update_ruleset))
            .route("/{ruleset_id}", web::delete().to(delete_ruleset))
            .route("/{ruleset_id}/rules", web::post().to(add_rule))
            .route("/{ruleset_id}/agents/{agent_id}", web::post().to(connect_agent))
            .route("/{ruleset_id}/agents/{agent_id}", web::delete().to(disconnect_agent)),
    );
}

async fn create_ruleset(
    pool: web::Data<PgPool>,
    user_id: web::ReqData<Uuid>,
    req: web::Json<CreateRulesetRequest>,
) -> impl Responder {
    let result = rulesets::create_ruleset(&pool, user_id.into_inner(), req.into_inner()).await;
    
    match result {
        Ok(ruleset) => HttpResponse::Created().json(ruleset),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

async fn list_rulesets(
    pool: web::Data<PgPool>,
    user_id: web::ReqData<Uuid>,
) -> impl Responder {
    let result = rulesets::list_rulesets(&pool, user_id.into_inner()).await;
    
    match result {
        Ok(rulesets) => HttpResponse::Ok().json(rulesets),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

async fn get_ruleset(
    pool: web::Data<PgPool>,
    user_id: web::ReqData<Uuid>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let ruleset_id = path.into_inner();
    let result = rulesets::get_ruleset(&pool, user_id.into_inner(), ruleset_id).await;
    
    match result {
        Ok(ruleset) => HttpResponse::Ok().json(ruleset),
        Err(ServiceError::NotFound) => HttpResponse::NotFound().finish(),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

async fn update_ruleset(
    pool: web::Data<PgPool>,
    user_id: web::ReqData<Uuid>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateRulesetRequest>,
) -> impl Responder {
    let ruleset_id = path.into_inner();
    let result = rulesets::update_ruleset(&pool, user_id.into_inner(), ruleset_id, req.into_inner()).await;
    
    match result {
        Ok(ruleset) => HttpResponse::Ok().json(ruleset),
        Err(ServiceError::NotFound) => HttpResponse::NotFound().finish(),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

async fn delete_ruleset(
    pool: web::Data<PgPool>,
    user_id: web::ReqData<Uuid>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let ruleset_id = path.into_inner();
    let result = rulesets::delete_ruleset(&pool, user_id.into_inner(), ruleset_id).await;
    
    match result {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(ServiceError::NotFound) => HttpResponse::NotFound().finish(),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[derive(serde::Deserialize)]
struct AddRuleRequest {
    content: String,
}

async fn add_rule(
    pool: web::Data<PgPool>,
    user_id: web::ReqData<Uuid>,
    path: web::Path<Uuid>,
    req: web::Json<AddRuleRequest>,
) -> impl Responder {
    let ruleset_id = path.into_inner();
    let result = rulesets::add_rule(
        &pool, 
        user_id.into_inner(), 
        ruleset_id, 
        req.content.clone(),
    ).await;
    
    match result {
        Ok(rule) => HttpResponse::Created().json(rule),
        Err(ServiceError::NotFound) => HttpResponse::NotFound().finish(),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

async fn connect_agent(
    pool: web::Data<PgPool>,
    user_id: web::ReqData<Uuid>,
    path: web::Path<(Uuid, Uuid)>,
) -> impl Responder {
    let (ruleset_id, agent_id) = path.into_inner();
    let result = rulesets::connect_agent_to_ruleset(&pool, user_id.into_inner(), agent_id, ruleset_id).await;
    
    match result {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(ServiceError::NotFound) => HttpResponse::NotFound().finish(),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

async fn disconnect_agent(
    pool: web::Data<PgPool>,
    user_id: web::ReqData<Uuid>,
    path: web::Path<(Uuid, Uuid)>,
) -> impl Responder {
    let (ruleset_id, agent_id) = path.into_inner();
    let result = rulesets::disconnect_agent_from_ruleset(&pool, user_id.into_inner(), agent_id, ruleset_id).await;
    
    match result {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(ServiceError::NotFound) => HttpResponse::NotFound().finish(),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}