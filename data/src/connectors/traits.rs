use async_trait::async_trait;
use uuid::Uuid;
use super::types::*;
use super::error::ConnectorError;

/// Main trait that all connectors must implement
#[async_trait]
pub trait Connector: Send + Sync {
    /// Returns the name of the connector
    fn name(&self) -> &str;
    
    /// Returns the connector type
    fn connector_type(&self) -> ConnectorType;
    
    /// Validates the configuration before connection
    fn validate_config(&self, config: &ConnectorConfigAuth) -> Result<(), ConnectorError>;
    
    /// Initiates the authentication flow
    /// Returns an authorization URL if OAuth is required, or None if credentials are sufficient
    async fn authenticate(&self, config: &ConnectorConfigAuth) -> Result<Option<String>, ConnectorError>;
    
    /// Completes the OAuth flow with the callback data
    async fn complete_oauth(&self, callback_data: OAuthCallbackData) -> Result<OAuthCredentials, ConnectorError>;
    
    /// Establishes connection with the external service
    async fn connect(&mut self, account: &ConnectedAccount) -> Result<(), ConnectorError>;
    
    /// Checks if the connection is still valid
    async fn check_connection(&self, account: &ConnectedAccount) -> Result<bool, ConnectorError>;
    
    /// Lists all available files/documents from the source
    async fn list_documents(
        &self,
        account: &ConnectedAccount,
        filters: Option<SyncFilters>,
    ) -> Result<Vec<DocumentMetadata>, ConnectorError>;
    
    /// Fetches the content of a specific document
    async fn get_document_content(
        &self,
        account: &ConnectedAccount,
        document_id: &str,
    ) -> Result<DocumentContent, ConnectorError>;
    
    /// Syncs documents from the external source
    /// Returns the list of documents that need to be embedded
    async fn sync(
        &self,
        account: &ConnectedAccount,
        request: &SyncRequestWithFilters,
    ) -> Result<(SyncResult, Vec<DocumentForEmbedding>), ConnectorError>;
    
    /// Handles incremental sync (only changed files since last sync)
    async fn incremental_sync(
        &self,
        account: &ConnectedAccount,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<DocumentMetadata>, ConnectorError>;
    
    /// Disconnects and cleans up resources
    async fn disconnect(&mut self, account: &ConnectedAccount) -> Result<(), ConnectorError>;
    
    /// Refreshes OAuth tokens if needed
    async fn refresh_credentials(&self, account: &ConnectedAccount) -> Result<OAuthCredentials, ConnectorError>;
}

/// Trait for connectors that support webhooks
#[async_trait]
pub trait WebhookConnector: Connector {
    /// Registers a webhook with the external service
    async fn register_webhook(
        &self,
        account: &ConnectedAccount,
        webhook_url: &str,
    ) -> Result<String, ConnectorError>; // Returns webhook ID
    
    /// Handles incoming webhook notifications
    async fn handle_webhook(
        &self,
        account: &ConnectedAccount,
        webhook_id: &str,
        payload: serde_json::Value,
    ) -> Result<Vec<DocumentMetadata>, ConnectorError>;
    
    /// Unregisters a webhook
    async fn unregister_webhook(
        &self,
        account: &ConnectedAccount,
        webhook_id: &str,
    ) -> Result<(), ConnectorError>;
}

/// Factory trait for creating connector instances
pub trait ConnectorFactory: Send + Sync {
    fn create(&self) -> Box<dyn Connector>;
    fn connector_type(&self) -> ConnectorType;
    fn supports_oauth(&self) -> bool;
    fn supports_webhooks(&self) -> bool;
}
