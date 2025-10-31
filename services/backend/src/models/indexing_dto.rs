use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct IndexRepositoryRequest {
    #[validate(url)]
    pub repository_url: String,
    pub branch: Option<String>,
    pub include_patterns: Option<Vec<String>>,
    pub exclude_patterns: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct IndexDocumentationRequest {
    #[validate(url)]
    pub url: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct IndexUrlRequest {
    #[validate(url)]
    pub url: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SearchRequest {
    #[validate(length(min = 1))]
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct IndexingResponse {
    pub job_id: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub content: String,
    pub score: f32,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct IndexingStatusResponse {
    pub job_id: String,
    pub status: String,
    pub progress: f32,
    pub message: String,
}
