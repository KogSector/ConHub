//! Context Builder Service
//! 
//! Merges results from vector and graph retrieval, applies re-ranking,
//! enforces token budgets, and produces final context blocks for AI agents.

use crate::models::query::{
    ContextBlock, ContextMetadata, RetrievalPlan, RerankStrategy,
};
use std::collections::{HashMap, HashSet};
use tracing::{info, debug};
use uuid::Uuid;

/// Context builder that merges and ranks retrieval results
pub struct ContextBuilder;

impl ContextBuilder {
    /// Merge vector and graph results into final context blocks
    pub fn merge_results(
        vector_results: Vec<ContextBlock>,
        graph_results: Vec<ContextBlock>,
        plan: &RetrievalPlan,
    ) -> Vec<ContextBlock> {
        info!(
            "🔀 Merging {} vector + {} graph results (strategy: {:?})",
            vector_results.len(),
            graph_results.len(),
            plan.rerank
        );
        
        // Deduplicate by source_id
        let mut seen: HashSet<String> = HashSet::new();
        let mut merged: Vec<ContextBlock> = Vec::new();
        
        // Add vector results first (typically higher priority)
        for block in vector_results {
            if seen.insert(block.source_id.clone()) {
                merged.push(block);
            }
        }
        
        // Add graph results
        for block in graph_results {
            if seen.insert(block.source_id.clone()) {
                merged.push(block);
            }
        }
        
        // Apply re-ranking
        merged = Self::rerank(merged, plan.rerank);
        
        // Apply token budget
        merged = Self::apply_token_budget(merged, plan.max_tokens, plan.max_blocks);
        
        info!("✅ Final context: {} blocks", merged.len());
        
        merged
    }
    
    /// Re-rank results based on strategy
    fn rerank(mut blocks: Vec<ContextBlock>, strategy: RerankStrategy) -> Vec<ContextBlock> {
        match strategy {
            RerankStrategy::ScoreBased => {
                // Simple score-based sorting (descending)
                blocks.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
            }
            
            RerankStrategy::ReciprocalRankFusion => {
                // RRF: 1 / (k + rank) where k is typically 60
                // Since we already have scores, we use a modified approach
                // Boost items that appear in multiple retrieval methods
                let k = 60.0;
                
                // Group by source and count occurrences (would need tracking)
                // For now, use score-based with a small boost for diversity
                blocks.sort_by(|a, b| {
                    let score_a = a.score + (1.0 / (k + 1.0)) as f32;
                    let score_b = b.score + (1.0 / (k + 1.0)) as f32;
                    score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            
            RerankStrategy::DiversityAware => {
                // MMR-like: balance relevance with diversity
                if blocks.is_empty() {
                    return blocks;
                }
                
                let mut selected: Vec<ContextBlock> = Vec::new();
                let mut remaining: Vec<ContextBlock> = blocks;
                
                // Always select the highest scoring first
                remaining.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
                
                if let Some(first) = remaining.first().cloned() {
                    selected.push(first);
                    remaining.remove(0);
                }
                
                // Greedily select remaining with diversity consideration
                let lambda = 0.7; // Balance between relevance and diversity
                
                while !remaining.is_empty() && selected.len() < 50 {
                    let mut best_idx = 0;
                    let mut best_mmr = f32::MIN;
                    
                    for (idx, candidate) in remaining.iter().enumerate() {
                        // Calculate max similarity to already selected
                        let max_sim = selected.iter()
                            .map(|s| Self::text_similarity(&candidate.text, &s.text))
                            .fold(0.0_f32, |a, b| a.max(b));
                        
                        // MMR score
                        let mmr = (lambda * candidate.score) - ((1.0 - lambda as f32) * max_sim);
                        
                        if mmr > best_mmr {
                            best_mmr = mmr;
                            best_idx = idx;
                        }
                    }
                    
                    selected.push(remaining.remove(best_idx));
                }
                
                return selected;
            }
            
            RerankStrategy::RecencyBiased => {
                // Boost recent items
                blocks.sort_by(|a, b| {
                    let time_a = a.metadata.timestamp.map(|t| t.timestamp()).unwrap_or(0);
                    let time_b = b.metadata.timestamp.map(|t| t.timestamp()).unwrap_or(0);
                    
                    // Combine score with recency (more recent = higher)
                    let score_a = a.score + (time_a as f32 / 1e10);
                    let score_b = b.score + (time_b as f32 / 1e10);
                    
                    score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
        }
        
        blocks
    }
    
    /// Simple text similarity (Jaccard on words)
    fn text_similarity(a: &str, b: &str) -> f32 {
        let words_a: HashSet<&str> = a.split_whitespace().collect();
        let words_b: HashSet<&str> = b.split_whitespace().collect();
        
        if words_a.is_empty() || words_b.is_empty() {
            return 0.0;
        }
        
        let intersection = words_a.intersection(&words_b).count();
        let union = words_a.union(&words_b).count();
        
        intersection as f32 / union as f32
    }
    
    /// Apply token budget constraints
    fn apply_token_budget(
        blocks: Vec<ContextBlock>,
        max_tokens: u32,
        max_blocks: u32,
    ) -> Vec<ContextBlock> {
        let mut result = Vec::new();
        let mut total_tokens = 0u32;
        
        for block in blocks {
            if result.len() >= max_blocks as usize {
                debug!("📊 Reached max blocks limit: {}", max_blocks);
                break;
            }
            
            if total_tokens + block.token_count > max_tokens {
                debug!("📊 Reached token limit: {} + {} > {}", total_tokens, block.token_count, max_tokens);
                
                // Try to fit a truncated version if we have room
                let remaining_tokens = max_tokens.saturating_sub(total_tokens);
                if remaining_tokens > 100 {
                    // Truncate the block
                    let truncated = Self::truncate_block(block, remaining_tokens);
                    result.push(truncated);
                }
                break;
            }
            
            total_tokens += block.token_count;
            result.push(block);
        }
        
        result
    }
    
    /// Truncate a block to fit within token budget
    fn truncate_block(mut block: ContextBlock, max_tokens: u32) -> ContextBlock {
        // Rough approximation: 1 token ≈ 4 characters
        let max_chars = (max_tokens * 4) as usize;
        
        if block.text.len() > max_chars {
            block.text = format!("{}...", &block.text[..max_chars.saturating_sub(3)]);
            block.token_count = max_tokens;
        }
        
        block
    }
    
    /// Estimate token count for text (rough approximation)
    pub fn estimate_tokens(text: &str) -> u32 {
        // Rough approximation: 1 token ≈ 4 characters for English
        // For code, it's often closer to 3 characters per token
        (text.len() as f32 / 3.5).ceil() as u32
    }
    
    /// Create a context block from raw data
    pub fn create_block(
        source_id: String,
        text: String,
        source_type: String,
        score: f32,
        metadata: ContextMetadata,
    ) -> ContextBlock {
        let token_count = Self::estimate_tokens(&text);
        
        ContextBlock {
            id: Uuid::new_v4(),
            source_id,
            text,
            source_type,
            score,
            token_count,
            metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn make_block(id: &str, score: f32, tokens: u32) -> ContextBlock {
        ContextBlock {
            id: Uuid::new_v4(),
            source_id: id.to_string(),
            text: "test content".repeat(tokens as usize / 3),
            source_type: "test".to_string(),
            score,
            token_count: tokens,
            metadata: ContextMetadata::default(),
        }
    }
    
    #[test]
    fn test_merge_deduplicates() {
        let plan = RetrievalPlan::default();
        
        let vector = vec![
            make_block("a", 0.9, 100),
            make_block("b", 0.8, 100),
        ];
        
        let graph = vec![
            make_block("a", 0.85, 100), // Duplicate
            make_block("c", 0.7, 100),
        ];
        
        let merged = ContextBuilder::merge_results(vector, graph, &plan);
        
        assert_eq!(merged.len(), 3); // a, b, c (a deduplicated)
    }
    
    #[test]
    fn test_token_budget() {
        let blocks = vec![
            make_block("a", 0.9, 3000),
            make_block("b", 0.8, 3000),
            make_block("c", 0.7, 3000),
            make_block("d", 0.6, 3000),
        ];
        
        let result = ContextBuilder::apply_token_budget(blocks, 8000, 20);
        
        // Should only fit 2-3 blocks within 8000 tokens
        assert!(result.len() <= 3);
    }
}
