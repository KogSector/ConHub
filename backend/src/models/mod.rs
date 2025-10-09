use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

pub mod copilot;
// pub mod mcp; // Disabled for now
pub mod auth;
pub mod billing;
pub mod social;

// VCS (Version Control System) Models
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum VcsType {
    Git,
    Subversion,
    Mercurial,
    Bazaar,
    Perforce,
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum VcsProvider {
    GitHub,
    GitLab,
    Bitbucket,
    Azure,
    Gitea,
    SourceForge,
    CodeCommit,
    SelfHosted,
    Local,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CredentialType {
    PersonalAccessToken { token: String },
    UsernamePassword { username: String, password: String },
    SshKey { private_key: String, passphrase: Option<String> },
    AppPassword { username: String, app_password: String },
    None,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RepositoryCredentials {
    pub credential_type: CredentialType,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RepositoryConfig {
    pub branch: String,
    pub auto_sync: bool,
    pub webhook_enabled: bool,
    pub webhook_secret: Option<String>,
    pub include_branches: Vec<String>,
    pub exclude_paths: Vec<String>,
    pub include_file_extensions: Vec<String>,
    pub max_file_size_mb: u32,
    pub sync_frequency_minutes: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RepositoryInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub url: String,
    pub vcs_type: VcsType,
    pub provider: VcsProvider,
    pub owner: String,
    pub is_private: bool,
    pub default_branch: String,
    pub clone_url: String,
    pub ssh_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_synced: Option<DateTime<Utc>>,
    pub sync_status: RepositorySyncStatus,
    pub credentials: RepositoryCredentials,
    pub config: RepositoryConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum RepositorySyncStatus {
    Connected,
    Syncing,
    SyncCompleted,
    SyncFailed,
    Disconnected,
    PendingAuth,
}

#[derive(Deserialize, Debug)]
pub struct ConnectRepositoryRequest {
    pub url: String,
    pub vcs_type: Option<VcsType>,
    pub provider: Option<VcsProvider>,
    pub credentials: RepositoryCredentials,
    pub config: Option<RepositoryConfig>,
}

// Legacy repository connection request (deprecated)
#[derive(Deserialize)]
pub struct ConnectRepoRequest {
    pub repo_url: String,
    pub access_token: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Serialize)]
pub struct ServiceStatus {
    pub name: String,
    pub url: String,
    pub status: String,
    pub response_time_ms: Option<u64>,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            message: "Success".to_string(),
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            message: message.clone(),
            data: None,
            error: Some(message),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserSettings {
    pub user_id: String,
    pub profile: ProfileSettings,
    pub notifications: NotificationSettings,
    pub security: SecuritySettings,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProfileSettings {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
    pub social_links: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NotificationSettings {
    pub email_notifications: bool,
    pub push_notifications: bool,
    pub security_alerts: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SecuritySettings {
    pub two_factor_enabled: bool,
    pub session_timeout: u32,
}

#[derive(Deserialize)]
pub struct UpdateSettingsRequest {
    pub profile: Option<ProfileSettings>,
    pub notifications: Option<NotificationSettings>,
    pub security: Option<SecuritySettings>,
}

// AI Agent Models
#[derive(Serialize, Deserialize, Clone)]
pub struct AgentRecord {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub agent_type: String, // "openai", "anthropic", "custom", etc.
    pub endpoint: Option<String>, // For custom agents
    pub api_key: String, // Encrypted in storage
    pub permissions: Vec<String>, // ["read", "write", "context", "repositories", "documents", "urls"]
    pub status: AgentStatus,
    pub config: AgentConfig,
    pub created_at: String,
    pub updated_at: String,
    pub last_used: Option<String>,
    pub usage_stats: AgentUsageStats,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum AgentStatus {
    Connected,
    Pending,
    Error,
    Inactive,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AgentConfig {
    pub model: Option<String>, // e.g., "gpt-4", "claude-3"
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub timeout: Option<u32>, // seconds
    pub custom_instructions: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AgentUsageStats {
    pub total_requests: u64,
    pub total_tokens: u64,
    pub avg_response_time: Option<f32>,
    pub last_error: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct CreateAgentRequest {
    pub name: String,
    pub agent_type: String,
    pub endpoint: Option<String>,
    pub api_key: String,
    pub permissions: Vec<String>,
    pub config: AgentConfig,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct UpdateAgentRequest {
    pub name: Option<String>,
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub config: Option<AgentConfig>,
    pub status: Option<AgentStatus>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct AgentInvokeRequest {
    pub message: String,
    pub context_type: Option<String>, // "repositories", "documents", "urls", "all"
    pub include_history: Option<bool>,
}

#[derive(Serialize)]
pub struct AgentInvokeResponse {
    pub response: String,
    pub usage: AgentInvokeUsage,
    pub context_used: Vec<String>,
}

#[derive(Serialize)]
pub struct AgentInvokeUsage {
    pub tokens_used: u32,
    pub response_time_ms: u64,
}

#[derive(Serialize, Debug, Clone)]
pub struct AgentContext {
    pub repositories: Vec<RepositoryContext>,
    pub documents: Vec<DocumentContext>,
    pub urls: Vec<UrlContext>,
}

#[derive(Serialize, Debug, Clone)]
pub struct RepositoryContext {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub language: String,
    pub recent_files: Vec<String>,
    pub recent_commits: Vec<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct DocumentContext {
    pub id: String,
    pub name: String,
    pub doc_type: String,
    pub summary: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct UrlContext {
    pub id: String,
    pub url: String,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub tags: Vec<String>,
}

// Data Source Types
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum DataSourceType {
    Repository,
    Document,
    Url,
    Dropbox,
    GoogleDrive,
    OneDrive,
    LocalFile,
    Notion,
}

// Model Context Protocol module is declared at the top of the file