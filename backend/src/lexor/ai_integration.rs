use crate::lexor::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeContext {
    pub symbols: Vec<Symbol>,
    pub related_files: Vec<String>,
    pub dependencies: Vec<String>,
    pub complexity_metrics: ComplexityMetrics,
    pub usage_patterns: Vec<UsagePattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePattern {
    pub pattern_type: String,
    pub frequency: u32,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub maintainability_index: f32,
    pub technical_debt_ratio: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIQuery {
    pub query: String,
    pub context_type: ContextType,
    pub include_examples: bool,
    pub max_context_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextType {
    Implementation,
    Documentation,
    Testing,
    Debugging,
    Refactoring,
}

pub struct AIContextEngine {
    symbol_graph: HashMap<String, Vec<String>>,
    dependency_graph: HashMap<String, Vec<String>>,
    usage_analytics: HashMap<String, UsagePattern>,
}

impl AIContextEngine {
    pub fn new() -> Self {
        Self {
            symbol_graph: HashMap::new(),
            dependency_graph: HashMap::new(),
            usage_analytics: HashMap::new(),
        }
    }

    pub fn build_context_for_ai(&self, query: &AIQuery, search_results: &SearchResult) -> CodeContext {
        let mut symbols = Vec::new();
        let mut related_files = Vec::new();
        let mut dependencies = Vec::new();

        // Extract symbols from search results
        for hit in &search_results.results {
            symbols.extend(hit.symbols.clone());
            related_files.push(hit.file.path.to_string_lossy().to_string());
        }

        // Find related dependencies
        for file_path in &related_files {
            if let Some(deps) = self.dependency_graph.get(file_path) {
                dependencies.extend(deps.clone());
            }
        }

        // Calculate aggregate complexity
        let complexity_metrics = self.calculate_aggregate_complexity(&search_results.results);

        // Extract usage patterns
        let usage_patterns = self.extract_usage_patterns(&symbols);

        CodeContext {
            symbols,
            related_files: related_files.into_iter().take(query.max_context_files).collect(),
            dependencies: dependencies.into_iter().collect::<std::collections::HashSet<_>>().into_iter().collect(),
            complexity_metrics,
            usage_patterns,
        }
    }

    fn calculate_aggregate_complexity(&self, hits: &[SearchHit]) -> ComplexityMetrics {
        let total_files = hits.len() as f32;
        if total_files == 0.0 {
            return ComplexityMetrics {
                cyclomatic_complexity: 0,
                cognitive_complexity: 0,
                maintainability_index: 100.0,
                technical_debt_ratio: 0.0,
            };
        }

        let avg_complexity = hits.iter()
            .map(|hit| hit.file.lines as f32 / 10.0) // Simplified complexity calculation
            .sum::<f32>() / total_files;

        ComplexityMetrics {
            cyclomatic_complexity: avg_complexity as u32,
            cognitive_complexity: (avg_complexity * 1.2) as u32,
            maintainability_index: (100.0 - avg_complexity).max(0.0),
            technical_debt_ratio: (avg_complexity / 100.0).min(1.0),
        }
    }

    fn extract_usage_patterns(&self, symbols: &[Symbol]) -> Vec<UsagePattern> {
        let mut patterns = HashMap::new();

        for symbol in symbols {
            let pattern_key = format!("{:?}", symbol.symbol_type);
            let entry = patterns.entry(pattern_key.clone()).or_insert(UsagePattern {
                pattern_type: pattern_key,
                frequency: 0,
                examples: Vec::new(),
            });
            
            entry.frequency += 1;
            if entry.examples.len() < 3 {
                entry.examples.push(symbol.name.clone());
            }
        }

        patterns.into_values().collect()
    }

    pub fn generate_ai_prompt(&self, query: &AIQuery, context: &CodeContext) -> String {
        let mut prompt = format!("Query: {}\n\n", query.query);
        
        prompt.push_str("Code Context:\n");
        prompt.push_str(&format!("- {} files analyzed\n", context.related_files.len()));
        prompt.push_str(&format!("- {} symbols found\n", context.symbols.len()));
        prompt.push_str(&format!("- Complexity: {}\n", context.complexity_metrics.cyclomatic_complexity));
        
        if query.include_examples {
            prompt.push_str("\nKey Symbols:\n");
            for symbol in context.symbols.iter().take(10) {
                prompt.push_str(&format!("- {} ({}:{})\n", symbol.name, symbol.symbol_type as u8, symbol.line));
            }
        }

        prompt.push_str("\nUsage Patterns:\n");
        for pattern in &context.usage_patterns {
            prompt.push_str(&format!("- {}: {} occurrences\n", pattern.pattern_type, pattern.frequency));
        }

        prompt
    }

    pub fn update_symbol_graph(&mut self, file_symbols: HashMap<String, Vec<Symbol>>) {
        for (file_path, symbols) in file_symbols {
            let symbol_names: Vec<String> = symbols.iter().map(|s| s.name.clone()).collect();
            self.symbol_graph.insert(file_path, symbol_names);
        }
    }

    pub fn update_dependency_graph(&mut self, dependencies: HashMap<String, Vec<String>>) {
        self.dependency_graph = dependencies;
    }
}