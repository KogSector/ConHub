use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

use super::qdrant::{QdrantService, SearchResult as QdrantSearchResult};

const RRF_K: usize = 60; // Standard RRF constant

/// Final fusion result with score and metadata
#[derive(Debug, Clone, Serialize)]
pub struct FusionResult {
    pub id: String,
    pub score: f32,
    pub source: String,
    pub content: String,
    pub metadata: Value,
}

/// Intermediate result during fusion processing
#[derive(Debug, Clone)]
pub struct IntermediateResult {
    pub id: String,
    pub rank: usize,
    pub score: f32,
    pub source: String,
    pub content: String,
    pub metadata: Value,
}

/// Request to embedding service for embeddings
#[derive(Debug, Serialize)]
struct EmbedRequest {
    text: Vec<String>,
    normalize: bool,
}

/// Response from embedding service
#[derive(Debug, Deserialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f32>>,
    dimension: usize,
    model: String,
    count: usize,
}

/// Document for reranking
#[derive(Debug, Serialize)]
struct RerankDocument {
    id: String,
    text: String,
}

/// Request to embedding service for reranking
#[derive(Debug, Serialize)]
struct RerankRequest {
    query: String,
    documents: Vec<RerankDocument>,
    top_k: Option<usize>,
}

/// Rerank result from embedding service
#[derive(Debug, Deserialize)]
struct RerankResult {
    id: String,
    score: f32,
}

/// Response from reranking service
#[derive(Debug, Deserialize)]
struct RerankResponse {
    results: Vec<RerankResult>,
}

/// Fusion service coordinating hybrid search
pub struct FusionService {
    embedding_service_url: String,
    http_client: reqwest::Client,
}

impl FusionService {
    pub fn new(embedding_service_url: String) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        Self {
            embedding_service_url,
            http_client,
        }
    }

    /// Generate embedding for query text
    pub async fn generate_query_embedding(&self, query: &str) -> Result<Vec<f32>> {
        let url = format!("{}/embed", self.embedding_service_url);

        let request = EmbedRequest {
            text: vec![query.to_string()],
            normalize: true,
        };

        log::debug!("Requesting embedding from: {}", url);

        match self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<EmbedResponse>().await {
                        Ok(embed_response) => {
                            if embed_response.embeddings.is_empty() {
                                Err(anyhow!("No embeddings returned"))
                            } else {
                                Ok(embed_response.embeddings[0].clone())
                            }
                        }
                        Err(e) => Err(anyhow!("Failed to parse embed response: {}", e)),
                    }
                } else {
                    Err(anyhow!("Embedding service returned error: {}", response.status()))
                }
            }
            Err(e) => Err(anyhow!("Failed to connect to embedding service: {}", e)),
        }
    }

    /// Reciprocal Rank Fusion algorithm
    pub fn reciprocal_rank_fusion(
        &self,
        keyword_results: Vec<IntermediateResult>,
        vector_results: Vec<IntermediateResult>,
    ) -> Vec<FusionResult> {
        let mut score_map: HashMap<String, (f32, IntermediateResult)> = HashMap::new();

        // Process keyword results
        for (rank, result) in keyword_results.into_iter().enumerate() {
            let rrf_score = 1.0 / (RRF_K + rank) as f32;
            score_map
                .entry(result.id.clone())
                .and_modify(|(score, _)| *score += rrf_score)
                .or_insert((rrf_score, result));
        }

        // Process vector results
        for (rank, result) in vector_results.into_iter().enumerate() {
            let rrf_score = 1.0 / (RRF_K + rank) as f32;
            score_map
                .entry(result.id.clone())
                .and_modify(|(score, _)| *score += rrf_score)
                .or_insert((rrf_score, result));
        }

        // Convert to FusionResult and sort by score
        let mut fusion_results: Vec<FusionResult> = score_map
            .into_iter()
            .map(|(_, (score, result))| FusionResult {
                id: result.id,
                score,
                source: result.source,
                content: result.content,
                metadata: result.metadata,
            })
            .collect();

        fusion_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Return top 100 results
        fusion_results.truncate(100);
        fusion_results
    }

    /// Rerank results using embedding service
    pub async fn rerank_results(
        &self,
        query: String,
        fusion_results: Vec<FusionResult>,
    ) -> Result<Vec<FusionResult>> {
        if fusion_results.is_empty() {
            return Ok(Vec::new());
        }

        let url = format!("{}/rerank", self.embedding_service_url);

        // Take top 100 for reranking
        let results_to_rerank: Vec<FusionResult> = fusion_results.into_iter().take(100).collect();

        // Prepare rerank request
        let documents: Vec<RerankDocument> = results_to_rerank
            .iter()
            .map(|r| RerankDocument {
                id: r.id.clone(),
                text: r.content.clone(),
            })
            .collect();

        let request = RerankRequest {
            query,
            documents,
            top_k: Some(20), // Final limit
        };

        log::debug!("Requesting reranking from: {}", url);

        match self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<RerankResponse>().await {
                        Ok(rerank_response) => {
                            // Map scores back to fusion results
                            let score_map: HashMap<String, f32> = rerank_response
                                .results
                                .into_iter()
                                .map(|r| (r.id, r.score))
                                .collect();

                            let mut reranked: Vec<FusionResult> = results_to_rerank
                                .into_iter()
                                .filter_map(|mut result| {
                                    if let Some(&rerank_score) = score_map.get(&result.id) {
                                        result.score = rerank_score;
                                        Some(result)
                                    } else {
                                        None
                                    }
                                })
                                .collect();

                            reranked.sort_by(|a, b| {
                                b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
                            });

                            reranked.truncate(20);
                            Ok(reranked)
                        }
                        Err(e) => {
                            log::warn!("Failed to parse rerank response: {}, using RRF scores", e);
                            Ok(self.fallback_to_rrf_scores(results_to_rerank))
                        }
                    }
                } else {
                    log::warn!("Reranking service returned error: {}, using RRF scores", response.status());
                    Ok(self.fallback_to_rrf_scores(results_to_rerank))
                }
            }
            Err(e) => {
                log::warn!("Failed to connect to reranking service: {}, using RRF scores", e);
                Ok(self.fallback_to_rrf_scores(results_to_rerank))
            }
        }
    }

    /// Fallback to RRF scores when reranking fails
    fn fallback_to_rrf_scores(&self, mut results: Vec<FusionResult>) -> Vec<FusionResult> {
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(20);
        results
    }

    /// Main fusion search orchestration
    pub async fn fusion_search(
        &self,
        query: String,
        keyword_results: Vec<IntermediateResult>,
        qdrant_service: &QdrantService,
        collection: &str,
    ) -> Result<Vec<FusionResult>> {
        log::info!("Starting fusion search for query: {}", query);

        // Execute parallel search
        let (keyword_res, vector_res) = tokio::join!(
            async { Ok::<_, anyhow::Error>(keyword_results) },
            async {
                // Generate query embedding
                match self.generate_query_embedding(&query).await {
                    Ok(query_vector) => {
                        // Search Qdrant
                        match qdrant_service.search_vectors(collection, query_vector, 100).await {
                            Ok(results) => {
                                // Convert to IntermediateResult
                                let intermediate: Vec<IntermediateResult> = results
                                    .into_iter()
                                    .enumerate()
                                    .map(|(rank, r)| IntermediateResult {
                                        id: r.id.clone(),
                                        rank,
                                        score: r.score,
                                        source: r.payload.get("source")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("unknown")
                                            .to_string(),
                                        content: r.payload.get("content")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string(),
                                        metadata: r.payload,
                                    })
                                    .collect();
                                Ok(intermediate)
                            }
                            Err(e) => {
                                log::warn!("Vector search failed: {}, using keyword-only", e);
                                Ok(Vec::new())
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("Embedding generation failed: {}, using keyword-only", e);
                        Ok(Vec::new())
                    }
                }
            }
        );

        let keyword_results = keyword_res?;
        let vector_results = vector_res?;

        // Handle degradation cases
        if keyword_results.is_empty() && vector_results.is_empty() {
            log::warn!("Both keyword and vector search returned empty results");
            return Ok(Vec::new());
        }

        if vector_results.is_empty() {
            log::info!("Vector search unavailable, using keyword-only mode");
            let mut results: Vec<FusionResult> = keyword_results
                .into_iter()
                .map(|r| FusionResult {
                    id: r.id,
                    score: r.score,
                    source: r.source,
                    content: r.content,
                    metadata: r.metadata,
                })
                .collect();
            results.truncate(20);
            return Ok(results);
        }

        if keyword_results.is_empty() {
            log::info!("Keyword search returned no results, using vector-only mode");
            let mut results: Vec<FusionResult> = vector_results
                .into_iter()
                .map(|r| FusionResult {
                    id: r.id,
                    score: r.score,
                    source: r.source,
                    content: r.content,
                    metadata: r.metadata,
                })
                .collect();
            results.truncate(20);
            return Ok(results);
        }

        // Perform RRF fusion
        log::debug!("Performing RRF fusion: {} keyword + {} vector results",
                   keyword_results.len(), vector_results.len());
        let fused_results = self.reciprocal_rank_fusion(keyword_results, vector_results);

        // Rerank
        log::debug!("Reranking {} fused results", fused_results.len());
        let final_results = self.rerank_results(query, fused_results).await?;

        log::info!("Fusion search complete: {} final results", final_results.len());
        Ok(final_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_fusion() {
        let fusion_service = FusionService::new("http://localhost:8082".to_string());

        let keyword_results = vec![
            IntermediateResult {
                id: "doc1".to_string(),
                rank: 0,
                score: 10.0,
                source: "code".to_string(),
                content: "content1".to_string(),
                metadata: Value::Null,
            },
            IntermediateResult {
                id: "doc2".to_string(),
                rank: 1,
                score: 8.0,
                source: "code".to_string(),
                content: "content2".to_string(),
                metadata: Value::Null,
            },
        ];

        let vector_results = vec![
            IntermediateResult {
                id: "doc2".to_string(),
                rank: 0,
                score: 0.9,
                source: "code".to_string(),
                content: "content2".to_string(),
                metadata: Value::Null,
            },
            IntermediateResult {
                id: "doc3".to_string(),
                rank: 1,
                score: 0.8,
                source: "code".to_string(),
                content: "content3".to_string(),
                metadata: Value::Null,
            },
        ];

        let results = fusion_service.reciprocal_rank_fusion(keyword_results, vector_results);

        // doc2 appears in both, should have highest combined score
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "doc2");
    }
}
