use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Client for decision_engine service
#[derive(Clone)]
pub struct DecisionEngineClient {
    base_url: String,
    client: reqwest::Client,
}

impl DecisionEngineClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    /// Query context with automatic strategy selection
    pub async fn query_context(
        &self,
        request: ContextQueryRequest,
    ) -> Result<ContextQueryResponse> {
        let url = format!("{}/context/query", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to call decision_engine")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Context query failed with status {}: {}", status, error_text);
        }

        let query_response: ContextQueryResponse = response
            .json()
            .await
            .context("Failed to parse context query response")?;

        Ok(query_response)
    }

    /// Get decision engine statistics
    pub async fn get_stats(&self) -> Result<StatsResponse> {
        let url = format!("{}/context/stats", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to get stats from decision_engine")?;

        if !response.status().is_success() {
            anyhow::bail!("Stats request failed");
        }

        let stats: StatsResponse = response
            .json()
            .await
            .context("Failed to parse stats response")?;

        Ok(stats)
    }
}

// Request/Response types matching decision_engine

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextQueryRequest {
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub query: String,
    
    #[serde(default)]
    pub filters: QueryFilters,
    
    #[serde(default)]
    pub strategy: Strategy,
    
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

fn default_top_k() -> usize {
    20
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryFilters {
    #[serde(default)]
    pub source_types: Vec<String>,
    
    #[serde(default)]
    pub repos: Vec<String>,
    
    #[serde(default)]
    pub channels: Vec<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_range: Option<TimeRange>,
    
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub from: chrono::DateTime<chrono::Utc>,
    pub to: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Strategy {
    Vector,
    Graph,
    Hybrid,
    Auto,
}

impl Default for Strategy {
    fn default() -> Self {
        Strategy::Auto
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextQueryResponse {
    pub strategy_used: Strategy,
    pub blocks: Vec<ContextBlock>,
    pub total_results: usize,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBlock {
    pub chunk_id: Uuid,
    pub document_id: Uuid,
    pub source: SourceInfo,
    pub content: String,
    pub score: f32,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub source_type: String,
    pub external_id: String,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector: Option<VectorProvenance>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph: Option<GraphProvenance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorProvenance {
    pub similarity: f32,
    pub embedding_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphProvenance {
    pub path: Vec<String>,
    pub distance: usize,
    pub relationship_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResponse {
    pub cache_enabled: bool,
    pub cache_hit_rate: f32,
    pub total_queries: usize,
    pub avg_latency_ms: f32,
    pub strategy_distribution: HashMap<String, usize>,
}
