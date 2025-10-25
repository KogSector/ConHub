use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::config::mcp_servers::{McpServersConfig, McpServerConfig, McpAuthMethod};
use crate::services::ai::mcp::{McpClient, McpClientConfig, AuthConfig};

/// MCP Registry manages connections to external MCP servers
/// and maintains them throughout the application lifecycle
pub struct McpRegistry {
    client: McpClient,
    config: McpServersConfig,
    connections: Arc<RwLock<HashMap<String, String>>>, // server_id -> connection_id mapping
}

#[derive(Debug)]
pub enum McpRegistryError {
    ConnectionFailed(String),
    ClientInitializationFailed(String),
    ServerNotFound(String),
}

impl std::fmt::Display for McpRegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpRegistryError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            McpRegistryError::ClientInitializationFailed(msg) => write!(f, "Client initialization failed: {}", msg),
            McpRegistryError::ServerNotFound(msg) => write!(f, "Server not found: {}", msg),
        }
    }
}

impl std::error::Error for McpRegistryError {}

impl McpRegistry {
    /// Create a new MCP registry with default configuration
    pub fn new() -> Result<Self, McpRegistryError> {
        let config = McpServersConfig::from_env();
        let client_config = McpClientConfig::default();

        let client = McpClient::with_config(client_config)
            .map_err(|e| McpRegistryError::ClientInitializationFailed(e.to_string()))?;

        Ok(Self {
            client,
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Initialize all configured MCP servers on application startup
    /// Connects to each enabled server with retry logic
    pub async fn initialize_all(&self) -> Result<(), McpRegistryError> {
        log::info!("Initializing MCP Registry...");

        let enabled_servers = self.config.enabled_servers();

        if enabled_servers.is_empty() {
            log::warn!("No MCP servers configured. External MCP integrations will not be available.");
            return Ok(());
        }

        log::info!("Found {} enabled MCP server(s)", enabled_servers.len());

        for server_config in enabled_servers {
            match self.connect_server_with_retry(server_config).await {
                Ok(connection_id) => {
                    log::info!(
                        "Successfully connected to MCP server '{}' (connection_id: {})",
                        server_config.name,
                        connection_id
                    );

                    let mut connections = self.connections.write().await;
                    connections.insert(server_config.id.clone(), connection_id);
                }
                Err(e) => {
                    log::error!(
                        "Failed to connect to MCP server '{}': {}",
                        server_config.name,
                        e
                    );
                    // Continue with other servers even if one fails
                }
            }
        }

        let connections = self.connections.read().await;
        log::info!(
            "MCP Registry initialized with {}/{} servers connected",
            connections.len(),
            enabled_servers.len()
        );

        Ok(())
    }

    /// Connect to a single MCP server with exponential backoff retry logic
    async fn connect_server_with_retry(
        &self,
        server_config: &McpServerConfig,
    ) -> Result<String, McpRegistryError> {
        let max_retries = server_config.max_retries;
        let mut current_delay = server_config.initial_retry_delay();
        let max_delay = server_config.max_retry_delay();

        for attempt in 1..=max_retries {
            log::debug!(
                "Attempting to connect to '{}' (attempt {}/{})",
                server_config.name,
                attempt,
                max_retries
            );

            match self.connect_server(server_config).await {
                Ok(connection_id) => {
                    return Ok(connection_id);
                }
                Err(e) => {
                    if attempt < max_retries {
                        log::warn!(
                            "Connection attempt {}/{} failed for '{}': {}. Retrying in {:?}",
                            attempt,
                            max_retries,
                            server_config.name,
                            e,
                            current_delay
                        );

                        tokio::time::sleep(current_delay).await;

                        // Exponential backoff: double the delay, capped at max_delay
                        current_delay = std::cmp::min(current_delay * 2, max_delay);
                    } else {
                        return Err(McpRegistryError::ConnectionFailed(format!(
                            "Failed after {} attempts: {}",
                            max_retries, e
                        )));
                    }
                }
            }
        }

        Err(McpRegistryError::ConnectionFailed(
            "Maximum retries exceeded".to_string(),
        ))
    }

    /// Connect to a single MCP server (single attempt)
    async fn connect_server(
        &self,
        server_config: &McpServerConfig,
    ) -> Result<String, McpRegistryError> {
        let auth_config = self.create_auth_config(&server_config.auth)?;

        let connection_id = self
            .client
            .connect(server_config.endpoint.clone(), auth_config)
            .await
            .map_err(|e| McpRegistryError::ConnectionFailed(e.to_string()))?;

        Ok(connection_id)
    }

    /// Convert McpAuthMethod to AuthConfig for the MCP client
    fn create_auth_config(
        &self,
        auth_method: &McpAuthMethod,
    ) -> Result<AuthConfig, McpRegistryError> {
        match auth_method {
            McpAuthMethod::None => Ok(AuthConfig {
                method: crate::models::mcp::AuthMethod::ApiKey,
                credentials: serde_json::json!({}),
            }),
            McpAuthMethod::OAuth2 {
                access_token,
                refresh_token,
            } => Ok(AuthConfig::oauth2(
                access_token.clone(),
                refresh_token.clone(),
            )),
            McpAuthMethod::ApiKey { key } => Ok(AuthConfig::api_key(key.clone())),
            McpAuthMethod::Bearer { token } => Ok(AuthConfig::bearer(token.clone())),
        }
    }

    /// Get connection ID for a specific MCP server
    pub async fn get_connection_id(&self, server_id: &str) -> Option<String> {
        let connections = self.connections.read().await;
        connections.get(server_id).cloned()
    }

    /// Get all active connections
    pub async fn list_connections(&self) -> HashMap<String, String> {
        let connections = self.connections.read().await;
        connections.clone()
    }

    /// Get the underlying MCP client for direct access
    pub fn client(&self) -> &McpClient {
        &self.client
    }

    /// Disconnect a specific MCP server
    pub async fn disconnect_server(&self, server_id: &str) -> Result<(), McpRegistryError> {
        let connection_id = {
            let connections = self.connections.read().await;
            connections
                .get(server_id)
                .cloned()
                .ok_or_else(|| McpRegistryError::ServerNotFound(server_id.to_string()))?
        };

        self.client
            .disconnect(&connection_id)
            .await
            .map_err(|e| McpRegistryError::ConnectionFailed(e.to_string()))?;

        let mut connections = self.connections.write().await;
        connections.remove(server_id);

        log::info!("Disconnected MCP server: {}", server_id);

        Ok(())
    }

    /// Reconnect a specific MCP server
    pub async fn reconnect_server(&self, server_id: &str) -> Result<String, McpRegistryError> {
        // First, disconnect if already connected
        if let Ok(()) = self.disconnect_server(server_id).await {
            log::info!("Disconnected existing connection for server: {}", server_id);
        }

        // Find server config
        let server_config = self
            .config
            .get_server(server_id)
            .ok_or_else(|| McpRegistryError::ServerNotFound(server_id.to_string()))?;

        // Reconnect with retry
        let connection_id = self.connect_server_with_retry(server_config).await?;

        // Update connections map
        let mut connections = self.connections.write().await;
        connections.insert(server_id.to_string(), connection_id.clone());

        Ok(connection_id)
    }

    /// Start health monitoring for all connected servers
    pub async fn start_health_monitoring(&self) {
        log::info!("Starting MCP health monitoring...");
        self.client.start_health_monitoring().await;
    }
}

impl Default for McpRegistry {
    fn default() -> Self {
        Self::new().expect("Failed to create MCP registry")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_auth_config() {
        let registry = McpRegistry::new().unwrap();

        // Test None auth
        let auth = McpAuthMethod::None;
        let config = registry.create_auth_config(&auth).unwrap();
        assert_eq!(config.method, crate::models::mcp::AuthMethod::ApiKey);

        // Test API Key auth
        let auth = McpAuthMethod::ApiKey {
            key: "test-key".to_string(),
        };
        let config = registry.create_auth_config(&auth).unwrap();
        assert_eq!(config.method, crate::models::mcp::AuthMethod::ApiKey);
        assert_eq!(
            config.credentials["api_key"],
            serde_json::json!("test-key")
        );

        // Test Bearer auth
        let auth = McpAuthMethod::Bearer {
            token: "test-token".to_string(),
        };
        let config = registry.create_auth_config(&auth).unwrap();
        assert_eq!(config.method, crate::models::mcp::AuthMethod::Bearer);
    }

    #[tokio::test]
    async fn test_connection_tracking() {
        let registry = McpRegistry::new().unwrap();

        // Initially empty
        let connections = registry.list_connections().await;
        assert_eq!(connections.len(), 0);

        // Simulate adding a connection
        {
            let mut conns = registry.connections.write().await;
            conns.insert("test-server".to_string(), "conn-123".to_string());
        }

        // Verify connection was added
        let connection_id = registry.get_connection_id("test-server").await;
        assert_eq!(connection_id, Some("conn-123".to_string()));

        // Verify list shows the connection
        let connections = registry.list_connections().await;
        assert_eq!(connections.len(), 1);
        assert_eq!(
            connections.get("test-server"),
            Some(&"conn-123".to_string())
        );
    }
}
