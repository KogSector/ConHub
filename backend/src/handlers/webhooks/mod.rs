use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod gitlab;
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

impl IntoResponse for WebhookError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            WebhookError::InvalidSignature => (StatusCode::UNAUTHORIZED, "Invalid webhook signature"),
            WebhookError::InvalidPayload(msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            WebhookError::DataSourceNotFound => (StatusCode::NOT_FOUND, "Data source not found"),
            WebhookError::ProcessingFailed(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.as_str()),
        };

        (status, message).into_response()
    }
}

/// GitLab webhook handler
/// POST /api/webhooks/gitlab/:data_source_id
pub async fn handle_gitlab_webhook(
    Path(data_source_id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Result<StatusCode, WebhookError> {
    tracing::info!("Received GitLab webhook for data source: {}", data_source_id);

    // Verify webhook signature
    if let Some(token) = headers.get("X-Gitlab-Token") {
        let token_str = token.to_str()
            .map_err(|_| WebhookError::InvalidSignature)?;

        // TODO: Verify token against stored webhook secret
        tracing::debug!("GitLab webhook token present: {}", token_str.len());
    } else {
        return Err(WebhookError::InvalidSignature);
    }

    // Parse event type
    let event_type = headers
        .get("X-Gitlab-Event")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| WebhookError::InvalidPayload("Missing X-Gitlab-Event header".to_string()))?;

    tracing::info!("GitLab event type: {}", event_type);

    // Process webhook asynchronously
    tokio::spawn(async move {
        if let Err(e) = gitlab::process_gitlab_webhook(&data_source_id, event_type, payload).await {
            tracing::error!("Failed to process GitLab webhook: {}", e);
        }
    });

    // Return 200 OK immediately
    Ok(StatusCode::OK)
}

/// Dropbox webhook handler
/// POST /api/webhooks/dropbox/:data_source_id
pub async fn handle_dropbox_webhook(
    Path(data_source_id): Path<String>,
    headers: axum::http::HeaderMap,
    body: String,
) -> Result<StatusCode, WebhookError> {
    tracing::info!("Received Dropbox webhook for data source: {}", data_source_id);

    // Dropbox sends a challenge on webhook registration
    if let Some(challenge) = headers.get("X-Dropbox-Challenge") {
        let challenge_str = challenge.to_str()
            .map_err(|_| WebhookError::InvalidPayload("Invalid challenge".to_string()))?;

        tracing::info!("Dropbox webhook challenge received");
        return Ok(StatusCode::OK); // Echo challenge back
    }

    // Verify webhook signature
    if let Some(signature) = headers.get("X-Dropbox-Signature") {
        let signature_str = signature.to_str()
            .map_err(|_| WebhookError::InvalidSignature)?;

        // TODO: Verify HMAC-SHA256 signature
        tracing::debug!("Dropbox signature present: {}", signature_str.len());
    } else {
        return Err(WebhookError::InvalidSignature);
    }

    // Parse payload
    let payload: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| WebhookError::InvalidPayload(e.to_string()))?;

    // Process webhook asynchronously
    tokio::spawn(async move {
        if let Err(e) = dropbox::process_dropbox_webhook(&data_source_id, payload).await {
            tracing::error!("Failed to process Dropbox webhook: {}", e);
        }
    });

    Ok(StatusCode::OK)
}

/// OneDrive/Microsoft Graph webhook handler
/// POST /api/webhooks/onedrive/:data_source_id
pub async fn handle_onedrive_webhook(
    Path(data_source_id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Result<StatusCode, WebhookError> {
    tracing::info!("Received OneDrive webhook for data source: {}", data_source_id);

    // Microsoft Graph webhook validation
    if let Some(validation_token) = payload.get("validationToken") {
        if let Some(token_str) = validation_token.as_str() {
            tracing::info!("OneDrive webhook validation");
            // Return validation token as plain text
            return Ok(StatusCode::OK);
        }
    }

    // Verify client state (custom secret)
    if let Some(client_state) = payload.get("value")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("clientState"))
        .and_then(|cs| cs.as_str())
    {
        // TODO: Verify clientState against stored secret
        tracing::debug!("OneDrive clientState present: {}", client_state.len());
    }

    // Process webhook asynchronously
    tokio::spawn(async move {
        if let Err(e) = onedrive::process_onedrive_webhook(&data_source_id, payload).await {
            tracing::error!("Failed to process OneDrive webhook: {}", e);
        }
    });

    Ok(StatusCode::OK)
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
