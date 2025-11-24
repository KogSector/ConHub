use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{QueryFilters, ContextBlock, SourceInfo, Provenance, GraphProvenance};

/// Client for graph_rag service
#[derive(Clone)]
pub struct GraphRagClient {
    base_url: String,
    client: reqwest::Client,
}

impl GraphRagClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    /// Search graph index
    pub async fn search(
        &self,
        query: &str,
        filters: &QueryFilters,
        top_k: usize,
    ) -> Result<Vec<ContextBlock>> {
        let request = GraphSearchRequest {
            query: query.to_string(),
            filters: GraphFilters {
                source_types: filters.source_types.clone(),
                repos: filters.repos.clone(),
            },
            mode: "keyword".to_string(),
            top_k,
        };

        let url = format!("{}/search", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to call graph_rag service")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Graph search failed with status {}: {}", status, error_text);
        }

        let search_response: GraphSearchResponse = response
            .json()
            .await
            .context("Failed to parse graph search response")?;

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
                    vector: None,
                    graph: Some(GraphProvenance {
                        path: r.path,
                        distance: r.distance,
                        relationship_types: r.relationship_types,
                    }),
                },
            })
            .collect())
    }

    /// Expand from seed nodes
    pub async fn expand(
        &self,
        seed_node_ids: Vec<Uuid>,
        max_hops: usize,
        edge_types: Vec<String>,
        max_nodes: usize,
    ) -> Result<Vec<ContextBlock>> {
        let request = GraphExpandRequest {
            seed_nodes: seed_node_ids,
            max_hops,
            edge_types,
            max_nodes,
        };

        let url = format!("{}/expand", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to call graph_rag expand")?;

        if !response.status().is_success() {
            return Ok(Vec::new()); // Graceful degradation
        }

        let expand_response: GraphExpandResponse = response
            .json()
            .await
            .context("Failed to parse graph expand response")?;

        // Convert nodes to ContextBlocks
        Ok(expand_response
            .nodes
            .into_iter()
            .filter_map(|n| {
                // Only include nodes that are chunks
                if n.node_type == "chunk" {
                    Some(ContextBlock {
                        chunk_id: n.id,
                        document_id: n.metadata.get("document_id")
                            .and_then(|v| v.as_str())
                            .and_then(|s| Uuid::parse_str(s).ok())
                            .unwrap_or_else(Uuid::nil),
                        source: SourceInfo {
                            source_type: n.metadata.get("source_type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            external_id: n.metadata.get("external_id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            metadata: n.metadata.clone(),
                        },
                        content: n.content.unwrap_or_default(),
                        score: 1.0 - (n.distance as f32 / 10.0), // Convert distance to score
                        provenance: Provenance {
                            vector: None,
                            graph: Some(GraphProvenance {
                                path: vec![],
                                distance: n.distance,
                                relationship_types: vec![],
                            }),
                        },
                    })
                } else {
                    None
                }
            })
            .collect())
    }
}

#[derive(Debug, Serialize)]
struct GraphSearchRequest {
    query: String,
    filters: GraphFilters,
    mode: String,
    top_k: usize,
}

#[derive(Debug, Serialize)]
struct GraphFilters {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    source_types: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    repos: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GraphSearchResponse {
    results: Vec<GraphResult>,
}

#[derive(Debug, Deserialize)]
struct GraphResult {
    chunk_id: Uuid,
    document_id: Uuid,
    score: f32,
    content: String,
    path: Vec<String>,
    distance: usize,
    relationship_types: Vec<String>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct GraphExpandRequest {
    seed_nodes: Vec<Uuid>,
    max_hops: usize,
    edge_types: Vec<String>,
    max_nodes: usize,
}

#[derive(Debug, Deserialize)]
struct GraphExpandResponse {
    nodes: Vec<GraphNode>,
}

#[derive(Debug, Deserialize)]
struct GraphNode {
    id: Uuid,
    node_type: String,
    content: Option<String>,
    distance: usize,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}
