use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

/// Entity resolution candidate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionCandidate {
    pub entity_id: Uuid,
    pub entity_type: String,
    pub source: String,
    pub name: String,
    pub features: EntityFeatures,
    pub confidence_score: f32,
}

/// Features extracted from entity for resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityFeatures {
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub username: Option<String>,
    pub user_id: Option<String>,
    pub display_name: Option<String>,
    pub profile_url: Option<String>,
    pub associated_repositories: Vec<String>,
    pub associated_channels: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Resolution result after matching entities
#[derive(Debug, Serialize)]
pub struct ResolutionResult {
    pub canonical_id: Uuid,
    pub resolved_entities: Vec<Uuid>,
    pub confidence_score: f32,
    pub matching_strategy: String,
}

/// Request to resolve entities
#[derive(Debug, Deserialize)]
pub struct ResolveEntitiesRequest {
    pub entity_ids: Vec<Uuid>,
    pub min_confidence: Option<f32>,
}

/// Matching strategy for entity resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchingStrategy {
    ExactEmailMatch,
    FuzzyNameMatch,
    AttributeOverlap,
    GraphBased,
    Composite,
}

impl MatchingStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            MatchingStrategy::ExactEmailMatch => "exact_email_match",
            MatchingStrategy::FuzzyNameMatch => "fuzzy_name_match",
            MatchingStrategy::AttributeOverlap => "attribute_overlap",
            MatchingStrategy::GraphBased => "graph_based",
            MatchingStrategy::Composite => "composite",
        }
    }
}

/// Resolution match between two entities
#[derive(Debug, Clone, Serialize)]
pub struct ResolutionMatch {
    pub entity1_id: Uuid,
    pub entity2_id: Uuid,
    pub confidence_score: f32,
    pub matching_features: Vec<String>,
    pub strategy: MatchingStrategy,
}

/// Entity resolution configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ResolutionConfig {
    pub min_confidence_threshold: f32,
    pub email_match_weight: f32,
    pub name_similarity_weight: f32,
    pub attribute_overlap_weight: f32,
    pub graph_similarity_weight: f32,
    pub fuzzy_match_threshold: f32,
}

impl Default for ResolutionConfig {
    fn default() -> Self {
        Self {
            min_confidence_threshold: 0.7,
            email_match_weight: 0.9,
            name_similarity_weight: 0.5,
            attribute_overlap_weight: 0.3,
            graph_similarity_weight: 0.3,
            fuzzy_match_threshold: 0.85,
        }
    }
}
