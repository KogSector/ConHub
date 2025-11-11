use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::PgPool;
use tracing::{info, error};

use crate::connectors::{ConnectorManager, ConnectRequest, SyncRequest, OAuthCallbackData};
use conhub_models::auth::Claims;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectRequestBody {
    pub connector_type: String,
    pub account_name: Option<String>,
    pub credentials: std::collections::HashMap<String, String>,
    pub settings: Option<std::collections::HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncRequestBody {
    pub account_id: String,
    pub incremental: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthCallbackBody {
    pub account_id: String,
    pub code: String,
    pub state: String,
}

/// List available connector types
pub async fn list_connectors(
    manager: web::Data<ConnectorManager>,
) -> Result<HttpResponse> {
    let connectors = manager.available_connectors();
    
    let connector_info: Vec<serde_json::Value> = connectors.iter()
        .map(|c| serde_json::json!({
            "type": c.as_str(),
            "name": format!("{:?}", c),
        }))
        .collect();
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "connectors": connector_info,
    })))
}

/// Connect a new data source
pub async fn connect_source(
    manager: web::Data<ConnectorManager>,
    claims: web::ReqData<Claims>,
    body: web::Json<ConnectRequestBody>,
) -> Result<HttpResponse> {
    let user_id = claims.sub;
    
    info!("🔌 User {} connecting to {}", user_id, body.connector_type);
    
    // Parse connector type
    let connector_type = match body.connector_type.as_str() {
        "local_file" => crate::connectors::ConnectorType::LocalFile,
        "github" => crate::connectors::ConnectorType::GitHub,
        "google_drive" => crate::connectors::ConnectorType::GoogleDrive,
        _ => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": format!("Unsupported connector type: {}", body.connector_type),
            })));
        }
    };
    
    let request = ConnectRequest {
        connector_type,
        account_name: body.account_name.clone(),
        credentials: body.credentials.clone(),
        settings: body.settings.clone(),
    };
    
    match manager.connect(user_id, request).await {
        Ok(account) => {
            info!("✅ Successfully connected account: {}", account.id);
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "account": account,
            })))
        }
        Err(e) => {
            error!("Failed to connect: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": e.to_string(),
            })))
        }
    }
}

/// Complete OAuth authentication
pub async fn complete_oauth_callback(
    manager: web::Data<ConnectorManager>,
    body: web::Json<OAuthCallbackBody>,
) -> Result<HttpResponse> {
    let account_id = match Uuid::parse_str(&body.account_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": "Invalid account ID",
            })));
        }
    };
    
    let callback_data = OAuthCallbackData {
        code: body.code.clone(),
        state: body.state.clone(),
    };
    
    match manager.complete_oauth(account_id, callback_data).await {
        Ok(account) => {
            info!("✅ OAuth completed for account: {}", account.id);
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "account": account,
            })))
        }
        Err(e) => {
            error!("Failed to complete OAuth: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": e.to_string(),
            })))
        }
    }
}

/// Sync a connected data source
pub async fn sync_source(
    manager: web::Data<ConnectorManager>,
    embedding_client: web::Data<crate::services::EmbeddingClient>,
    claims: web::ReqData<Claims>,
    body: web::Json<SyncRequestBody>,
) -> Result<HttpResponse> {
    let user_id = claims.sub;
    
    let account_id = match Uuid::parse_str(&body.account_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": "Invalid account ID",
            })));
        }
    };
    
    info!("🔄 User {} syncing account {}", user_id, account_id);
    
    let request = SyncRequest {
        account_id,
        incremental: body.incremental,
        filters: None,
    };
    
    match manager.sync(account_id, request).await {
        Ok((sync_result, documents)) => {
            info!("✅ Sync completed: {} documents", documents.len());
            
            // Send documents to embedding service
            if !documents.is_empty() {
                match embedding_client.embed_documents(documents).await {
                    Ok(embed_result) => {
                        info!("📊 Embedding completed successfully");
                    }
                    Err(e) => {
                        error!("Failed to send documents to embedding service: {}", e);
                    }
                }
            }
            
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "sync_result": sync_result,
                "documents_count": sync_result.total_documents,
            })))
        }
        Err(e) => {
            error!("Failed to sync: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": e.to_string(),
            })))
        }
    }
}

/// Disconnect a data source
pub async fn disconnect_source(
    manager: web::Data<ConnectorManager>,
    claims: web::ReqData<Claims>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let user_id = claims.sub;
    let account_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": "Invalid account ID",
            })));
        }
    };
    
    info!("🔌 User {} disconnecting account {}", user_id, account_id);
    
    match manager.disconnect(account_id).await {
        Ok(_) => {
            info!("✅ Successfully disconnected account: {}", account_id);
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "Account disconnected successfully",
            })))
        }
        Err(e) => {
            error!("Failed to disconnect: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": e.to_string(),
            })))
        }
    }
}

/// List connected accounts for the authenticated user
pub async fn list_connected_accounts(
    pool: web::Data<Option<PgPool>>,
    claims: web::ReqData<Claims>,
) -> Result<HttpResponse> {
    let user_id = claims.sub;
    
    let pool = match pool.get_ref() {
        Some(p) => p,
        None => {
            return Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "success": false,
                "error": "Database not available",
            })));
        }
    };
    
    match sqlx::query!(
        r#"
        SELECT id, connector_type, account_name, account_identifier, status, last_sync_at, created_at
        FROM connected_accounts
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await
    {
        Ok(accounts) => {
            let accounts_json: Vec<serde_json::Value> = accounts.iter()
                .map(|a| serde_json::json!({
                    "id": a.id,
                    "connector_type": a.connector_type,
                    "account_name": a.account_name,
                    "account_identifier": a.account_identifier,
                    "status": a.status,
                    "last_sync_at": a.last_sync_at,
                    "created_at": a.created_at,
                }))
                .collect();
            
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "accounts": accounts_json,
            })))
        }
        Err(e) => {
            error!("Failed to fetch connected accounts: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": e.to_string(),
            })))
        }
    }
}
