use actix_web::{web, HttpResponse, HttpRequest, Result};
use serde_json::Value;

/// GitHub webhook handler
/// POST /api/webhooks/github
pub async fn handle_github_webhook(
    req: HttpRequest,
    payload: web::Json<Value>,
) -> Result<HttpResponse> {
    tracing::info!("Received GitHub webhook");

    // Verify webhook signature
    if let Some(signature) = req.headers().get("X-Hub-Signature-256") {
        let signature_str = signature.to_str()
            .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid signature format"))?;

        // TODO: Verify HMAC-SHA256 signature
        tracing::debug!("GitHub signature present: {} chars", signature_str.len());
    } else {
        return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Missing X-Hub-Signature-256 header"
        })));
    }

    // Parse event type
    let event_type = req.headers()
        .get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing X-GitHub-Event header"))?;

    tracing::info!("GitHub event type: {}", event_type);

    let payload_value = payload.into_inner();

    // Process webhook asynchronously
    let event_type_owned = event_type.to_string();
    tokio::spawn(async move {
        if let Err(e) = process_github_webhook(&event_type_owned, payload_value).await {
            tracing::error!("Failed to process GitHub webhook: {}", e);
        }
    });

    // Return 200 OK immediately
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "accepted"
    })))
}

/// Process GitHub webhook events
pub async fn process_github_webhook(
    event_type: &str,
    payload: Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Processing GitHub webhook: event={}", event_type);

    match event_type {
        "push" => handle_push_event(payload).await?,
        "pull_request" => handle_pull_request_event(payload).await?,
        "release" => handle_release_event(payload).await?,
        _ => {
            tracing::debug!("Unhandled GitHub event type: {}", event_type);
        }
    }

    Ok(())
}

async fn handle_push_event(
    payload: Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("GitHub push event");

    // Extract repository information
    if let Some(repository) = payload.get("repository") {
        let repo_id = repository.get("id");
        let repo_name = repository.get("name");

        tracing::debug!("Repository: {:?}, ID: {:?}", repo_name, repo_id);
    }

    // Extract commits
    if let Some(commits) = payload.get("commits").and_then(|c| c.as_array()) {
        tracing::info!("Push contains {} commit(s)", commits.len());

        // TODO: Trigger incremental sync for affected files
        // TODO: Clear cache for this repository
        // TODO: Update indexed documents
    }

    Ok(())
}

async fn handle_pull_request_event(
    payload: Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("GitHub pull request event");

    if let Some(pull_request) = payload.get("pull_request") {
        let action = payload.get("action").and_then(|a| a.as_str());
        let pr_number = pull_request.get("number");

        tracing::debug!("PR action: {:?}, Number: {:?}", action, pr_number);

        // TODO: Handle pull request events if configured
    }

    Ok(())
}

async fn handle_release_event(
    payload: Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("GitHub release event");

    if let Some(release) = payload.get("release") {
        let action = payload.get("action").and_then(|a| a.as_str());
        let tag_name = release.get("tag_name");

        tracing::debug!("Release action: {:?}, Tag: {:?}", action, tag_name);

        // TODO: Handle release events
    }

    Ok(())
}
