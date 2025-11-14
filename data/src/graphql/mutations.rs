use async_graphql::*;
use sqlx::PgPool;
use uuid::Uuid;
use std::sync::Arc;
use chrono::Utc;

use super::types::*;
use crate::connectors::ConnectorManager;
use crate::connectors::types::{
    ConnectorType,
    ConnectorConfigAuth,
    OAuthCallbackData,
    SyncRequestWithFilters,
    ConnectedAccount as ConnectedAccountInternal,
    ConnectionStatus as ConnectionStatusInternal,
};

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Connect a new data source
    async fn connect_source(
        &self,
        ctx: &Context<'_>,
        input: ConnectSourceInput,
    ) -> Result<ConnectResult> {
        let pool = ctx.data::<Option<PgPool>>()
            .map_err(|_| Error::new("Database pool not found"))?;
        
        let connector_manager = ctx.data::<Arc<ConnectorManager>>()
            .map_err(|_| Error::new("Connector manager not found"))?;
        
        if let Some(pool) = pool {
            let user_id = ctx.data::<Uuid>()
                .map_err(|_| Error::new("User ID not found in context"))?;
            
            // Convert GraphQL connector type to internal type
            let connector_type = match input.connector_type {
                ConnectorTypeGql::Github => ConnectorType::GitHub,
                ConnectorTypeGql::GoogleDrive => ConnectorType::GoogleDrive,
                ConnectorTypeGql::Dropbox => ConnectorType::Dropbox,
                ConnectorTypeGql::Notion => ConnectorType::Notion,
                ConnectorTypeGql::LocalFile => ConnectorType::LocalFile,
                ConnectorTypeGql::WebUrl => ConnectorType::UrlScraper,
            };
            
            // Create connector auth config
            let config = ConnectorConfigAuth {
                connector_type: connector_type.clone(),
                credentials: input.credentials
                    .and_then(|j| serde_json::from_value(j.0).ok())
                    .unwrap_or_default(),
                settings: input.settings
                    .and_then(|j| serde_json::from_value(j.0).ok())
                    .unwrap_or_default(),
            };
            
            // Get connector and initiate authentication
            let connector = connector_manager.get_connector(&connector_type)
                .map_err(|e| Error::new(format!("Failed to get connector: {}", e)))?;
            
            match connector.authenticate(&config).await {
                Ok(Some(auth_url)) => {
                    // OAuth flow required - return auth URL
                    Ok(ConnectResult {
                        success: true,
                        message: "OAuth authorization required".to_string(),
                        account: None,
                        authorization_url: Some(auth_url),
                    })
                }
                Ok(None) => {
                    // Direct connection (no OAuth)
                    // Create connected account record
                    let account_id = Uuid::new_v4();
                    let connector_type_str = format!("{:?}", connector_type).to_lowercase();
                    
                    sqlx::query(
                        r#"
                        INSERT INTO connected_accounts 
                        (id, user_id, connector_type, account_name, account_identifier, credentials, status)
                        VALUES ($1, $2, $3, $4, $5, $6, $7)
                        "#
                    )
                    .bind(account_id)
                    .bind(user_id)
                    .bind(connector_type_str)
                    .bind(input.account_name.clone())
                    .bind(input.account_name.clone())
                    .bind(serde_json::json!(config.credentials))
                    .bind(serde_json::json!({"status": "connected"}))
                    .execute(pool)
                    .await
                    .map_err(|e| Error::new(format!("Failed to create connected account: {}", e)))?;
                    
                    // Fetch the created account
                    let account = sqlx::query_as::<_, ConnectedAccountDb>(
                        r#"
                        SELECT 
                            ca.id,
                            ca.user_id,
                            ca.connector_type,
                            ca.account_name,
                            ca.account_identifier,
                            ca.status,
                            ca.last_sync_at,
                            ca.created_at,
                            ca.updated_at,
                            0::int AS document_count
                        FROM connected_accounts ca
                        WHERE ca.id = $1
                        "#
                    )
                    .bind(account_id)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| Error::new(format!("Failed to fetch account: {}", e)))?;
                    
                    Ok(ConnectResult {
                        success: true,
                        message: "Source connected successfully".to_string(),
                        account: Some(account.into()),
                        authorization_url: None,
                    })
                }
                Err(e) => {
                    Ok(ConnectResult {
                        success: false,
                        message: format!("Failed to connect: {}", e),
                        account: None,
                        authorization_url: None,
                    })
                }
            }
        } else {
            Err(Error::new("Database not available"))
        }
    }
    
    /// Complete OAuth callback
    async fn complete_oauth_callback(
        &self,
        ctx: &Context<'_>,
        input: OAuthCallbackInput,
    ) -> Result<ConnectResult> {
        let pool = ctx.data::<Option<PgPool>>()
            .map_err(|_| Error::new("Database pool not found"))?;
        
        let connector_manager = ctx.data::<Arc<ConnectorManager>>()
            .map_err(|_| Error::new("Connector manager not found"))?;
        
        if let Some(pool) = pool {
            let user_id = ctx.data::<Uuid>()
                .map_err(|_| Error::new("User ID not found in context"))?;
            
            let connector_type = match input.connector_type {
                ConnectorTypeGql::Github => ConnectorType::GitHub,
                ConnectorTypeGql::GoogleDrive => ConnectorType::GoogleDrive,
                ConnectorTypeGql::Dropbox => ConnectorType::Dropbox,
                ConnectorTypeGql::Notion => ConnectorType::Notion,
                _ => return Err(Error::new("Invalid connector type for OAuth")),
            };
            
            let connector = connector_manager.get_connector(&connector_type)
                .map_err(|e| Error::new(format!("Failed to get connector: {}", e)))?;
            
            // Complete OAuth flow
            let callback_data = OAuthCallbackData {
                code: input.code,
                state: input.state,
            };
            
            match connector.complete_oauth(callback_data).await {
                Ok(credentials) => {
                    // Create connected account
                    let account_id = Uuid::new_v4();
                    let connector_type_str = format!("{:?}", connector_type).to_lowercase();
                    let account_name = credentials.metadata.get("account_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Connected Account")
                        .to_string();
                    let account_identifier = credentials.metadata.get("account_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&account_id.to_string())
                        .to_string();
                    
                    sqlx::query(
                        r#"
                        INSERT INTO connected_accounts 
                        (id, user_id, connector_type, account_name, account_identifier, credentials, status)
                        VALUES ($1, $2, $3, $4, $5, $6, $7)
                        "#
                    )
                    .bind(account_id)
                    .bind(user_id)
                    .bind(connector_type_str)
                    .bind(account_name)
                    .bind(account_identifier)
                    .bind(serde_json::json!({
                        "access_token": credentials.access_token,
                        "refresh_token": credentials.refresh_token,
                        "expires_at": credentials.expires_at,
                        "metadata": credentials.metadata
                    }))
                    .bind(serde_json::json!({"status": "connected"}))
                    .execute(pool)
                    .await
                    .map_err(|e| Error::new(format!("Failed to create connected account: {}", e)))?;
                    
                    // Fetch the created account
                    let account = sqlx::query_as::<_, ConnectedAccountDb>(
                        r#"
                        SELECT 
                            ca.id,
                            ca.user_id,
                            ca.connector_type,
                            ca.account_name,
                            ca.account_identifier,
                            ca.status,
                            ca.last_sync_at,
                            ca.created_at,
                            ca.updated_at,
                            0::int AS document_count
                        FROM connected_accounts ca
                        WHERE ca.id = $1
                        "#
                    )
                    .bind(account_id)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| Error::new(format!("Failed to fetch account: {}", e)))?;
                    
                    // Trigger initial sync in background
                    // TODO: Send to background worker/queue
                    
                    Ok(ConnectResult {
                        success: true,
                        message: "OAuth completed successfully. Syncing data...".to_string(),
                        account: Some(account.into()),
                        authorization_url: None,
                    })
                }
                Err(e) => {
                    Ok(ConnectResult {
                        success: false,
                        message: format!("OAuth failed: {}", e),
                        account: None,
                        authorization_url: None,
                    })
                }
            }
        } else {
            Err(Error::new("Database not available"))
        }
    }
    
    /// Sync a connected source
    async fn sync_source(
        &self,
        ctx: &Context<'_>,
        input: SyncSourceInput,
    ) -> Result<SyncResult> {
        let pool = ctx.data::<Option<PgPool>>()
            .map_err(|_| Error::new("Database pool not found"))?;
        
        let connector_manager = ctx.data::<Arc<ConnectorManager>>()
            .map_err(|_| Error::new("Connector manager not found"))?;
        
        if let Some(pool) = pool {
            let user_id = ctx.data::<Uuid>()
                .map_err(|_| Error::new("User ID not found in context"))?;
            let account_id = Uuid::parse_str(&input.account_id)
                .map_err(|_| Error::new("Invalid account ID"))?;
            
            // Fetch connected account
            let account_db = sqlx::query_as::<_, ConnectedAccountDbMut>(
                r#"
                SELECT 
                    id, user_id, connector_type, account_name, account_identifier, 
                    credentials, status, last_sync_at, metadata, created_at, updated_at
                FROM connected_accounts
                WHERE id = $1 AND user_id = $2
                "#
            )
            .bind(account_id)
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(|e| Error::new(format!("Failed to fetch account: {}", e)))?;
            
            // Update status to syncing
            sqlx::query(
                "UPDATE connected_accounts SET status = $1 WHERE id = $2"
            )
            .bind(serde_json::json!({"status": "syncing"}))
            .bind(account_id)
            .execute(pool)
            .await
            .map_err(|e| Error::new(format!("Failed to update status: {}", e)))?;
            
            // Convert to internal types
            let connector_type = parse_connector_type(&account_db.connector_type);
            let connector = connector_manager.get_connector(&connector_type)
                .map_err(|e| Error::new(format!("Failed to get connector: {}", e)))?;
            
            // Build connected account object
            // Clone to avoid moving, we still need fields later
            let connected_account = build_connected_account_from_db(account_db.clone());
            let connector_type_for_docs = account_db.connector_type.clone();
            
            // Perform sync
            let sync_request = SyncRequestWithFilters {
                force_full_sync: input.force_full_sync.unwrap_or(false),
                filters: input.filters
                    .and_then(|j| serde_json::from_value(j.0).ok()),
            };
            
            match connector.sync(&connected_account, &sync_request).await {
                Ok((sync_result, documents_for_embedding)) => {
                    // Store documents in database
                    let mut documents_synced = 0;
                    let mut documents_queued = 0;
                    
                    for doc in documents_for_embedding {
                        // Insert or update source_documents
                        // Extract optional metadata fields
                        let mime_type: Option<String> = doc
                            .metadata
                            .get("mime_type")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let size: Option<i64> = doc
                            .metadata
                            .get("size")
                            .and_then(|v| v.as_i64());
                        let url: Option<String> = doc
                            .metadata
                            .get("url")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        // Convert enum content type to string
                        let content_type_str: Option<String> = Some(match doc.content_type {
                            crate::connectors::types::ContentType::Text => "text".to_string(),
                            crate::connectors::types::ContentType::Binary => "binary".to_string(),
                            crate::connectors::types::ContentType::Code => "code".to_string(),
                            crate::connectors::types::ContentType::Markdown => "markdown".to_string(),
                            crate::connectors::types::ContentType::Html => "html".to_string(),
                            crate::connectors::types::ContentType::Pdf => "pdf".to_string(),
                            crate::connectors::types::ContentType::Image => "image".to_string(),
                            crate::connectors::types::ContentType::Video => "video".to_string(),
                            crate::connectors::types::ContentType::Audio => "audio".to_string(),
                            crate::connectors::types::ContentType::Archive => "archive".to_string(),
                            crate::connectors::types::ContentType::Unknown => "unknown".to_string(),
                        });

                        match sqlx::query_scalar::<_, Uuid>(
                            r#"
                            INSERT INTO source_documents 
                            (id, source_id, connector_type, external_id, name, path, content_type, 
                             mime_type, size, url, is_folder, metadata)
                            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                            ON CONFLICT (source_id, external_id) 
                            DO UPDATE SET 
                                name = EXCLUDED.name,
                                updated_at = CURRENT_TIMESTAMP
                            RETURNING id
                            "#
                        )
                        .bind(doc.id)
                        .bind(account_id)
                        .bind(connector_type_for_docs.clone())
                        .bind(doc.external_id)
                        .bind(doc.name)
                        .bind(doc.path)
                        .bind(content_type_str)
                        .bind(mime_type)
                        .bind(size)
                        .bind(url)
                        .bind(false)
                        .bind(doc.metadata)
                        .fetch_one(pool)
                        .await {
                            Ok(inserted_id) => {
                                documents_synced += 1;
                                
                                // Queue for embedding
                                sqlx::query(
                                    r#"
                                    INSERT INTO embedding_queue (document_id, status)
                                    VALUES ($1, 'pending')
                                    ON CONFLICT (document_id) DO NOTHING
                                    "#
                                )
                                .bind(inserted_id)
                                .execute(pool)
                                .await
                                .ok();
                                
                                documents_queued += 1;
                            }
                            Err(e) => {
                                tracing::error!("Failed to insert document: {}", e);
                            }
                        }
                    }
                    
                    // Update last_sync_at
                    sqlx::query(
                        "UPDATE connected_accounts SET last_sync_at = $1, status = $2 WHERE id = $3"
                    )
                    .bind(Utc::now())
                    .bind(serde_json::json!({"status": "connected"}))
                    .bind(account_id)
                    .execute(pool)
                    .await
                    .ok();
                    
                    Ok(SyncResult {
                        success: true,
                        message: format!("Synced {} documents", documents_synced),
                        documents_synced,
                        documents_queued_for_embedding: documents_queued,
                    })
                }
                Err(e) => {
                    // Update status to error
                    sqlx::query!(
                        "UPDATE connected_accounts SET status = $1 WHERE id = $2",
                        serde_json::json!({"status": "error", "error": e.to_string()}),
                        account_id
                    )
                    .execute(pool)
                    .await
                    .ok();
                    
                    Ok(SyncResult {
                        success: false,
                        message: format!("Sync failed: {}", e),
                        documents_synced: 0,
                        documents_queued_for_embedding: 0,
                    })
                }
            }
        } else {
            Err(Error::new("Database not available"))
        }
    }
    
    /// Disconnect a source
    async fn disconnect_source(
        &self,
        ctx: &Context<'_>,
        account_id: ID,
    ) -> Result<DisconnectResult> {
        let pool = ctx.data::<Option<PgPool>>()
            .map_err(|_| Error::new("Database pool not found"))?;
        
        if let Some(pool) = pool {
            let user_id = ctx.data::<Uuid>()
                .map_err(|_| Error::new("User ID not found in context"))?;
            let id = Uuid::parse_str(&account_id)
                .map_err(|_| Error::new("Invalid account ID"))?;
            
            // Delete connected account (cascade will delete related documents and queue entries)
            let result = sqlx::query!(
                "DELETE FROM connected_accounts WHERE id = $1 AND user_id = $2",
                id,
                user_id
            )
            .execute(pool)
            .await
            .map_err(|e| Error::new(format!("Failed to disconnect source: {}", e)))?;
            
            if result.rows_affected() > 0 {
                Ok(DisconnectResult {
                    success: true,
                    message: "Source disconnected successfully".to_string(),
                })
            } else {
                Ok(DisconnectResult {
                    success: false,
                    message: "Source not found or already disconnected".to_string(),
                })
            }
        } else {
            Err(Error::new("Database not available"))
        }
    }
    
    /// Upload a local file
    async fn upload_file(
        &self,
        ctx: &Context<'_>,
        input: UploadFileInput,
    ) -> Result<SourceDocument> {
        let pool = ctx.data::<Option<PgPool>>()
            .map_err(|_| Error::new("Database pool not found"))?;
        
        if let Some(pool) = pool {
            let user_id = ctx.data::<Uuid>()
                .map_err(|_| Error::new("User ID not found in context"))?;
            
            // Find or create local_file connected account
            let account_id = sqlx::query_scalar::<_, Uuid>(
                r#"
                INSERT INTO connected_accounts 
                (user_id, connector_type, account_name, account_identifier, status)
                VALUES ($1, 'local_file', 'Local Files', 'local', $2)
                ON CONFLICT (user_id, connector_type, account_identifier) 
                DO UPDATE SET updated_at = CURRENT_TIMESTAMP
                RETURNING id
                "#
            )
            .bind(user_id)
            .bind(serde_json::json!({"status": "connected"}))
            .fetch_one(pool)
            .await
            .map_err(|e| Error::new(format!("Failed to get local account: {}", e)))?;
            
            // Decode base64 content
            let content_bytes = base64::decode(&input.content)
                .map_err(|_| Error::new("Invalid base64 content"))?;
            
            // TODO: Store file content in object storage or file system
            // For now, we'll just store metadata
            
            let document_id = Uuid::new_v4();
            let external_id = document_id.to_string();
            
            sqlx::query(
                r#"
                INSERT INTO source_documents 
                (id, source_id, connector_type, external_id, name, content_type, 
                 mime_type, size, is_folder, metadata)
                VALUES ($1, $2, 'local_file', $3, $4, $5, $6, $7, false, $8)
                "#
            )
            .bind(document_id)
            .bind(account_id)
            .bind(&external_id)
            .bind(input.name.clone())
            .bind(Some(input.content_type.clone()))
            .bind(Some(input.content_type.clone()))
            .bind(input.size)
            .bind(serde_json::json!({
                "tags": input.tags.unwrap_or_default(),
                "uploaded_by": user_id
            }))
            .execute(pool)
            .await
            .map_err(|e| Error::new(format!("Failed to create document: {}", e)))?;
            
            // Queue for embedding
            sqlx::query(
                "INSERT INTO embedding_queue (document_id, status) VALUES ($1, 'pending')"
            )
            .bind(document_id)
            .execute(pool)
            .await
            .ok();
            
            // Return created document
            Ok(SourceDocument {
                id: ID(document_id.to_string()),
                source_id: ID(account_id.to_string()),
                connector_type: ConnectorTypeGql::LocalFile,
                external_id,
                name: input.name,
                path: None,
                content_type: Some(input.content_type.clone()),
                mime_type: None,
                size: Some(input.size),
                url: None,
                is_folder: false,
                status: DocumentStatus::Pending,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                indexed_at: None,
            })
        } else {
            Err(Error::new("Database not available"))
        }
    }
}

// Helper functions
fn parse_connector_type(type_str: &str) -> ConnectorType {
    match type_str {
        "github" => ConnectorType::GitHub,
        "google_drive" => ConnectorType::GoogleDrive,
        "dropbox" => ConnectorType::Dropbox,
        "notion" => ConnectorType::Notion,
        "local_file" => ConnectorType::LocalFile,
        "web_url" | "url_scraper" => ConnectorType::UrlScraper,
        _ => ConnectorType::LocalFile,
    }
}

fn build_connected_account_from_db(db: ConnectedAccountDbMut) -> ConnectedAccountInternal {
    let status_json = db.status.0;
    let status = if let Some(s) = status_json.get("status").and_then(|v| v.as_str()) {
        match s {
            "connected" => ConnectionStatusInternal::Connected,
            "disconnected" => ConnectionStatusInternal::Disconnected,
            "syncing" => ConnectionStatusInternal::Syncing,
            "pending_auth" => ConnectionStatusInternal::PendingAuth,
            "error" => {
                let msg = status_json
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                ConnectionStatusInternal::Error(msg)
            }
            _ => ConnectionStatusInternal::Disconnected,
        }
    } else {
        ConnectionStatusInternal::Disconnected
    };

    ConnectedAccountInternal {
        id: db.id,
        user_id: db.user_id,
        connector_type: parse_connector_type(&db.connector_type),
        account_name: db.account_name,
        account_identifier: db.account_identifier,
        credentials: db.credentials.0,
        status,
        last_sync_at: db.last_sync_at,
        metadata: db.metadata.map(|m| m.0),
        created_at: db.created_at,
        updated_at: db.updated_at,
    }
}

// Database types

#[derive(Debug, sqlx::FromRow)]
struct ConnectedAccountDb {
    id: Uuid,
    user_id: Uuid,
    connector_type: String,
    account_name: String,
    account_identifier: String,
    status: sqlx::types::Json<serde_json::Value>,
    last_sync_at: Option<chrono::DateTime<chrono::Utc>>, 
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    document_count: Option<i32>,
}

#[derive(Debug, sqlx::FromRow, Clone)]
struct ConnectedAccountDbMut {
    id: Uuid,
    user_id: Uuid,
    connector_type: String,
    account_name: String,
    account_identifier: String,
    credentials: sqlx::types::Json<serde_json::Value>,
    status: sqlx::types::Json<serde_json::Value>,
    last_sync_at: Option<chrono::DateTime<chrono::Utc>>,
    metadata: Option<sqlx::types::Json<serde_json::Value>>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<ConnectedAccountDb> for ConnectedAccount {
    fn from(db: ConnectedAccountDb) -> Self {
        let status_str = db.status.get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("connected");

        let status = match status_str {
            "connecting" => ConnectionStatus::Connecting,
            "syncing" => ConnectionStatus::Syncing,
            "error" => ConnectionStatus::Error,
            "disconnected" => ConnectionStatus::Disconnected,
            _ => ConnectionStatus::Connected,
        };

        let connector_type = match db.connector_type.as_str() {
            "github" => ConnectorTypeGql::Github,
            "google_drive" => ConnectorTypeGql::GoogleDrive,
            "dropbox" => ConnectorTypeGql::Dropbox,
            "notion" => ConnectorTypeGql::Notion,
            "local_file" => ConnectorTypeGql::LocalFile,
            "web_url" => ConnectorTypeGql::WebUrl,
            _ => ConnectorTypeGql::LocalFile,
        };

        ConnectedAccount {
            id: ID(db.id.to_string()),
            user_id: ID(db.user_id.to_string()),
            connector_type,
            account_name: db.account_name,
            account_identifier: db.account_identifier,
            status,
            last_sync_at: db.last_sync_at,
            created_at: db.created_at,
            updated_at: db.updated_at,
            document_count: db.document_count.unwrap_or(0),
        }
    }
}
