
use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::services::rule_bank::{
    AIRuleBankService, AIRule, AIRuleType, RuleMetadata, 
    MemoryBankEntry, MemoryType
};

#[derive(Deserialize)]
pub struct CreateRuleRequest {
    pub title: String,
    pub description: String,
    pub content: String,
    pub rule_type: AIRuleType,
    pub priority: Option<i32>,
    pub tags: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct RuleResponse {
    pub success: bool,
    pub data: Option<AIRule>,
    pub message: String,
}

#[derive(Serialize)]
pub struct RulesListResponse {
    pub success: bool,
    pub data: Vec<AIRule>,
    pub count: usize,
    pub message: String,
}


pub async fn store_rule(
    rule_bank: web::Data<AIRuleBankService>,
    req: web::Json<CreateRuleRequest>,
) -> Result<HttpResponse> {
    let rule = AIRule {
        id: Uuid::new_v4(),
        title: req.title.clone(),
        description: req.description.clone(),
        content: req.content.clone(),
        rule_type: req.rule_type.clone(),
        metadata: RuleMetadata::default(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        version: 1,
        priority: req.priority.unwrap_or(0),
        is_active: true,
        tags: req.tags.clone().unwrap_or_default(),
    };

    match rule_bank.store_rule(rule.clone()).await {
        Ok(id) => Ok(HttpResponse::Created().json(RuleResponse {
            success: true,
            data: Some(rule),
            message: format!("Rule created successfully with ID: {}", id),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(RuleResponse {
            success: false,
            data: None,
            message: format!("Failed to create rule: {}", e),
        })),
    }
}


pub async fn get_rules_for_agent(
    rule_bank: web::Data<AIRuleBankService>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let agent_type = path.into_inner();
    
    match rule_bank.get_rules_for_agent(&agent_type, None).await {
        Ok(rules) => {
            let count = rules.len();
            Ok(HttpResponse::Ok().json(RulesListResponse {
                success: true,
                count,
                data: rules,
                message: format!("Retrieved {} rules for agent type: {}", count, agent_type),
            }))
        },
        Err(e) => Ok(HttpResponse::InternalServerError().json(RulesListResponse {
            success: false,
            count: 0,
            data: vec![],
            message: format!("Failed to retrieve rules: {}", e),
        })),
    }
}


pub async fn health_check(
    rule_bank: web::Data<AIRuleBankService>,
) -> Result<HttpResponse> {
    let stats = rule_bank.get_cache_stats().await;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "service": "ai_rule_bank",
        "status": "healthy",
        "cache_performance": {
            "hit_rate": stats.hit_rate,
            "total_operations": stats.cache_hits + stats.cache_misses,
        },
        "timestamp": chrono::Utc::now()
    })))
}

pub fn configure_rule_bank_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/rule-bank")
            .route("/health", web::get().to(health_check))
            .route("/rules", web::post().to(store_rule))
            .route("/rules/agent/{agent_type}", web::get().to(get_rules_for_agent))
    );
}