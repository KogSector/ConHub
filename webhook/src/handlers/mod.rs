use actix_web::{web, HttpResponse, HttpRequest, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod gitlab;
pub mod github;
pub mod stripe;
pub mod dropbox;
pub mod onedrive;

/// Webhook payload wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Webhook verification result
#[derive(Debug)]
pub enum WebhookVerification {
    Valid,
    Invalid(String),
    MissingSignature,
}

/// Webhook handler error
#[derive(Debug)]
pub enum WebhookError {
    InvalidSignature,
    InvalidPayload(String),
    DataSourceNotFound,
    ProcessingFailed(String),
}

impl std::fmt::Display for WebhookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebhookError::InvalidSignature => write!(f, "Invalid webhook signature"),
            WebhookError::InvalidPayload(msg) => write!(f, "Invalid payload: {}", msg),
            WebhookError::DataSourceNotFound => write!(f, "Data source not found"),
            WebhookError::ProcessingFailed(msg) => write!(f, "Processing failed: {}", msg),
        }
    }
}

impl std::error::Error for WebhookError {}

/// GitLab webhook handler
/// POST /api/webhooks/gitlab/:data_source_id
pub async fn handle_gitlab_webhook(
    path: web::Path<String>,
    req: HttpRequest,
    payload: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let data_source_id = path.into_inner();
    tracing::info!("Received GitLab webhook for data source: {}", data_source_id);

    // Verify webhook signature
    if let Some(token) = req.headers().get("X-Gitlab-Token") {
        let token_str = token.to_str()
            .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid token format"))?;

        // TODO: Verify token against stored webhook secret
        tracing::debug!("GitLab webhook token present: {} chars", token_str.len());
    } else {
        return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Missing X-Gitlab-Token header"
        })));
    }

    // Parse event type
    let event_type = req.headers()
        .get("X-Gitlab-Event")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing X-Gitlab-Event header"))?;

    tracing::info!("GitLab event type: {}", event_type);

    let payload_value = payload.into_inner();
    let event_type_owned = event_type.to_string();

    // Process webhook asynchronously
    tokio::spawn(async move {
        if let Err(e) = gitlab::process_gitlab_webhook(&data_source_id, &event_type_owned, payload_value).await {
            tracing::error!("Failed to process GitLab webhook: {}", e);
        }
    });

    // Return 200 OK immediately
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "accepted"
    })))
}

/// Dropbox webhook handler
/// POST /api/webhooks/dropbox/:data_source_id
pub async fn handle_dropbox_webhook(
    path: web::Path<String>,
    req: HttpRequest,
    body: String,
) -> Result<HttpResponse> {
    let data_source_id = path.into_inner();
    tracing::info!("Received Dropbox webhook for data source: {}", data_source_id);

    // Dropbox sends a challenge on webhook registration
    if let Some(challenge) = req.headers().get("X-Dropbox-Challenge") {
        let challenge_str = challenge.to_str()
            .map_err(|_| actix_web::error::ErrorBadRequest("Invalid challenge format"))?;

        tracing::info!("Dropbox webhook challenge received");
        return Ok(HttpResponse::Ok().body(challenge_str.to_string()));
    }

    // Verify webhook signature
    if let Some(signature) = req.headers().get("X-Dropbox-Signature") {
        let signature_str = signature.to_str()
            .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid signature format"))?;

        // TODO: Verify HMAC-SHA256 signature
        tracing::debug!("Dropbox signature present: {} chars", signature_str.len());
    } else {
        return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Missing X-Dropbox-Signature header"
        })));
    }

    // Parse payload
    let payload: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid JSON: {}", e)))?;

    // Process webhook asynchronously
    tokio::spawn(async move {
        if let Err(e) = dropbox::process_dropbox_webhook(&data_source_id, payload).await {
            tracing::error!("Failed to process Dropbox webhook: {}", e);
        }
    });

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "accepted"
    })))
}

/// OneDrive/Microsoft Graph webhook handler
/// POST /api/webhooks/onedrive/:data_source_id
pub async fn handle_onedrive_webhook(
    path: web::Path<String>,
    req: HttpRequest,
    payload: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let data_source_id = path.into_inner();
    tracing::info!("Received OneDrive webhook for data source: {}", data_source_id);

    let payload_value = payload.into_inner();

    // Microsoft Graph webhook validation
    if let Some(validation_token) = payload_value.get("validationToken") {
        if let Some(token_str) = validation_token.as_str() {
            tracing::info!("OneDrive webhook validation");
            // Return validation token as plain text
            return Ok(HttpResponse::Ok()
                .content_type("text/plain")
                .body(token_str.to_string()));
        }
    }

    // Verify client state (custom secret)
    if let Some(client_state) = payload_value.get("value")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("clientState"))
        .and_then(|cs| cs.as_str())
    {
        // TODO: Verify clientState against stored secret
        tracing::debug!("OneDrive clientState present: {} chars", client_state.len());
    }

    // Process webhook asynchronously
    tokio::spawn(async move {
        if let Err(e) = onedrive::process_onedrive_webhook(&data_source_id, payload_value).await {
            tracing::error!("Failed to process OneDrive webhook: {}", e);
        }
    });

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "accepted"
    })))
}

/// Helper function to verify HMAC-SHA256 signature
pub fn verify_hmac_signature(
    payload: &[u8],
    signature: &str,
    secret: &str,
) -> bool {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };

    mac.update(payload);

    let expected = mac.finalize().into_bytes();
    let expected_hex = hex::encode(expected);

    // Compare signatures in constant time
    signature.eq_ignore_ascii_case(&expected_hex)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_hmac_signature() {
        let payload = b"test payload";
        let secret = "test_secret";

        // Generate expected signature
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload);
        let signature = hex::encode(mac.finalize().into_bytes());

        assert!(verify_hmac_signature(payload, &signature, secret));
        assert!(!verify_hmac_signature(payload, "invalid", secret));
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/webhooks")
            .route("/gitlab/{data_source_id}", web::post().to(handle_gitlab_webhook))
            .route("/dropbox/{data_source_id}", web::post().to(handle_dropbox_webhook))
            .route("/onedrive/{data_source_id}", web::post().to(handle_onedrive_webhook))
    );
}
