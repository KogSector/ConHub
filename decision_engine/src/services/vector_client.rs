use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{QueryFilters, ContextBlock, SourceInfo, Provenance, VectorProvenance};

/// Client for vector_rag service
#[derive(Clone)]
pub struct VectorRagClient {
    base_url: String,
    client: reqwest::Client,
}

impl VectorRagClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    /// Search vector index
    pub async fn search(
        &self,
        query: &str,
        filters: &QueryFilters,
        top_k: usize,
    ) -> Result<Vec<ContextBlock>> {
        let request = VectorSearchRequest {
            profile: "default".to_string(),
            query: query.to_string(),
            filters: VectorFilters {
                source_types: filters.source_types.clone(),
                repos: filters.repos.clone(),
                tags: filters.tags.clone(),
            },
            top_k,
        };

        let url = format!("{}/search", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to call vector_rag service")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Vector search failed with status {}: {}", status, error_text);
        }

        let search_response: VectorSearchResponse = response
            .json()
            .await
            .context("Failed to parse vector search response")?;

        // Convert to ContextBlocks
        Ok(search_response
            .results
            .into_iter()
            .map(|r| ContextBlock {
                chunk_id: r.chunk_id,
                document_id: r.document_id,
                source: SourceInfo {
                    source_type: r.metadata.get("source_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    external_id: r.metadata.get("external_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    metadata: r.metadata.clone(),
                },
                content: r.content,
                score: r.score,
                provenance: Provenance {
                    vector: Some(VectorProvenance {
                        similarity: r.score,
                        embedding_model: "default".to_string(),
                    }),
                    graph: None,
                },
            })
            .collect())
    }
}

#[derive(Debug, Serialize)]
struct VectorSearchRequest {
    profile: String,
    query: String,
    filters: VectorFilters,
    top_k: usize,
}

#[derive(Debug, Serialize)]
struct VectorFilters {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    source_types: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    repos: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct VectorSearchResponse {
    results: Vec<VectorResult>,
}

#[derive(Debug, Deserialize)]
struct VectorResult {
    chunk_id: Uuid,
    document_id: Uuid,
    score: f32,
    content: String,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}
