use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Authentication method for MCP server connections
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpAuthMethod {
    /// No authentication required (for local filesystem server)
    None,

    /// OAuth2 authentication with access token
    OAuth2 {
        access_token: String,
        refresh_token: Option<String>,
    },

    /// API Key authentication
    ApiKey {
        key: String,
    },

    /// Bearer token authentication
    Bearer {
        token: String,
    },
}

/// Configuration for a single external MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Unique identifier for this MCP server
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Server endpoint URL
    pub endpoint: String,

    /// Authentication method
    pub auth: McpAuthMethod,

    /// Request timeout in seconds
    pub timeout_secs: u64,

    /// Maximum retry attempts on failure
    pub max_retries: u32,

    /// Initial retry delay in milliseconds
    pub initial_retry_delay_ms: u64,

    /// Maximum retry delay in milliseconds
    pub max_retry_delay_ms: u64,

    /// Whether this server is enabled
    pub enabled: bool,
}

impl McpServerConfig {
    /// Get timeout as Duration
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }

    /// Get initial retry delay as Duration
    pub fn initial_retry_delay(&self) -> Duration {
        Duration::from_millis(self.initial_retry_delay_ms)
    }

    /// Get max retry delay as Duration
    pub fn max_retry_delay(&self) -> Duration {
        Duration::from_millis(self.max_retry_delay_ms)
    }
}

/// Registry of all configured external MCP servers
#[derive(Debug, Clone)]
pub struct McpServersConfig {
    pub servers: Vec<McpServerConfig>,
}

impl McpServersConfig {
    /// Load MCP server configurations from environment variables
    pub fn from_env() -> Self {
        let mut servers = Vec::new();

        // Google Drive MCP Server
        let google_drive_endpoint = std::env::var("MCP_GOOGLE_DRIVE_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:3005".to_string());

        if let Ok(client_id) = std::env::var("GOOGLE_DRIVE_CLIENT_ID") {
            if let Ok(client_secret) = std::env::var("GOOGLE_DRIVE_CLIENT_SECRET") {
                servers.push(McpServerConfig {
                    id: "google-drive".to_string(),
                    name: "Google Drive MCP".to_string(),
                    endpoint: google_drive_endpoint,
                    auth: McpAuthMethod::OAuth2 {
                        access_token: format!("{}:{}", client_id, client_secret),
                        refresh_token: None,
                    },
                    timeout_secs: 30,
                    max_retries: 3,
                    initial_retry_delay_ms: 1000,
                    max_retry_delay_ms: 10000,
                    enabled: true,
                });
            }
        }

        // Dropbox MCP Server
        let dropbox_endpoint = std::env::var("MCP_DROPBOX_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:3006".to_string());

        if let Ok(access_token) = std::env::var("DROPBOX_ACCESS_TOKEN") {
            servers.push(McpServerConfig {
                id: "dropbox".to_string(),
                name: "Dropbox MCP".to_string(),
                endpoint: dropbox_endpoint,
                auth: McpAuthMethod::Bearer {
                    token: access_token,
                },
                timeout_secs: 30,
                max_retries: 3,
                initial_retry_delay_ms: 1000,
                max_retry_delay_ms: 10000,
                enabled: true,
            });
        }

        // Filesystem MCP Server
        let filesystem_endpoint = std::env::var("MCP_FILESYSTEM_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:3007".to_string());

        servers.push(McpServerConfig {
            id: "filesystem".to_string(),
            name: "Filesystem MCP".to_string(),
            endpoint: filesystem_endpoint,
            auth: McpAuthMethod::None,
            timeout_secs: 30,
            max_retries: 3,
            initial_retry_delay_ms: 1000,
            max_retry_delay_ms: 10000,
            enabled: true,
        });

        Self { servers }
    }

    /// Get server configuration by ID
    pub fn get_server(&self, id: &str) -> Option<&McpServerConfig> {
        self.servers.iter().find(|s| s.id == id)
    }

    /// Get all enabled servers
    pub fn enabled_servers(&self) -> Vec<&McpServerConfig> {
        self.servers.iter().filter(|s| s.enabled).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_timeouts() {
        let config = McpServerConfig {
            id: "test".to_string(),
            name: "Test Server".to_string(),
            endpoint: "http://localhost:3000".to_string(),
            auth: McpAuthMethod::None,
            timeout_secs: 10,
            max_retries: 3,
            initial_retry_delay_ms: 100,
            max_retry_delay_ms: 5000,
            enabled: true,
        };

        assert_eq!(config.timeout(), Duration::from_secs(10));
        assert_eq!(config.initial_retry_delay(), Duration::from_millis(100));
        assert_eq!(config.max_retry_delay(), Duration::from_millis(5000));
    }

    #[test]
    fn test_get_server_by_id() {
        let config = McpServersConfig {
            servers: vec![
                McpServerConfig {
                    id: "test1".to_string(),
                    name: "Test 1".to_string(),
                    endpoint: "http://localhost:3000".to_string(),
                    auth: McpAuthMethod::None,
                    timeout_secs: 10,
                    max_retries: 3,
                    initial_retry_delay_ms: 100,
                    max_retry_delay_ms: 5000,
                    enabled: true,
                },
            ],
        };

        assert!(config.get_server("test1").is_some());
        assert!(config.get_server("nonexistent").is_none());
    }
}
