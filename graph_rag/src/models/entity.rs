use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use std::collections::HashMap;

/// Universal entity types that span across all data sources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Person,
    CodeEntity,
    Document,
    Conversation,
    Project,
    Concept,
    Timestamp,
    Organization,
    Repository,
    File,
    Function,
    Class,
    Module,
    Commit,
    PullRequest,
    Issue,
    Message,
    Thread,
    Channel,
    NotionPage,
    SlackMessage,
    Email,
    Bug,
    Feature,
}

impl EntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::Person => "person",
            EntityType::CodeEntity => "code_entity",
            EntityType::Document => "document",
            EntityType::Conversation => "conversation",
            EntityType::Project => "project",
            EntityType::Concept => "concept",
            EntityType::Timestamp => "timestamp",
            EntityType::Organization => "organization",
            EntityType::Repository => "repository",
            EntityType::File => "file",
            EntityType::Function => "function",
            EntityType::Class => "class",
            EntityType::Module => "module",
            EntityType::Commit => "commit",
            EntityType::PullRequest => "pull_request",
            EntityType::Issue => "issue",
            EntityType::Message => "message",
            EntityType::Thread => "thread",
            EntityType::Channel => "channel",
            EntityType::NotionPage => "notion_page",
            EntityType::SlackMessage => "slack_message",
            EntityType::Email => "email",
            EntityType::Bug => "bug",
            EntityType::Feature => "feature",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "person" => Some(EntityType::Person),
            "code_entity" => Some(EntityType::CodeEntity),
            "document" => Some(EntityType::Document),
            "conversation" => Some(EntityType::Conversation),
            "project" => Some(EntityType::Project),
            "concept" => Some(EntityType::Concept),
            "timestamp" => Some(EntityType::Timestamp),
            "organization" => Some(EntityType::Organization),
            "repository" => Some(EntityType::Repository),
            "file" => Some(EntityType::File),
            "function" => Some(EntityType::Function),
            "class" => Some(EntityType::Class),
            "module" => Some(EntityType::Module),
            "commit" => Some(EntityType::Commit),
            "pull_request" => Some(EntityType::PullRequest),
            "issue" => Some(EntityType::Issue),
            "message" => Some(EntityType::Message),
            "thread" => Some(EntityType::Thread),
            "channel" => Some(EntityType::Channel),
            "notion_page" => Some(EntityType::NotionPage),
            "slack_message" => Some(EntityType::SlackMessage),
            "email" => Some(EntityType::Email),
            "bug" => Some(EntityType::Bug),
            "feature" => Some(EntityType::Feature),
            _ => None,
        }
    }
}

/// Data source from which entity was extracted
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DataSource {
    GitHub,
    Slack,
    Notion,
    GoogleDrive,
    Dropbox,
    LocalFile,
    Bitbucket,
    UrlCrawler,
    Email,
    Jira,
    Confluence,
}

impl DataSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            DataSource::GitHub => "github",
            DataSource::Slack => "slack",
            DataSource::Notion => "notion",
            DataSource::GoogleDrive => "google_drive",
            DataSource::Dropbox => "dropbox",
            DataSource::LocalFile => "local_file",
            DataSource::Bitbucket => "bitbucket",
            DataSource::UrlCrawler => "url_crawler",
            DataSource::Email => "email",
            DataSource::Jira => "jira",
            DataSource::Confluence => "confluence",
        }
    }
}

/// Core entity in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Entity {
    pub id: Uuid,
    pub entity_type: String,
    pub source: String,
    pub source_id: String,
    pub name: String,
    pub canonical_id: Option<Uuid>,
    pub properties: serde_json::Value,
    pub embedding: Option<Vec<f32>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity {
    pub fn new(
        entity_type: EntityType,
        source: DataSource,
        source_id: String,
        name: String,
        properties: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_type: entity_type.as_str().to_string(),
            source: source.as_str().to_string(),
            source_id,
            name,
            canonical_id: None,
            properties: serde_json::to_value(properties).unwrap_or(serde_json::json!({})),
            embedding: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

/// Canonical entity representing merged view across sources
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CanonicalEntity {
    pub id: Uuid,
    pub entity_type: String,
    pub canonical_name: String,
    pub merged_properties: serde_json::Value,
    pub source_entities: serde_json::Value,
    pub confidence_score: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CanonicalEntity {
    pub fn new(
        entity_type: EntityType,
        canonical_name: String,
        source_entities: Vec<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_type: entity_type.as_str().to_string(),
            canonical_name,
            merged_properties: serde_json::json!({}),
            source_entities: serde_json::to_value(source_entities).unwrap_or(serde_json::json!([])),
            confidence_score: 1.0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

/// Request to create a new entity
#[derive(Debug, Deserialize)]
pub struct CreateEntityRequest {
    pub entity_type: String,
    pub source: String,
    pub source_id: String,
    pub name: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub text_for_embedding: Option<String>,
}

/// Response after creating entity
#[derive(Debug, Serialize)]
pub struct CreateEntityResponse {
    pub entity_id: Uuid,
    pub canonical_id: Option<Uuid>,
    pub resolved: bool,
}
