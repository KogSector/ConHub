//! Query Analysis Service
//! 
//! Analyzes natural language queries to determine:
//! - Query kind (fact lookup, explanation, topology, etc.)
//! - Modality hint (code, docs, robot memory, etc.)
//! - Suggested retrieval strategy

use crate::models::query::{
    QueryKind, ModalityHint, RetrievalStrategy, RetrievalPlan, GraphQuerySpec, RerankStrategy,
    MemoryQuery, QueryAnalysisResult, ExtractedEntity,
};
use std::collections::HashMap;
use tracing::info;

/// Query analyzer that classifies queries and suggests retrieval strategies
pub struct QueryAnalyzer {
    /// Keywords that suggest graph/topology queries
    topology_keywords: Vec<&'static str>,
    
    /// Keywords that suggest episodic queries
    episodic_keywords: Vec<&'static str>,
    
    /// Keywords that suggest how-to queries
    howto_keywords: Vec<&'static str>,
    
    /// Keywords that suggest troubleshooting
    troubleshooting_keywords: Vec<&'static str>,
    
    /// Keywords that suggest code-related queries
    code_keywords: Vec<&'static str>,
}

impl QueryAnalyzer {
    pub fn new() -> Self {
        Self {
            topology_keywords: vec![
                "who", "whom", "whose",
                "depends on", "dependency", "dependencies",
                "related to", "relationship", "relationships",
                "connected", "connection", "connections",
                "references", "referenced by", "mentions",
                "owns", "owned by", "owner",
                "created by", "author", "contributor",
                "calls", "called by", "imports", "imported by",
                "inherits", "extends", "implements",
                "parent", "child", "children",
                "upstream", "downstream",
            ],
            
            episodic_keywords: vec![
                "when", "last time", "previously",
                "happened", "occurred", "history",
                "yesterday", "today", "this week", "last week",
                "ago", "before", "after", "since",
                "episode", "event", "incident",
                "saw", "seen", "observed", "detected",
                "did i", "have i", "was there",
            ],
            
            howto_keywords: vec![
                "how to", "how do i", "how can i",
                "steps to", "guide", "tutorial",
                "setup", "configure", "install",
                "create", "build", "deploy",
                "implement", "write", "add",
            ],
            
            troubleshooting_keywords: vec![
                "why", "cause", "reason",
                "error", "bug", "issue", "problem",
                "fail", "failed", "failing", "failure",
                "not working", "doesn't work", "broken",
                "debug", "fix", "resolve", "solve",
                "crash", "exception", "traceback",
            ],
            
            code_keywords: vec![
                "function", "method", "class", "struct",
                "variable", "parameter", "argument",
                "import", "module", "package", "crate",
                "api", "endpoint", "route", "handler",
                "type", "interface", "trait",
                "async", "await", "promise",
                "test", "spec", "unit test",
                "impl", "fn", "def", "const", "let",
            ],
        }
    }
    
    /// Analyze a query and return classification results
    pub fn analyze(&self, query: &MemoryQuery) -> QueryAnalysisResult {
        let query_lower = query.query.to_lowercase();
        
        // Detect query kind
        let (kind, kind_confidence) = self.detect_query_kind(&query_lower, query);
        
        // Detect modality hint
        let (modality, modality_confidence) = self.detect_modality(&query_lower, query);
        
        // Extract entities (basic)
        let entities = self.extract_entities(&query.query);
        
        // Build suggested plan
        let suggested_plan = self.build_plan(kind, modality, query);
        
        info!(
            "🔍 Query analysis: kind={:?} ({:.2}), modality={:?} ({:.2})",
            kind, kind_confidence, modality, modality_confidence
        );
        
        QueryAnalysisResult {
            kind,
            modality,
            confidence: (kind_confidence + modality_confidence) / 2.0,
            entities,
            suggested_plan,
        }
    }
    
    /// Detect the query kind based on keywords and patterns
    fn detect_query_kind(&self, query_lower: &str, query: &MemoryQuery) -> (QueryKind, f32) {
        let mut scores: HashMap<QueryKind, f32> = HashMap::new();
        
        // Check topology keywords
        for kw in &self.topology_keywords {
            if query_lower.contains(kw) {
                *scores.entry(QueryKind::TopologyQuestion).or_insert(0.0) += 0.3;
            }
        }
        
        // Check episodic keywords
        for kw in &self.episodic_keywords {
            if query_lower.contains(kw) {
                *scores.entry(QueryKind::EpisodicLookup).or_insert(0.0) += 0.3;
            }
        }
        
        // Check how-to keywords
        for kw in &self.howto_keywords {
            if query_lower.contains(kw) {
                *scores.entry(QueryKind::HowTo).or_insert(0.0) += 0.3;
            }
        }
        
        // Check troubleshooting keywords
        for kw in &self.troubleshooting_keywords {
            if query_lower.contains(kw) {
                *scores.entry(QueryKind::Troubleshooting).or_insert(0.0) += 0.3;
            }
        }
        
        // Question patterns
        if query_lower.starts_with("what is") || query_lower.starts_with("what's") {
            *scores.entry(QueryKind::FactLookup).or_insert(0.0) += 0.4;
        }
        
        if query_lower.starts_with("explain") || query_lower.contains("how does") {
            *scores.entry(QueryKind::Explainer).or_insert(0.0) += 0.4;
        }
        
        if query_lower.contains("difference between") || query_lower.contains("compare") {
            *scores.entry(QueryKind::Comparison).or_insert(0.0) += 0.5;
        }
        
        if query_lower.starts_with("how many") || query_lower.starts_with("list all") || query_lower.contains("summary of") {
            *scores.entry(QueryKind::Aggregation).or_insert(0.0) += 0.5;
        }
        
        if query_lower.contains("should i") || query_lower.contains("recommend") || query_lower.contains("suggest") {
            *scores.entry(QueryKind::TaskSupport).or_insert(0.0) += 0.4;
        }
        
        // Time range present suggests episodic
        if query.time_range.is_some() {
            *scores.entry(QueryKind::EpisodicLookup).or_insert(0.0) += 0.2;
        }
        
        // Find best match
        if let Some((&kind, &score)) = scores.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()) {
            let confidence = (score / 1.0).min(0.95);
            (kind, confidence)
        } else {
            // Default to generic with low confidence
            (QueryKind::Generic, 0.3)
        }
    }
    
    /// Detect modality hint
    fn detect_modality(&self, query_lower: &str, query: &MemoryQuery) -> (ModalityHint, f32) {
        // Check explicit source filters first
        if !query.sources.is_empty() {
            for source in &query.sources {
                let source_lower = source.to_lowercase();
                if source_lower.contains("robot") || source_lower.contains("episode") {
                    return (ModalityHint::RobotEpisodic, 0.9);
                }
                if source_lower.contains("semantic") || source_lower.contains("fact") {
                    return (ModalityHint::RobotSemantic, 0.9);
                }
                if source_lower.contains("code") || source_lower.contains("github") {
                    return (ModalityHint::Code, 0.9);
                }
                if source_lower.contains("doc") || source_lower.contains("wiki") {
                    return (ModalityHint::Docs, 0.9);
                }
                if source_lower.contains("slack") || source_lower.contains("chat") {
                    return (ModalityHint::Chat, 0.9);
                }
                if source_lower.contains("ticket") || source_lower.contains("issue") {
                    return (ModalityHint::Tickets, 0.9);
                }
            }
        }
        
        // Check for code keywords
        for kw in &self.code_keywords {
            if query_lower.contains(kw) {
                return (ModalityHint::Code, 0.7);
            }
        }
        
        // Robot-related
        if query_lower.contains("robot") || query_lower.contains("sensor") || query_lower.contains("episode") {
            return (ModalityHint::RobotEpisodic, 0.7);
        }
        
        // Chat-related
        if query_lower.contains("message") || query_lower.contains("conversation") || query_lower.contains("said") {
            return (ModalityHint::Chat, 0.6);
        }
        
        // Issue/ticket related
        if query_lower.contains("issue") || query_lower.contains("bug") || query_lower.contains("ticket") {
            return (ModalityHint::Tickets, 0.6);
        }
        
        // Default to mixed
        (ModalityHint::Mixed, 0.4)
    }
    
    /// Extract entities from the query (basic implementation)
    fn extract_entities(&self, query: &str) -> Vec<ExtractedEntity> {
        let mut entities = Vec::new();
        
        // Look for quoted strings
        let mut in_quote = false;
        let mut current = String::new();
        let mut quote_char = '"';
        
        for ch in query.chars() {
            if ch == '"' || ch == '\'' || ch == '`' {
                if in_quote && ch == quote_char {
                    if !current.is_empty() {
                        entities.push(ExtractedEntity {
                            text: current.clone(),
                            entity_type: "quoted".to_string(),
                            confidence: 0.8,
                        });
                    }
                    current.clear();
                    in_quote = false;
                } else if !in_quote {
                    in_quote = true;
                    quote_char = ch;
                }
            } else if in_quote {
                current.push(ch);
            }
        }
        
        // Look for file paths
        for word in query.split_whitespace() {
            if word.contains('/') && word.len() > 3 {
                entities.push(ExtractedEntity {
                    text: word.to_string(),
                    entity_type: "path".to_string(),
                    confidence: 0.7,
                });
            }
            
            // Look for common file extensions
            if word.ends_with(".rs") || word.ends_with(".ts") || word.ends_with(".py") 
                || word.ends_with(".js") || word.ends_with(".go") || word.ends_with(".java") {
                entities.push(ExtractedEntity {
                    text: word.to_string(),
                    entity_type: "file".to_string(),
                    confidence: 0.8,
                });
            }
        }
        
        entities
    }
    
    /// Build a retrieval plan based on analysis
    fn build_plan(&self, kind: QueryKind, modality: ModalityHint, query: &MemoryQuery) -> RetrievalPlan {
        let mut plan = RetrievalPlan::default();
        
        // Set strategy based on query kind
        plan.strategy = match kind {
            QueryKind::TopologyQuestion => RetrievalStrategy::GraphOnly,
            QueryKind::FactLookup => RetrievalStrategy::VectorOnly,
            QueryKind::EpisodicLookup => RetrievalStrategy::VectorOnly, // Time-filtered vector
            QueryKind::Explainer | QueryKind::HowTo => RetrievalStrategy::Hybrid,
            QueryKind::Troubleshooting => RetrievalStrategy::VectorThenGraph,
            QueryKind::TaskSupport => RetrievalStrategy::Hybrid,
            QueryKind::Comparison => RetrievalStrategy::VectorOnly,
            QueryKind::Aggregation => RetrievalStrategy::VectorOnly,
            QueryKind::Generic => RetrievalStrategy::VectorOnly,
        };
        
        // Override if force_strategy is set
        if let Some(forced) = query.force_strategy {
            plan.strategy = forced;
        }
        
        // Set collections based on modality
        plan.vector_collections = match modality {
            ModalityHint::Code => vec!["code".to_string(), "docs".to_string()],
            ModalityHint::Docs => vec!["docs".to_string(), "wikis".to_string()],
            ModalityHint::Chat => vec!["chat".to_string(), "slack".to_string()],
            ModalityHint::Tickets => vec!["tickets".to_string(), "issues".to_string()],
            ModalityHint::RobotEpisodic => vec!["robot_episodic_memory".to_string()],
            ModalityHint::RobotSemantic => vec!["robot_semantic_memory".to_string()],
            ModalityHint::Mixed => vec!["default".to_string()],
        };
        
        // Set graph queries for topology questions
        if kind == QueryKind::TopologyQuestion {
            plan.graph_queries.push(GraphQuerySpec {
                start_node_types: vec!["Document".to_string(), "Function".to_string(), "Class".to_string()],
                edge_types: vec![
                    "MENTIONS".to_string(),
                    "IMPORTS".to_string(),
                    "CALLS".to_string(),
                    "BELONGS_TO".to_string(),
                    "DEPENDS_ON".to_string(),
                ],
                max_hops: 2,
                filters: HashMap::new(),
            });
        }
        
        // Set limits from query
        plan.max_blocks = query.max_blocks;
        plan.max_tokens = query.max_tokens;
        
        // Set rerank strategy
        plan.rerank = match kind {
            QueryKind::EpisodicLookup => RerankStrategy::RecencyBiased,
            QueryKind::Explainer | QueryKind::HowTo => RerankStrategy::DiversityAware,
            _ => RerankStrategy::ScoreBased,
        };
        
        plan.include_provenance = query.include_debug;
        
        plan
    }
}

impl Default for QueryAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    
    fn make_query(q: &str) -> MemoryQuery {
        MemoryQuery {
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            query: q.to_string(),
            sources: vec![],
            time_range: None,
            filters: HashMap::new(),
            max_blocks: 20,
            max_tokens: 8000,
            force_strategy: None,
            include_debug: false,
        }
    }
    
    #[test]
    fn test_topology_detection() {
        let analyzer = QueryAnalyzer::new();
        let query = make_query("Who owns the auth module?");
        let result = analyzer.analyze(&query);
        assert_eq!(result.kind, QueryKind::TopologyQuestion);
    }
    
    #[test]
    fn test_howto_detection() {
        let analyzer = QueryAnalyzer::new();
        let query = make_query("How do I configure authentication?");
        let result = analyzer.analyze(&query);
        assert_eq!(result.kind, QueryKind::HowTo);
    }
    
    #[test]
    fn test_troubleshooting_detection() {
        let analyzer = QueryAnalyzer::new();
        let query = make_query("Why is the database connection failing?");
        let result = analyzer.analyze(&query);
        assert_eq!(result.kind, QueryKind::Troubleshooting);
    }
}
