use actix_web::{web, HttpResponse, HttpRequest, Result};
use serde_json::Value;

/// Stripe webhook handler
/// POST /api/webhooks/stripe
pub async fn handle_stripe_webhook(
    req: HttpRequest,
    body: String,
) -> Result<HttpResponse> {
    tracing::info!("Received Stripe webhook");

    // Verify webhook signature
    if let Some(signature) = req.headers().get("Stripe-Signature") {
        let signature_str = signature.to_str()
            .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid signature format"))?;

        // TODO: Verify Stripe webhook signature
        tracing::debug!("Stripe signature present: {} chars", signature_str.len());
    } else {
        return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Missing Stripe-Signature header"
        })));
    }

    // Parse payload
    let payload: Value = serde_json::from_str(&body)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid JSON: {}", e)))?;

    // Extract event type
    let event_type = payload.get("type")
        .and_then(|t| t.as_str())
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing event type"))?;

    tracing::info!("Stripe event type: {}", event_type);

    // Process webhook asynchronously
    let event_type_owned = event_type.to_string();
    tokio::spawn(async move {
        if let Err(e) = process_stripe_webhook(&event_type_owned, payload).await {
            tracing::error!("Failed to process Stripe webhook: {}", e);
        }
    });

    // Return 200 OK immediately
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "accepted"
    })))
}

/// Process Stripe webhook events
pub async fn process_stripe_webhook(
    event_type: &str,
    payload: Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Processing Stripe webhook: event={}", event_type);

    match event_type {
        "customer.subscription.created" => handle_subscription_created(payload).await?,
        "customer.subscription.updated" => handle_subscription_updated(payload).await?,
        "customer.subscription.deleted" => handle_subscription_deleted(payload).await?,
        "invoice.payment_succeeded" => handle_payment_succeeded(payload).await?,
        "invoice.payment_failed" => handle_payment_failed(payload).await?,
        _ => {
            tracing::debug!("Unhandled Stripe event type: {}", event_type);
        }
    }

    Ok(())
}

async fn handle_subscription_created(
    payload: Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Stripe subscription created");

    if let Some(data) = payload.get("data").and_then(|d| d.get("object")) {
        let customer_id = data.get("customer").and_then(|c| c.as_str());
        let subscription_id = data.get("id").and_then(|i| i.as_str());

        tracing::debug!("Customer: {:?}, Subscription: {:?}", customer_id, subscription_id);

        // TODO: Update user subscription status
        // TODO: Enable premium features
    }

    Ok(())
}

async fn handle_subscription_updated(
    payload: Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Stripe subscription updated");

    if let Some(data) = payload.get("data").and_then(|d| d.get("object")) {
        let customer_id = data.get("customer").and_then(|c| c.as_str());
        let subscription_id = data.get("id").and_then(|i| i.as_str());
        let status = data.get("status").and_then(|s| s.as_str());

        tracing::debug!("Customer: {:?}, Subscription: {:?}, Status: {:?}", 
                       customer_id, subscription_id, status);

        // TODO: Update subscription status
        // TODO: Handle plan changes
    }

    Ok(())
}

async fn handle_subscription_deleted(
    payload: Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Stripe subscription deleted");

    if let Some(data) = payload.get("data").and_then(|d| d.get("object")) {
        let customer_id = data.get("customer").and_then(|c| c.as_str());
        let subscription_id = data.get("id").and_then(|i| i.as_str());

        tracing::debug!("Customer: {:?}, Subscription: {:?}", customer_id, subscription_id);

        // TODO: Disable premium features
        // TODO: Update user subscription status
    }

    Ok(())
}

async fn handle_payment_succeeded(
    payload: Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Stripe payment succeeded");

    if let Some(data) = payload.get("data").and_then(|d| d.get("object")) {
        let customer_id = data.get("customer").and_then(|c| c.as_str());
        let amount = data.get("amount_paid").and_then(|a| a.as_i64());

        tracing::debug!("Customer: {:?}, Amount: {:?}", customer_id, amount);

        // TODO: Record successful payment
        // TODO: Send confirmation email
    }

    Ok(())
}

async fn handle_payment_failed(
    payload: Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Stripe payment failed");

    if let Some(data) = payload.get("data").and_then(|d| d.get("object")) {
        let customer_id = data.get("customer").and_then(|c| c.as_str());
        let amount = data.get("amount_due").and_then(|a| a.as_i64());

        tracing::debug!("Customer: {:?}, Amount: {:?}", customer_id, amount);

        // TODO: Handle failed payment
        // TODO: Send notification email
        // TODO: Update subscription status if needed
    }

    Ok(())
}
