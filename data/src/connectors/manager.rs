use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use sqlx::PgPool;
use uuid::Uuid;
use tracing::{info, error, warn};

use super::traits::{Connector, ConnectorFactory};
use super::types::*;
use super::error::ConnectorError;
use super::local_file::LocalFileConnector;
use super::github::GitHubConnector;
use super::google_drive::GoogleDriveConnector;

/// Manages all connectors and their instances
pub struct ConnectorManager {
    factories: HashMap<ConnectorType, Arc<dyn ConnectorFactory>>,
    active_connectors: Arc<RwLock<HashMap<Uuid, Arc<RwLock<Box<dyn Connector>>>>>>,
    db_pool: Option<PgPool>,
}

impl ConnectorManager {
    /// Create a new ConnectorManager
    pub fn new(db_pool: Option<PgPool>) -> Self {
        let mut manager = Self {
            factories: HashMap::new(),
            active_connectors: Arc::new(RwLock::new(HashMap::new())),
            db_pool,
        };
        
        // Register default connectors
        manager.register_default_connectors();
        
        manager
    }
    
    /// Register all default connector factories
    fn register_default_connectors(&mut self) {
        info!("📦 Registering default connectors...");
        
        // Local file connector
        self.factories.insert(
            ConnectorType::LocalFile,
            Arc::new(LocalFileConnector::factory()),
        );
        
        // GitHub connector
        self.factories.insert(
            ConnectorType::GitHub,
            Arc::new(GitHubConnector::factory()),
        );
        
        // Google Drive connector
        self.factories.insert(
            ConnectorType::GoogleDrive,
            Arc::new(GoogleDriveConnector::factory()),
        );
        
        info!("✅ Registered {} connectors", self.factories.len());
    }
    
    /// Get a list of all available connector types
    pub fn available_connectors(&self) -> Vec<ConnectorType> {
        self.factories.keys().cloned().collect()
    }
    
    /// Create a new connector instance
    pub fn create_connector(&self, connector_type: &ConnectorType) -> Result<Box<dyn Connector>, ConnectorError> {
        self.factories
            .get(connector_type)
            .ok_or_else(|| ConnectorError::UnsupportedOperation(
                format!("Connector type {:?} is not registered", connector_type)
            ))
            .map(|factory| factory.create())
    }
    
    /// Connect a new data source
    pub async fn connect(
        &self,
        user_id: Uuid,
        request: ConnectRequest,
    ) -> Result<ConnectedAccount, ConnectorError> {
        info!("🔌 Connecting new data source: {:?}", request.connector_type);
        
        let config = ConnectorConfig {
            user_id,
            connector_type: request.connector_type.clone(),
            credentials: request.credentials.clone(),
            settings: request.settings.unwrap_or_default(),
        };
        
        // Create connector instance
        let mut connector = self.create_connector(&request.connector_type)?;
        
        // Validate configuration
        connector.validate_config(&config)?;
        
        // Authenticate
        let auth_result = connector.authenticate(&config).await?;
        
        if let Some(auth_url) = auth_result {
            // OAuth flow required - return pending auth status
            return Ok(ConnectedAccount {
                id: Uuid::new_v4(),
                user_id,
                connector_type: request.connector_type,
                account_name: request.account_name.unwrap_or_else(|| "Pending".to_string()),
                account_identifier: String::new(),
                credentials: serde_json::json!({ "auth_url": auth_url }),
                status: ConnectionStatus::PendingAuth,
                last_sync_at: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                metadata: None,
            });
        }
        
        // Create connected account record
        let account = ConnectedAccount {
            id: Uuid::new_v4(),
            user_id,
            connector_type: request.connector_type.clone(),
            account_name: request.account_name.unwrap_or_else(|| {
                format!("{} Account", request.connector_type.as_str())
            }),
            account_identifier: config.credentials.get("identifier")
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            credentials: serde_json::to_value(&config.credentials)?,
            status: ConnectionStatus::Connected,
            last_sync_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: Some(serde_json::to_value(&config.settings)?),
        };
        
        // Connect
        connector.connect(&account).await?;
        
        // Save to database if available
        if let Some(ref pool) = self.db_pool {
            self.save_account(pool, &account).await?;
        }
        
        // Store active connector
        let mut connectors = self.active_connectors.write().await;
        connectors.insert(account.id, Arc::new(RwLock::new(connector)));
        
        info!("✅ Connected data source: {}", account.id);
        
        Ok(account)
    }
    
    /// Complete OAuth authentication
    pub async fn complete_oauth(
        &self,
        account_id: Uuid,
        callback_data: OAuthCallbackData,
    ) -> Result<ConnectedAccount, ConnectorError> {
        info!("🔐 Completing OAuth for account: {}", account_id);
        
        // Get account from database
        let pool = self.db_pool.as_ref()
            .ok_or_else(|| ConnectorError::DatabaseError("Database not available".to_string()))?;
        
        let mut account = self.get_account(pool, account_id).await?;
        
        // Get connector
        let connector = self.create_connector(&account.connector_type)?;
        
        // Complete OAuth
        let credentials = connector.complete_oauth(callback_data).await?;
        
        // Update account
        account.credentials = serde_json::to_value(&credentials)?;
        account.status = ConnectionStatus::Connected;
        account.updated_at = chrono::Utc::now();
        
        // Save to database
        self.update_account(pool, &account).await?;
        
        info!("✅ OAuth completed for account: {}", account_id);
        
        Ok(account)
    }
    
    /// Sync a connected data source
    pub async fn sync(
        &self,
        account_id: Uuid,
        request: SyncRequest,
    ) -> Result<(SyncResult, Vec<DocumentForEmbedding>), ConnectorError> {
        info!("🔄 Syncing data source: {}", account_id);
        
        // Get account
        let pool = self.db_pool.as_ref()
            .ok_or_else(|| ConnectorError::DatabaseError("Database not available".to_string()))?;
        
        let account = self.get_account(pool, account_id).await?;
        
        // Get or create connector
        let connector = self.get_or_create_connector(account_id, &account).await?;
        
        // Sync
        let connector_lock = connector.read().await;
        let result = connector_lock.sync(&account, &request).await?;
        
        // Update last sync time
        self.update_last_sync(pool, account_id).await?;
        
        info!("✅ Sync completed for account: {}", account_id);
        
        Ok(result)
    }
    
    /// Disconnect a data source
    pub async fn disconnect(&self, account_id: Uuid) -> Result<(), ConnectorError> {
        info!("🔌 Disconnecting data source: {}", account_id);
        
        // Get account
        let pool = self.db_pool.as_ref()
            .ok_or_else(|| ConnectorError::DatabaseError("Database not available".to_string()))?;
        
        let account = self.get_account(pool, account_id).await?;
        
        // Get connector if active
        let mut connectors = self.active_connectors.write().await;
        if let Some(connector) = connectors.remove(&account_id) {
            let mut connector_lock = connector.write().await;
            connector_lock.disconnect(&account).await?;
        }
        
        // Update database
        self.delete_account(pool, account_id).await?;
        
        info!("✅ Disconnected data source: {}", account_id);
        
        Ok(())
    }
    
    /// Get or create a connector for an account
    async fn get_or_create_connector(
        &self,
        account_id: Uuid,
        account: &ConnectedAccount,
    ) -> Result<Arc<RwLock<Box<dyn Connector>>>, ConnectorError> {
        let connectors = self.active_connectors.read().await;
        
        if let Some(connector) = connectors.get(&account_id) {
            return Ok(Arc::clone(connector));
        }
        
        drop(connectors);
        
        // Create new connector
        let mut connector = self.create_connector(&account.connector_type)?;
        connector.connect(account).await?;
        
        let connector_arc = Arc::new(RwLock::new(connector));
        
        let mut connectors = self.active_connectors.write().await;
        connectors.insert(account_id, Arc::clone(&connector_arc));
        
        Ok(connector_arc)
    }
    
    // Database operations
    
    async fn save_account(&self, pool: &PgPool, account: &ConnectedAccount) -> Result<(), ConnectorError> {
        sqlx::query!(
            r#"
            INSERT INTO connected_accounts (
                id, user_id, connector_type, account_name, account_identifier,
                credentials, status, last_sync_at, metadata, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            account.id,
            account.user_id,
            account.connector_type.as_str(),
            account.account_name,
            account.account_identifier,
            account.credentials,
            serde_json::to_value(&account.status).unwrap_or(serde_json::json!("connected")),
            account.last_sync_at,
            account.metadata,
            account.created_at,
            account.updated_at,
        )
        .execute(pool)
        .await?;
        
        Ok(())
    }
    
    async fn get_account(&self, pool: &PgPool, account_id: Uuid) -> Result<ConnectedAccount, ConnectorError> {
        let row = sqlx::query!(
            r#"
            SELECT id, user_id, connector_type, account_name, account_identifier,
                   credentials, status, last_sync_at, metadata, created_at, updated_at
            FROM connected_accounts
            WHERE id = $1
            "#,
            account_id
        )
        .fetch_one(pool)
        .await?;
        
        Ok(ConnectedAccount {
            id: row.id,
            user_id: row.user_id,
            connector_type: serde_json::from_str(&format!("\"{}\"", row.connector_type))
                .unwrap_or(ConnectorType::LocalFile),
            account_name: row.account_name,
            account_identifier: row.account_identifier,
            credentials: row.credentials,
            status: serde_json::from_value(row.status).unwrap_or(ConnectionStatus::Disconnected),
            last_sync_at: row.last_sync_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
            metadata: row.metadata,
        })
    }
    
    async fn update_account(&self, pool: &PgPool, account: &ConnectedAccount) -> Result<(), ConnectorError> {
        sqlx::query!(
            r#"
            UPDATE connected_accounts
            SET credentials = $1, status = $2, updated_at = $3
            WHERE id = $4
            "#,
            account.credentials,
            serde_json::to_value(&account.status).unwrap_or(serde_json::json!("connected")),
            account.updated_at,
            account.id,
        )
        .execute(pool)
        .await?;
        
        Ok(())
    }
    
    async fn update_last_sync(&self, pool: &PgPool, account_id: Uuid) -> Result<(), ConnectorError> {
        sqlx::query!(
            r#"
            UPDATE connected_accounts
            SET last_sync_at = $1, updated_at = $2
            WHERE id = $3
            "#,
            chrono::Utc::now(),
            chrono::Utc::now(),
            account_id,
        )
        .execute(pool)
        .await?;
        
        Ok(())
    }
    
    async fn delete_account(&self, pool: &PgPool, account_id: Uuid) -> Result<(), ConnectorError> {
        sqlx::query!(
            r#"
            DELETE FROM connected_accounts
            WHERE id = $1
            "#,
            account_id
        )
        .execute(pool)
        .await?;
        
        Ok(())
    }
}
