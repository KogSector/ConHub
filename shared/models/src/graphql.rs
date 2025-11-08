use serde::{Deserialize, Serialize};
use async_graphql::{InputObject, SimpleObject};

// Common GraphQL types for inter-service communication

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct ServiceHealthStatus {
    pub service_name: String,
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub database_connected: bool,
    pub redis_connected: bool,
    pub qdrant_connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct EmbeddingResult {
    pub embeddings: Vec<Vec<f32>>,
    pub dimension: usize,
    pub model: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, InputObject)]
pub struct EmbeddingRequest {
    pub texts: Vec<String>,
    pub normalize: Option<bool>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct RerankResult {
    pub id: String,
    pub score: f32,
    pub index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, InputObject)]
pub struct RerankRequest {
    pub query: String,
    pub documents: Vec<RerankDocument>,
    pub top_k: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, InputObject)]
pub struct RerankDocument {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct SearchResult {
    pub id: String,
    pub content: String,
    pub score: f32,
    pub source_type: String,
    pub metadata: String, // JSON string
}

#[derive(Debug, Clone, Serialize, Deserialize, InputObject)]
pub struct SearchRequest {
    pub query: String,
    pub source_types: Option<Vec<String>>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct IndexingStatus {
    pub job_id: String,
    pub status: String,
    pub progress_percentage: f32,
    pub items_processed: i64,
    pub items_total: i64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, InputObject)]
pub struct IndexingRequest {
    pub source_id: String,
    pub source_type: String,
    pub repository_url: Option<String>,
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct UserContext {
    pub user_id: String,
    pub email: String,
    pub roles: Vec<String>,
    pub organizations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct PluginStatus {
    pub plugin_id: String,
    pub plugin_name: String,
    pub plugin_type: String,
    pub status: String,
    pub enabled: bool,
    pub last_sync: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, InputObject)]
pub struct PluginAction {
    pub plugin_id: String,
    pub action: String,
    pub parameters: Option<String>, // JSON string
}

// Service-to-service communication types
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct ServiceResponse<T: async_graphql::OutputType> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

// Pagination support
#[derive(Debug, Clone, Serialize, Deserialize, InputObject)]
pub struct PaginationInput {
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct PaginatedResponse<T: async_graphql::OutputType> {
    pub items: Vec<T>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

impl<T: async_graphql::OutputType> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total_count: i64, page: i32, page_size: i32) -> Self {
        let total_pages = ((total_count as f64) / (page_size as f64)).ceil() as i32;
        Self {
            items,
            total_count,
            page,
            page_size,
            total_pages,
        }
    }
}

// Batch operations (non-generic GraphQL-friendly definitions)
#[derive(Debug, Clone, Serialize, Deserialize, InputObject)]
pub struct BatchRequest {
    pub items: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct BatchResult {
    pub results: Vec<serde_json::Value>,
    pub success_count: i32,
    pub failure_count: i32,
    pub errors: Vec<String>,
}
