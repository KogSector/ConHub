use anyhow::Result;
use std::collections::HashMap;
use tracing::{info, warn};

use crate::models::{
    ContextQueryRequest, ContextQueryResponse, ContextBlock, Strategy,
};
use crate::services::{VectorRagClient, GraphRagClient};

/// Decision service that orchestrates retrieval strategies
pub struct DecisionService;

impl DecisionService {
    /// Execute context query with strategy selection
    pub async fn query_context(
        vector_client: &VectorRagClient,
        graph_client: &GraphRagClient,
        request: &ContextQueryRequest,
    ) -> Result<ContextQueryResponse> {
        let start_time = std::time::Instant::now();

        // Determine strategy
        let strategy = match request.strategy {
            Strategy::Auto => Self::auto_select_strategy(&request.query, &request.filters),
            s => s,
        };

        info!("🎯 Using strategy: {:?} for query: '{}'", strategy, request.query);

        // Execute strategy
        let blocks = match strategy {
            Strategy::Vector => {
                Self::execute_vector_strategy(vector_client, request).await?
            }
            Strategy::Graph => {
                Self::execute_graph_strategy(graph_client, request).await?
            }
            Strategy::Hybrid => {
                Self::execute_hybrid_strategy(vector_client, graph_client, request).await?
            }
            Strategy::Auto => {
                // Already selected above, shouldn't reach here
                vec![]
            }
        };

        let processing_time = start_time.elapsed().as_millis() as u64;

        Ok(ContextQueryResponse {
            strategy_used: strategy,
            total_results: blocks.len(),
            blocks,
            processing_time_ms: processing_time,
        })
    }

    /// Auto-select strategy based on query analysis
    fn auto_select_strategy(query: &str, filters: &crate::models::QueryFilters) -> Strategy {
        let query_lower = query.to_lowercase();

        // Rule-based strategy selection
        // Graph RAG for relationship queries
        if query_lower.contains("who") 
            || query_lower.contains("relationship")
            || query_lower.contains("connected")
            || query_lower.contains("related")
            || query_lower.contains("mentions")
            || query_lower.contains("referenced") {
            return Strategy::Graph;
        }

        // Hybrid for complex queries
        if query.split_whitespace().count() > 15 
            || query_lower.contains("how")
            || query_lower.contains("why")
            || query_lower.contains("explain") {
            return Strategy::Hybrid;
        }

        // Vector for simple Q&A
        Strategy::Vector
    }

    /// Execute vector-only strategy
    async fn execute_vector_strategy(
        vector_client: &VectorRagClient,
        request: &ContextQueryRequest,
    ) -> Result<Vec<ContextBlock>> {
        info!("📊 Executing vector strategy");
        vector_client.search(&request.query, &request.filters, request.top_k).await
    }

    /// Execute graph-only strategy
    async fn execute_graph_strategy(
        graph_client: &GraphRagClient,
        request: &ContextQueryRequest,
    ) -> Result<Vec<ContextBlock>> {
        info!("🕸️  Executing graph strategy");
        graph_client.search(&request.query, &request.filters, request.top_k).await
    }

    /// Execute hybrid strategy (vector seeds → graph expansion)
    async fn execute_hybrid_strategy(
        vector_client: &VectorRagClient,
        graph_client: &GraphRagClient,
        request: &ContextQueryRequest,
    ) -> Result<Vec<ContextBlock>> {
        info!("🔀 Executing hybrid strategy");

        // Phase 1: Vector search for initial recall
        let vector_k = (request.top_k / 2).max(5);
        let mut vector_results = vector_client
            .search(&request.query, &request.filters, vector_k)
            .await?;

        info!("  📊 Vector recall: {} results", vector_results.len());

        // Phase 2: Extract seed node IDs from vector results
        let seed_nodes: Vec<uuid::Uuid> = vector_results
            .iter()
            .map(|b| b.chunk_id)
            .collect();

        if seed_nodes.is_empty() {
            return Ok(vector_results);
        }

        // Phase 3: Graph expansion from seeds
        let graph_results = graph_client
            .expand(
                seed_nodes,
                2, // max 2 hops
                vec!["MENTIONS".to_string(), "BELONGS_TO".to_string()],
                request.top_k - vector_results.len(),
            )
            .await?;

        info!("  🕸️  Graph expansion: {} results", graph_results.len());

        // Phase 4: Merge and deduplicate
        let mut seen_chunks = std::collections::HashSet::new();
        let mut merged_results = Vec::new();

        // Add vector results first (higher priority)
        for block in vector_results {
            if seen_chunks.insert(block.chunk_id) {
                merged_results.push(block);
            }
        }

        // Add graph results
        for block in graph_results {
            if seen_chunks.insert(block.chunk_id) {
                merged_results.push(block);
            }
        }

        // Phase 5: Re-rank (simple score-based for now)
        merged_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Trim to top_k
        merged_results.truncate(request.top_k);

        info!("  ✅ Hybrid final: {} results", merged_results.len());

        Ok(merged_results)
    }
}
