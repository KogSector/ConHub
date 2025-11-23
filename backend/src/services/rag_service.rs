use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use reqwest::Client;
use anyhow::{Result, Context};

#[derive(Debug, Deserialize)]
pub struct RagQueryRequest {
    pub query: String,
    pub tenant_id: String,
    pub mode: Option<RagMode>,
    pub filters: Option<RagFilters>,
    pub top_k: Option<usize>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum RagMode {
    Vector,
    Hybrid,
    Agentic,
    Auto, // Automatically choose based on query
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RagFilters {
    pub connector_types: Option<Vec<String>>,
    pub repositories: Option<Vec<String>>,
    pub date_range: Option<DateRange>,
    pub authors: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DateRange {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Serialize)]
pub struct RagQueryResponse {
    pub answer: String,
    pub mode_used: String,
    pub sources: Vec<Source>,
    pub confidence: f32,
    pub query_time_ms: u64,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, Clone)]
pub struct Source {
    pub source_type: String, // "vector", "graph", "hybrid"
    pub content: String,
    pub score: f32,
    pub metadata: serde_json::Value,
    pub citation: Option<String>,
}

pub struct RagService {
    embedding_url: String,
    graph_url: String,
    agentic_url: String,
    client: Client,
}

impl RagService {
    pub fn new(embedding_url: String, graph_url: String, agentic_url: String) -> Self {
        Self {
            embedding_url,
            graph_url,
            agentic_url,
            client: Client::new(),
        }
    }

    pub async fn query(&self, request: RagQueryRequest) -> Result<RagQueryResponse> {
        let start = std::time::Instant::now();
        
        // Determine mode
        let mode = request.mode.clone().unwrap_or(RagMode::Auto);
        let actual_mode = match mode {
            RagMode::Auto => self.classify_query(&request.query),
            other => other,
        };

        let (answer, sources) = match actual_mode {
            RagMode::Vector => self.vector_rag(&request).await?,
            RagMode::Hybrid => self.hybrid_rag(&request).await?,
            RagMode::Agentic => self.agentic_rag(&request).await?,
            RagMode::Auto => unreachable!(),
        };

        let query_time_ms = start.elapsed().as_millis() as u64;
        let confidence = self.calculate_confidence(&sources);

        Ok(RagQueryResponse {
            answer,
            mode_used: format!("{:?}", actual_mode),
            sources,
            confidence,
            query_time_ms,
            metadata: serde_json::json!({
                "query": request.query,
                "tenant_id": request.tenant_id,
            }),
        })
    }

    fn classify_query(&self, query: &str) -> RagMode {
        let query_lower = query.to_lowercase();
        
        // Ownership/structure questions → Hybrid (graph + vector)
        if query_lower.contains("who owns") 
            || query_lower.contains("who wrote")
            || query_lower.contains("who maintains")
            || query_lower.contains("related to")
            || query_lower.contains("connected to") {
            return RagMode::Hybrid;
        }
        
        // Multi-step/investigation questions → Agentic
        if query_lower.contains("trace")
            || query_lower.contains("investigate")
            || query_lower.contains("how did")
            || query_lower.contains("timeline") {
            return RagMode::Agentic;
        }
        
        // Default to vector for content questions
        RagMode::Vector
    }

    async fn vector_rag(&self, request: &RagQueryRequest) -> Result<(String, Vec<Source>)> {
        log::info!("Executing Vector RAG for query: {}", request.query);
        
        // Call embedding service for vector search
        let search_req = serde_json::json!({
            "query_text": request.query,
            "tenant_id": request.tenant_id,
            "top_k": request.top_k.unwrap_or(10),
            "filters": request.filters,
        });

        let response = self.client
            .post(format!("{}/vector/search", self.embedding_url))
            .json(&search_req)
            .send()
            .await
            .context("Failed to call embedding service")?;

        let search_results: serde_json::Value = response.json().await?;
        
        // Convert to sources
        let sources = self.parse_vector_results(&search_results);
        
        // Generate answer from sources
        let answer = self.generate_answer_from_sources(&request.query, &sources);
        
        Ok((answer, sources))
    }

    async fn hybrid_rag(&self, request: &RagQueryRequest) -> Result<(String, Vec<Source>)> {
        log::info!("Executing Hybrid RAG (Graph + Vector) for query: {}", request.query);
        
        // Step 1: Graph search for entities
        let graph_results = self.graph_search(&request.query, &request.tenant_id).await?;
        
        // Step 2: Vector search with entity context
        let vector_results = self.vector_rag(request).await?;
        
        // Step 3: Fuse results
        let mut all_sources = graph_results.clone();
        all_sources.extend(vector_results.1);
        
        // Rerank based on graph proximity + vector similarity
        let reranked_sources = self.rerank_sources(all_sources);
        
        // Generate answer
        let answer = self.generate_answer_from_sources(&request.query, &reranked_sources);
        
        Ok((answer, reranked_sources))
    }

    async fn agentic_rag(&self, request: &RagQueryRequest) -> Result<(String, Vec<Source>)> {
        log::info!("Executing Agentic RAG for query: {}", request.query);
        
        // Call agentic service for multi-step orchestration
        let agentic_req = serde_json::json!({
            "query": request.query,
            "tenant_id": request.tenant_id,
            "max_steps": 5,
        });

        let response = self.client
            .post(format!("{}/api/agentic/query", self.agentic_url))
            .json(&agentic_req)
            .send()
            .await
            .context("Failed to call agentic service")?;

        let agentic_response: serde_json::Value = response.json().await?;
        
        // Extract answer and sources from agentic response
        let answer = agentic_response.get("answer")
            .and_then(|a| a.as_str())
            .unwrap_or("No answer generated")
            .to_string();
        
        let sources = agentic_response.get("sources")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter().filter_map(|s| {
                    Some(Source {
                        source_type: s.get("source_type")?.as_str()?.to_string(),
                        content: s.get("content")?.as_str()?.to_string(),
                        score: s.get("score")?.as_f64()? as f32,
                        metadata: s.get("metadata")?.clone(),
                        citation: None,
                    })
                }).collect()
            })
            .unwrap_or_default();
        
        Ok((answer, sources))
    }

    async fn graph_search(&self, query: &str, tenant_id: &str) -> Result<Vec<Source>> {
        // Call graph service for entity/relationship search
        let search_req = serde_json::json!({
            "query": query,
            "tenant_id": tenant_id,
        });

        let response = self.client
            .post(format!("{}/api/graph/query", self.graph_url))
            .json(&search_req)
            .send()
            .await
            .context("Failed to call graph service")?;

        let graph_results: serde_json::Value = response.json().await?;
        
        Ok(self.parse_graph_results(&graph_results))
    }

    fn parse_vector_results(&self, results: &serde_json::Value) -> Vec<Source> {
        let results_array = results.get("results").and_then(|r| r.as_array());
        
        results_array.map(|arr| {
            arr.iter().filter_map(|r| {
                Some(Source {
                    source_type: "vector".to_string(),
                    content: r.get("content")?.as_str()?.to_string(),
                    score: r.get("score")?.as_f64()? as f32,
                    metadata: r.get("metadata")?.clone(),
                    citation: r.get("source").and_then(|s| s.as_str()).map(String::from),
                })
            }).collect()
        }).unwrap_or_default()
    }

    fn parse_graph_results(&self, results: &serde_json::Value) -> Vec<Source> {
        let entities = results.get("entities").and_then(|e| e.as_array());
        
        entities.map(|arr| {
            arr.iter().filter_map(|e| {
                Some(Source {
                    source_type: "graph".to_string(),
                    content: e.get("name")?.as_str()?.to_string(),
                    score: e.get("relevance_score")?.as_f64().unwrap_or(0.5) as f32,
                    metadata: e.clone(),
                    citation: e.get("source_id").and_then(|s| s.as_str()).map(String::from),
                })
            }).collect()
        }).unwrap_or_default()
    }

    fn rerank_sources(&self, mut sources: Vec<Source>) -> Vec<Source> {
        // Simple reranking: boost graph results slightly, then sort by score
        for source in &mut sources {
            if source.source_type == "graph" {
                source.score *= 1.1; // 10% boost for graph results
            }
        }
        
        sources.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        sources.truncate(20); // Top 20 results
        sources
    }

    fn generate_answer_from_sources(&self, query: &str, sources: &[Source]) -> String {
        if sources.is_empty() {
            return "No relevant information found.".to_string();
        }
        
        // Simple answer generation (in production, would use LLM)
        let context: Vec<String> = sources.iter()
            .take(5)
            .map(|s| s.content.clone())
            .collect();
        
        format!(
            "Based on the available information:\n\n{}\n\nSources: {} relevant documents found.",
            context.join("\n\n"),
            sources.len()
        )
    }

    fn calculate_confidence(&self, sources: &[Source]) -> f32 {
        if sources.is_empty() {
            return 0.0;
        }
        
        let avg_score: f32 = sources.iter().map(|s| s.score).sum::<f32>() / sources.len() as f32;
        avg_score.min(1.0)
    }
}
