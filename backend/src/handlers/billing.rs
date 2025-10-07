use actix_web::{web, HttpResponse, Result};
use validator::Validate;

use crate::models::billing::*;
use crate::services::billing::BillingService;

pub async fn get_subscription_plans() -> Result<HttpResponse> {
    let billing_service = BillingService::new();
    match billing_service.get_subscription_plans().await {
        Ok(plans) => Ok(HttpResponse::Ok().json(plans)),
        Err(e) => {
            log::error!("Failed to get subscription plans: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get subscription plans"
            })))
        }
    }
}

pub async fn get_billing_dashboard() -> Result<HttpResponse> {
    let billing_service = BillingService::new();
    // Mock user ID for development
    let user_id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    
    match billing_service.get_billing_dashboard(user_id).await {
        Ok(dashboard) => Ok(HttpResponse::Ok().json(dashboard)),
        Err(e) => {
            log::error!("Failed to get billing dashboard: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get billing dashboard"
            })))
        }
    }
}

pub fn configure_billing_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/billing")
            .route("/plans", web::get().to(get_subscription_plans))
            .route("/dashboard", web::get().to(get_billing_dashboard))
    );
}