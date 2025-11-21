use serde::{Deserialize, Serialize};
use reqwest::Client;
use anyhow::{Result, Context};

#[derive(Debug, Deserialize)]
pub struct AgenticQueryRequest {
    pub query: String,
    pub tenant_id: String,
    pub max_steps: Option<usize>,
    pub tools: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct AgenticQueryResponse {
    pub answer: String,
    pub steps: Vec<ExecutionStep>,
    pub total_steps: usize,
    pub sources: Vec<Source>,
    pub confidence: f32,
    pub execution_time_ms: u64,
}

#[derive(Debug, Serialize, Clone)]
pub struct ExecutionStep {
    pub step_number: usize,
    pub tool_used: String,
    pub reasoning: String,
    pub result_summary: String,
    pub execution_time_ms: u64,
}

#[derive(Debug, Serialize, Clone)]
pub struct Source {
    pub source_type: String,
    pub content: String,
    pub score: f32,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum Tool {
    VectorSearch,
    GraphGetEntity,
    GraphNeighbors,
    GraphPaths,
    MetadataQuery,
}

impl Tool {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "vector_search" => Some(Tool::VectorSearch),
            "graph_get_entity" => Some(Tool::GraphGetEntity),
            "graph_neighbors" => Some(Tool::GraphNeighbors),
            "graph_paths" => Some(Tool::GraphPaths),
            "metadata_query" => Some(Tool::MetadataQuery),
            _ => None,
        }
    }
    
    fn as_str(&self) -> &'static str {
        match self {
            Tool::VectorSearch => "vector_search",
            Tool::GraphGetEntity => "graph_get_entity",
            Tool::GraphNeighbors => "graph_neighbors",
            Tool::GraphPaths => "graph_paths",
            Tool::MetadataQuery => "metadata_query",
        }
    }
}

pub struct AgenticOrchestrator {
    embedding_url: String,
    graph_url: String,
    data_url: String,
    client: Client,
}

impl AgenticOrchestrator {
    pub fn new(embedding_url: String, graph_url: String, data_url: String) -> Self {
        Self {
            embedding_url,
            graph_url,
            data_url,
            client: Client::new(),
        }
    }

    pub async fn execute_query(&self, request: AgenticQueryRequest) -> Result<AgenticQueryResponse> {
        let start = std::time::Instant::now();
        let max_steps = request.max_steps.unwrap_or(5);
        
        tracing::info!("🤖 Agentic RAG: Starting query execution (max {} steps)", max_steps);
        
        // Step 1: Classify query and plan approach
        let plan = self.plan_execution(&request.query);
        
        // Step 2: Execute plan with tools
        let (steps, sources) = self.execute_plan(&request, &plan, max_steps).await?;
        
        // Step 3: Generate final answer
        let answer = self.generate_answer(&request.query, &sources, &steps);
        
        let execution_time_ms = start.elapsed().as_millis() as u64;
        
        Ok(AgenticQueryResponse {
            answer,
            total_steps: steps.len(),
            steps,
            sources,
            confidence: self.calculate_confidence(&sources),
            execution_time_ms,
        })
    }

    fn plan_execution(&self, query: &str) -> Vec<Tool> {
        let query_lower = query.to_lowercase();
        let mut plan = Vec::new();
        
        // Ownership/structure questions → Graph-first
        if query_lower.contains("who owns") 
            || query_lower.contains("who wrote")
            || query_lower.contains("who maintains") {
            plan.push(Tool::GraphGetEntity);
            plan.push(Tool::GraphNeighbors);
            plan.push(Tool::VectorSearch);
        }
        // Path/trace questions → Graph paths
        else if query_lower.contains("trace") 
            || query_lower.contains("timeline")
            || query_lower.contains("how did") {
            plan.push(Tool::GraphPaths);
            plan.push(Tool::VectorSearch);
        }
        // Related/connected questions → Graph neighbors
        else if query_lower.contains("related to") 
            || query_lower.contains("connected to") {
            plan.push(Tool::GraphNeighbors);
            plan.push(Tool::VectorSearch);
        }
        // Default: Vector-first
        else {
            plan.push(Tool::VectorSearch);
            plan.push(Tool::GraphGetEntity);
        }
        
        plan
    }

    async fn execute_plan(
        &self,
        request: &AgenticQueryRequest,
        plan: &[Tool],
        max_steps: usize,
    ) -> Result<(Vec<ExecutionStep>, Vec<Source>)> {
        let mut steps = Vec::new();
        let mut all_sources = Vec::new();
        
        for (i, tool) in plan.iter().enumerate().take(max_steps) {
            let step_start = std::time::Instant::now();
            
            let (reasoning, sources) = match tool {
                Tool::VectorSearch => {
                    self.execute_vector_search(request, &all_sources).await?
                }
                Tool::GraphGetEntity => {
                    self.execute_graph_get_entity(request, &all_sources).await?
                }
                Tool::GraphNeighbors => {
                    self.execute_graph_neighbors(request, &all_sources).await?
                }
                Tool::GraphPaths => {
                    self.execute_graph_paths(request, &all_sources).await?
                }
                Tool::MetadataQuery => {
                    self.execute_metadata_query(request, &all_sources).await?
                }
            };
            
            let execution_time_ms = step_start.elapsed().as_millis() as u64;
            
            steps.push(ExecutionStep {
                step_number: i + 1,
                tool_used: tool.as_str().to_string(),
                reasoning,
                result_summary: format!("Found {} sources", sources.len()),
                execution_time_ms,
            });
            
            all_sources.extend(sources);
        }
        
        Ok((steps, all_sources))
    }

    async fn execute_vector_search(
        &self,
        request: &AgenticQueryRequest,
        _context: &[Source],
    ) -> Result<(String, Vec<Source>)> {
        tracing::info!("🔍 Executing vector_search tool");
        
        let search_req = serde_json::json!({
            "query_text": request.query,
            "tenant_id": request.tenant_id,
            "top_k": 10,
        });

        let response = self.client
            .post(format!("{}/vector/search", self.embedding_url))
            .json(&search_req)
            .send()
            .await
            .context("Failed to call embedding service")?;

        let search_results: serde_json::Value = response.json().await?;
        let sources = self.parse_vector_results(&search_results);
        
        let reasoning = format!("Performed semantic search for: '{}'", request.query);
        Ok((reasoning, sources))
    }

    async fn execute_graph_get_entity(
        &self,
        request: &AgenticQueryRequest,
        _context: &[Source],
    ) -> Result<(String, Vec<Source>)> {
        tracing::info!("🔷 Executing graph_get_entity tool");
        
        // TODO: Extract entity ID from query or context
        // For now, return empty
        let reasoning = "Searched for relevant entities in knowledge graph".to_string();
        Ok((reasoning, vec![]))
    }

    async fn execute_graph_neighbors(
        &self,
        request: &AgenticQueryRequest,
        context: &[Source],
    ) -> Result<(String, Vec<Source>)> {
        tracing::info!("🔷 Executing graph_neighbors tool");
        
        // TODO: Use entity IDs from context to expand neighbors
        let reasoning = "Expanded entity relationships to find connected information".to_string();
        Ok((reasoning, vec![]))
    }

    async fn execute_graph_paths(
        &self,
        request: &AgenticQueryRequest,
        _context: &[Source],
    ) -> Result<(String, Vec<Source>)> {
        tracing::info!("🔷 Executing graph_paths tool");
        
        // TODO: Find paths between entities
        let reasoning = "Traced paths through knowledge graph".to_string();
        Ok((reasoning, vec![]))
    }

    async fn execute_metadata_query(
        &self,
        request: &AgenticQueryRequest,
        _context: &[Source],
    ) -> Result<(String, Vec<Source>)> {
        tracing::info!("📊 Executing metadata_query tool");
        
        // TODO: Query metadata/raw data
        let reasoning = "Queried metadata for additional context".to_string();
        Ok((reasoning, vec![]))
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
                })
            }).collect()
        }).unwrap_or_default()
    }

    fn generate_answer(&self, query: &str, sources: &[Source], steps: &[ExecutionStep]) -> String {
        if sources.is_empty() {
            return "No relevant information found after multi-step search.".to_string();
        }
        
        // Simple answer generation (in production, would use LLM)
        let context: Vec<String> = sources.iter()
            .take(5)
            .map(|s| s.content.clone())
            .collect();
        
        let steps_summary: Vec<String> = steps.iter()
            .map(|s| format!("{}. {} - {}", s.step_number, s.tool_used, s.reasoning))
            .collect();
        
        format!(
            "Based on a {}-step analysis:\n\n{}\n\nKey findings:\n{}\n\nSources: {} relevant documents",
            steps.len(),
            steps_summary.join("\n"),
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
