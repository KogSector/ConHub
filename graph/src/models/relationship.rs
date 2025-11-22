use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Universal relationship types that connect entities across sources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RelationshipType {
    AuthoredBy,
    DiscussedIn,
    DocumentedIn,
    References,
    BelongsTo,
    DependsOn,
    Implements,
    RelatedTo,
    MentionedIn,
    Resolves,
    ResolvesTo,
    Contains,
    Imports,
    Calls,
    Modifies,
    ContributedTo,
    HasSection,
    RepliedTo,
    Mentions,
    Discusses,
    ParticipatedIn,
    LinksTo,
    ChildOf,
    SemanticallyRelated,
    Next,
}

impl RelationshipType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationshipType::AuthoredBy => "AUTHORED_BY",
            RelationshipType::DiscussedIn => "DISCUSSED_IN",
            RelationshipType::DocumentedIn => "DOCUMENTED_IN",
            RelationshipType::References => "REFERENCES",
            RelationshipType::BelongsTo => "BELONGS_TO",
            RelationshipType::DependsOn => "DEPENDS_ON",
            RelationshipType::Implements => "IMPLEMENTS",
            RelationshipType::RelatedTo => "RELATED_TO",
            RelationshipType::MentionedIn => "MENTIONED_IN",
            RelationshipType::Resolves => "RESOLVES",
            RelationshipType::ResolvesTo => "RESOLVES_TO",
            RelationshipType::Contains => "CONTAINS",
            RelationshipType::Imports => "IMPORTS",
            RelationshipType::Calls => "CALLS",
            RelationshipType::Modifies => "MODIFIES",
            RelationshipType::ContributedTo => "CONTRIBUTED_TO",
            RelationshipType::HasSection => "HAS_SECTION",
            RelationshipType::RepliedTo => "REPLIED_TO",
            RelationshipType::Mentions => "MENTIONS",
            RelationshipType::Discusses => "DISCUSSES",
            RelationshipType::ParticipatedIn => "PARTICIPATED_IN",
            RelationshipType::LinksTo => "LINKS_TO",
            RelationshipType::ChildOf => "CHILD_OF",
            RelationshipType::SemanticallyRelated => "SEMANTICALLY_RELATED",
            RelationshipType::Next => "NEXT",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "AUTHORED_BY" => Some(RelationshipType::AuthoredBy),
            "DISCUSSED_IN" => Some(RelationshipType::DiscussedIn),
            "DOCUMENTED_IN" => Some(RelationshipType::DocumentedIn),
            "REFERENCES" => Some(RelationshipType::References),
            "BELONGS_TO" => Some(RelationshipType::BelongsTo),
            "DEPENDS_ON" => Some(RelationshipType::DependsOn),
            "IMPLEMENTS" => Some(RelationshipType::Implements),
            "RELATED_TO" => Some(RelationshipType::RelatedTo),
            "MENTIONED_IN" => Some(RelationshipType::MentionedIn),
            "RESOLVES" => Some(RelationshipType::Resolves),
            "RESOLVES_TO" => Some(RelationshipType::ResolvesTo),
            "CONTAINS" => Some(RelationshipType::Contains),
            "IMPORTS" => Some(RelationshipType::Imports),
            "CALLS" => Some(RelationshipType::Calls),
            "MODIFIES" => Some(RelationshipType::Modifies),
            "CONTRIBUTED_TO" => Some(RelationshipType::ContributedTo),
            "HAS_SECTION" => Some(RelationshipType::HasSection),
            "REPLIED_TO" => Some(RelationshipType::RepliedTo),
            "MENTIONS" => Some(RelationshipType::Mentions),
            "DISCUSSES" => Some(RelationshipType::Discusses),
            "PARTICIPATED_IN" => Some(RelationshipType::ParticipatedIn),
            "LINKS_TO" => Some(RelationshipType::LinksTo),
            "CHILD_OF" => Some(RelationshipType::ChildOf),
            "SEMANTICALLY_RELATED" => Some(RelationshipType::SemanticallyRelated),
            "NEXT" => Some(RelationshipType::Next),
            _ => None,
        }
    }
}

/// Relationship between two entities in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Relationship {
    pub id: Uuid,
    pub from_entity_id: Uuid,
    pub to_entity_id: Uuid,
    pub relationship_type: String,
    pub properties: serde_json::Value,
    pub weight: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Relationship {
    pub fn new(
        from_entity_id: Uuid,
        to_entity_id: Uuid,
        relationship_type: RelationshipType,
        properties: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            from_entity_id,
            to_entity_id,
            relationship_type: relationship_type.as_str().to_string(),
            properties,
            weight: 1.0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

/// Request to create a new relationship
#[derive(Debug, Deserialize)]
pub struct CreateRelationshipRequest {
    pub from_entity_id: Uuid,
    pub to_entity_id: Uuid,
    pub relationship_type: String,
    pub properties: Option<serde_json::Value>,
    pub weight: Option<f32>,
}

/// Response after creating relationship
#[derive(Debug, Serialize)]
pub struct CreateRelationshipResponse {
    pub relationship_id: Uuid,
    pub created: bool,
}

/// Query for finding related entities
#[derive(Debug, Deserialize)]
pub struct FindRelatedRequest {
    pub entity_id: Uuid,
    pub relationship_types: Option<Vec<String>>,
    pub max_depth: Option<u32>,
    pub limit: Option<u32>,
}

/// Response with related entities
#[derive(Debug, Serialize)]
pub struct RelatedEntity {
    pub entity_id: Uuid,
    pub entity_type: String,
    pub name: String,
    pub relationship_path: Vec<String>,
    pub depth: u32,
    pub relevance_score: f32,
}
