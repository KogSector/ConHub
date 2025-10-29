use actix_web::{web, HttpResponse, HttpRequest};
use validator::Validate;
use serde_json::json;
use uuid::Uuid;

use conhub_models::billing::*;
use crate::services::billing::BillingService;
use crate::errors::ServiceError;

const DEMO_USER_ID: &str = "550e8400-e29b-41d4-a716-446655440000";

pub async fn get_subscription_plans() -> Result<HttpResponse, ServiceError> {
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

pub async fn get_billing_dashboard() -> Result<HttpResponse, ServiceError> {
    let billing_service = BillingService::new();
    
    let user_id = uuid::Uuid::parse_str(DEMO_USER_ID)
        .map_err(|e| ServiceError::ParseError(format!("Invalid UUID: {}", e)))?;
    
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


#[derive(serde::Deserialize, Validate)]
pub struct CreateCustomerRequest {
    #[validate(email)]
    pub email: String,
    pub name: String,
}

// Using CreateSubscriptionRequest from conhub_models::billing

#[derive(serde::Deserialize, Validate)]
pub struct CreatePaymentIntentRequest {
    pub amount: i64,
    pub currency: String,
    pub customer_id: String,
}


pub async fn create_customer(request: web::Json<CreateCustomerRequest>) -> Result<HttpResponse, ServiceError> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    let billing_service = BillingService::new();
    let user_id = Uuid::new_v4(); 

    match billing_service.create_customer(user_id, &request.email, &request.name).await {
        Ok(customer_id) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "customer_id": customer_id
        }))),
        Err(e) => {
            log::error!("Failed to create customer: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to create customer"
            })))
        }
    }
}


pub async fn create_payment_intent(request: web::Json<CreatePaymentIntentRequest>) -> Result<HttpResponse, ServiceError> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    let billing_service = BillingService::new();

    match billing_service.create_payment_intent(request.amount, &request.currency).await {
        Ok(payment_intent_id) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "payment_intent_id": payment_intent_id
        }))),
        Err(e) => {
            log::error!("Failed to create payment intent: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to create payment intent"
            })))
        }
    }
}


pub async fn create_setup_intent(request: web::Json<serde_json::Value>) -> Result<HttpResponse, ServiceError> {
    let customer_id = request.get("customer_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ServiceError::ValidationError("customer_id is required".to_string()))?;

    let billing_service = BillingService::new();

    match billing_service.create_setup_intent(customer_id).await {
        Ok(setup_intent_id) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "setup_intent_id": setup_intent_id
        }))),
        Err(e) => {
            log::error!("Failed to create setup intent: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to create setup intent"
            })))
        }
    }
}


pub async fn create_subscription(request: web::Json<CreateSubscriptionRequest>) -> Result<HttpResponse, ServiceError> {
    if let Err(validation_errors) = request.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Validation failed",
            "details": validation_errors
        })));
    }

    // For demo purposes, using a fixed user ID - in real app, extract from JWT token
    let user_id = match Uuid::parse_str(DEMO_USER_ID) {
        Ok(id) => id,
        Err(_) => return Ok(HttpResponse::InternalServerError().json(json!({
            "error": "Invalid demo user ID"
        })))
    };

    let billing_service = BillingService::new();

    match billing_service.create_subscription(user_id, &request).await {
        Ok(subscription) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "subscription_id": subscription.id,
            "status": subscription.status,
            "plan_id": subscription.plan_id
        }))),
        Err(e) => {
            log::error!("Failed to create subscription: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to create subscription"
            })))
        }
    }
}


pub async fn cancel_subscription(path: web::Path<String>) -> Result<HttpResponse, ServiceError> {
    let subscription_id = path.into_inner();
    let billing_service = BillingService::new();

    match billing_service.cancel_subscription(&subscription_id).await {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Subscription cancelled successfully"
        }))),
        Err(e) => {
            log::error!("Failed to cancel subscription: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to cancel subscription"
            })))
        }
    }
}


pub async fn get_payment_methods(path: web::Path<String>) -> Result<HttpResponse, ServiceError> {
    let customer_id = path.into_inner();
    let billing_service = BillingService::new();

    match billing_service.get_payment_methods(&customer_id).await {
        Ok(payment_methods) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "payment_methods": payment_methods
        }))),
        Err(e) => {
            log::error!("Failed to get payment methods: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to get payment methods"
            })))
        }
    }
}


pub async fn get_invoices(path: web::Path<String>) -> Result<HttpResponse, ServiceError> {
    let customer_id_str = path.into_inner();
    let customer_id = match Uuid::parse_str(&customer_id_str) {
        Ok(id) => id,
        Err(_) => return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Invalid customer ID format"
        })))
    };
    
    let billing_service = BillingService::new();

    match billing_service.get_invoices(customer_id).await {
        Ok(invoices) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "invoices": invoices
        }))),
        Err(e) => {
            log::error!("Failed to get invoices: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to get invoices"
            })))
        }
    }
}


pub async fn handle_stripe_webhook(req: HttpRequest, body: web::Bytes) -> Result<HttpResponse, ServiceError> {
    let signature = req.headers()
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ServiceError::ValidationError("Missing stripe-signature header".to_string()))?;

    let payload = std::str::from_utf8(&body)
        .map_err(|_| ServiceError::ValidationError("Invalid UTF-8 in request body".to_string()))?;

    let billing_service = BillingService::new();

    match billing_service.handle_webhook_event(payload, signature).await {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "success": true
        }))),
        Err(e) => {
            log::error!("Failed to handle webhook: {}", e);
            Ok(HttpResponse::BadRequest().json(json!({
                "error": "Failed to handle webhook"
            })))
        }
    }
}

pub async fn get_subscription(path: web::Path<String>) -> Result<HttpResponse, ServiceError> {
    let user_id = match Uuid::parse_str(&path) {
        Ok(id) => id,
        Err(_) => return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Invalid user ID format"
        })))
    };

    let billing_service = BillingService::new();
    match billing_service.get_subscription(user_id).await {
        Ok(Some(subscription)) => Ok(HttpResponse::Ok().json(subscription)),
        Ok(None) => Ok(HttpResponse::NotFound().json(json!({
            "error": "No subscription found for user"
        }))),
        Err(e) => {
            log::error!("Failed to get subscription: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to get subscription"
            })))
        }
    }
}

pub async fn add_payment_method(request: web::Json<CreatePaymentMethodRequest>) -> Result<HttpResponse, ServiceError> {
    let user_id = match Uuid::parse_str(DEMO_USER_ID) {
        Ok(id) => id,
        Err(_) => return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Invalid user ID format"
        })))
    };

    let billing_service = BillingService::new();
    match billing_service.add_payment_method(user_id, request.into_inner()).await {
        Ok(payment_method) => Ok(HttpResponse::Created().json(payment_method)),
        Err(e) => {
            log::error!("Failed to add payment method: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to add payment method"
            })))
        }
    }
}

pub fn configure_billing_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/billing")
            .route("/plans", web::get().to(get_subscription_plans))
            .route("/dashboard", web::get().to(get_billing_dashboard))
            .route("/customers", web::post().to(create_customer))
            .route("/payment-intents", web::post().to(create_payment_intent))
            .route("/setup-intents", web::post().to(create_setup_intent))
            .route("/subscriptions", web::post().to(create_subscription))
            .route("/subscriptions/{subscription_id}", web::delete().to(cancel_subscription))
            .route("/customers/{customer_id}/payment-methods", web::get().to(get_payment_methods))
            .route("/customers/{customer_id}/invoices", web::get().to(get_invoices))
            .route("/webhooks/stripe", web::post().to(handle_stripe_webhook))
    );
}