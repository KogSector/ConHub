use async_graphql::*;
use sqlx::PgPool;
use uuid::Uuid;
use std::sync::Arc;

use super::types::*;
use crate::connectors::ConnectorManager;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get all available connector types
    async fn available_connectors(&self, ctx: &Context<'_>) -> Result<Vec<ConnectorInfo>> {
        Ok(vec![
            ConnectorInfo {
                connector_type: ConnectorTypeGql::Github,
                name: "GitHub".to_string(),
                description: "Connect to GitHub repositories for code context".to_string(),
                requires_oauth: true,
                supports_webhooks: true,
                supported_file_types: vec!["*".to_string()],
            },
            ConnectorInfo {
                connector_type: ConnectorTypeGql::GoogleDrive,
                name: "Google Drive".to_string(),
                description: "Connect to Google Drive for document context".to_string(),
                requires_oauth: true,
                supports_webhooks: true,
                supported_file_types: vec!["*".to_string()],
            },
            ConnectorInfo {
                connector_type: ConnectorTypeGql::LocalFile,
                name: "Local Files".to_string(),
                description: "Upload files from your local system".to_string(),
                requires_oauth: false,
                supports_webhooks: false,
                supported_file_types: vec!["*".to_string()],
            },
        ])
    }
    
    /// Get all connected accounts for the current user
    async fn connected_accounts(&self, ctx: &Context<'_>) -> Result<Vec<ConnectedAccount>> {
        let pool = ctx.data::<Option<PgPool>>()
            .map_err(|_| Error::new("Database pool not found"))?;
        
        if let Some(pool) = pool {
            let user_id = ctx.data::<Uuid>()
                .map_err(|_| Error::new("User ID not found in context"))?;
            
            let accounts = sqlx::query_as::<_, ConnectedAccountDb>(
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
                    COALESCE(COUNT(sd.id), 0)::int AS document_count
                FROM connected_accounts ca
                LEFT JOIN source_documents sd ON sd.source_id = ca.id
                WHERE ca.user_id = $1
                GROUP BY ca.id
                ORDER BY ca.created_at DESC
                "#
            )
            .bind(user_id)
            .fetch_all(pool)
            .await
            .map_err(|e| Error::new(format!("Failed to fetch connected accounts: {}", e)))?;
            
            Ok(accounts.into_iter().map(|a| a.into()).collect())
        } else {
            Err(Error::new("Database not available"))
        }
    }
    
    /// Get a specific connected account by ID
    async fn connected_account(&self, ctx: &Context<'_>, id: ID) -> Result<Option<ConnectedAccount>> {
        let pool = ctx.data::<Option<PgPool>>()
            .map_err(|_| Error::new("Database pool not found"))?;
        
        if let Some(pool) = pool {
            let user_id = ctx.data::<Uuid>()
                .map_err(|_| Error::new("User ID not found in context"))?;
            let account_id = Uuid::parse_str(&id)
                .map_err(|_| Error::new("Invalid account ID"))?;
            
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
                    COALESCE(COUNT(sd.id), 0)::int AS document_count
                FROM connected_accounts ca
                LEFT JOIN source_documents sd ON sd.source_id = ca.id
                WHERE ca.id = $1 AND ca.user_id = $2
                GROUP BY ca.id
                "#
            )
            .bind(account_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| Error::new(format!("Failed to fetch connected account: {}", e)))?;
            
            Ok(account.map(|a| a.into()))
        } else {
            Err(Error::new("Database not available"))
        }
    }
    
    /// Get documents for a specific source
    async fn source_documents(
        &self,
        ctx: &Context<'_>,
        source_id: ID,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<SourceDocument>> {
        let pool = ctx.data::<Option<PgPool>>()
            .map_err(|_| Error::new("Database pool not found"))?;
        
        if let Some(pool) = pool {
            let user_id = ctx.data::<Uuid>()
                .map_err(|_| Error::new("User ID not found in context"))?;
            let src_id = Uuid::parse_str(&source_id)
                .map_err(|_| Error::new("Invalid source ID"))?;
            
            let limit = limit.unwrap_or(50).min(100);
            let offset = offset.unwrap_or(0);
            
            // Verify user owns this source
            let has_access: bool = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM connected_accounts WHERE id = $1 AND user_id = $2)"
            )
            .bind(src_id)
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(|e| Error::new(format!("Failed to verify access: {}", e)))?;
            
            if !has_access {
                return Err(Error::new("Access denied"));
            }
            
            let documents = sqlx::query_as::<_, SourceDocumentDb>(
                r#"
                SELECT 
                    id,
                    source_id,
                    connector_type,
                    external_id,
                    name,
                    path,
                    content_type,
                    mime_type,
                    size,
                    url,
                    is_folder,
                    created_at,
                    updated_at,
                    indexed_at
                FROM source_documents
                WHERE source_id = $1 AND is_folder = false
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#
            )
            .bind(src_id)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await
            .map_err(|e| Error::new(format!("Failed to fetch documents: {}", e)))?;
            
            Ok(documents.into_iter().map(|d| d.into()).collect())
        } else {
            Err(Error::new("Database not available"))
        }
    }
    
    /// Get embedding queue status
    async fn embedding_queue_status(&self, ctx: &Context<'_>) -> Result<Vec<EmbeddingQueueEntry>> {
        let pool = ctx.data::<Option<PgPool>>()
            .map_err(|_| Error::new("Database pool not found"))?;
        
        if let Some(pool) = pool {
            let user_id = ctx.data::<Uuid>()
                .map_err(|_| Error::new("User ID not found in context"))?;
            
            let entries = sqlx::query_as::<_, EmbeddingQueueDb>(
                r#"
                SELECT 
                    eq.id,
                    eq.document_id,
                    eq.status,
                    eq.retry_count,
                    eq.error_message,
                    eq.created_at,
                    eq.processed_at
                FROM embedding_queue eq
                INNER JOIN source_documents sd ON sd.id = eq.document_id
                INNER JOIN connected_accounts ca ON ca.id = sd.source_id
                WHERE ca.user_id = $1 AND eq.status != 'completed'
                ORDER BY eq.created_at DESC
                LIMIT 50
                "#
            )
            .bind(user_id)
            .fetch_all(pool)
            .await
            .map_err(|e| Error::new(format!("Failed to fetch embedding queue: {}", e)))?;
            
            Ok(entries.into_iter().map(|e| e.into()).collect())
        } else {
            Err(Error::new("Database not available"))
        }
    }
}

// Database models for query results
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
    document_count: i32,
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
        
        Self {
            id: ID(db.id.to_string()),
            user_id: ID(db.user_id.to_string()),
            connector_type,
            account_name: db.account_name,
            account_identifier: db.account_identifier,
            status,
            last_sync_at: db.last_sync_at,
            created_at: db.created_at,
            updated_at: db.updated_at,
            document_count: db.document_count,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct SourceDocumentDb {
    id: Uuid,
    source_id: Uuid,
    connector_type: String,
    external_id: String,
    name: String,
    path: Option<String>,
    content_type: Option<String>,
    mime_type: Option<String>,
    size: Option<i64>,
    url: Option<String>,
    is_folder: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    indexed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<SourceDocumentDb> for SourceDocument {
    fn from(db: SourceDocumentDb) -> Self {
        let connector_type = match db.connector_type.as_str() {
            "github" => ConnectorTypeGql::Github,
            "google_drive" => ConnectorTypeGql::GoogleDrive,
            "dropbox" => ConnectorTypeGql::Dropbox,
            "notion" => ConnectorTypeGql::Notion,
            "local_file" => ConnectorTypeGql::LocalFile,
            "web_url" => ConnectorTypeGql::WebUrl,
            _ => ConnectorTypeGql::LocalFile,
        };
        
        let status = if db.indexed_at.is_some() {
            DocumentStatus::Indexed
        } else {
            DocumentStatus::Pending
        };
        
        Self {
            id: ID(db.id.to_string()),
            source_id: ID(db.source_id.to_string()),
            connector_type,
            external_id: db.external_id,
            name: db.name,
            path: db.path,
            content_type: db.content_type,
            mime_type: db.mime_type,
            size: db.size,
            url: db.url,
            is_folder: db.is_folder,
            status,
            created_at: db.created_at,
            updated_at: db.updated_at,
            indexed_at: db.indexed_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct EmbeddingQueueDb {
    id: Uuid,
    document_id: Uuid,
    status: String,
    retry_count: i32,
    error_message: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    processed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<EmbeddingQueueDb> for EmbeddingQueueEntry {
    fn from(db: EmbeddingQueueDb) -> Self {
        let status = match db.status.as_str() {
            "processing" => DocumentStatus::Processing,
            "embedding" => DocumentStatus::Embedding,
            "indexed" => DocumentStatus::Indexed,
            "failed" => DocumentStatus::Failed,
            _ => DocumentStatus::Pending,
        };
        
        Self {
            id: ID(db.id.to_string()),
            document_id: ID(db.document_id.to_string()),
            status,
            retry_count: db.retry_count,
            error_message: db.error_message,
            created_at: db.created_at,
            processed_at: db.processed_at,
        }
    }
}
