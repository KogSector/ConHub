//! Ingestion Cost Policy
//!
//! Determines whether chunks should be sent to vector_rag, graph_rag, or both.
//! Helps control costs by avoiding redundant indexing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use conhub_models::chunking::SourceKind;
use tracing::info;

/// Ingestion target flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestionTargets {
    /// Send to vector RAG for semantic search
    pub enable_vector: bool,
    /// Send to graph RAG for relationship queries
    pub enable_graph: bool,
}

impl Default for IngestionTargets {
    fn default() -> Self {
        Self {
            enable_vector: true,
            enable_graph: true,
        }
    }
}

impl IngestionTargets {
    /// Both vector and graph enabled
    pub fn both() -> Self {
        Self { enable_vector: true, enable_graph: true }
    }

    /// Vector only - good for simple semantic search
    pub fn vector_only() -> Self {
        Self { enable_vector: true, enable_graph: false }
    }

    /// Graph only - good for relationship-heavy data
    pub fn graph_only() -> Self {
        Self { enable_vector: false, enable_graph: true }
    }

    /// No indexing - for passthrough or skip
    pub fn none() -> Self {
        Self { enable_vector: false, enable_graph: false }
    }
}

/// Policy rule for determining ingestion targets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostPolicyRule {
    /// Source kind to match (None = match all)
    pub source_kind: Option<SourceKind>,
    /// Content type prefix to match (None = match all)
    pub content_type_prefix: Option<String>,
    /// Language to match (for code) (None = match all)
    pub language: Option<String>,
    /// Minimum chunk size in tokens (None = no minimum)
    pub min_tokens: Option<usize>,
    /// Maximum chunk size in tokens (None = no maximum)
    pub max_tokens: Option<usize>,
    /// Ingestion targets if rule matches
    pub targets: IngestionTargets,
    /// Priority (higher = checked first)
    pub priority: i32,
}

/// Cost policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostPolicy {
    /// Policy name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Policy rules (checked in priority order)
    pub rules: Vec<CostPolicyRule>,
    /// Default targets if no rule matches
    pub default_targets: IngestionTargets,
}

impl Default for CostPolicy {
    fn default() -> Self {
        Self::balanced()
    }
}

impl CostPolicy {
    /// Balanced policy - index most content in both stores
    pub fn balanced() -> Self {
        Self {
            name: "balanced".to_string(),
            description: Some("Balanced indexing in both vector and graph stores".to_string()),
            rules: vec![
                // Code: Both for better code understanding
                CostPolicyRule {
                    source_kind: Some(SourceKind::CodeRepo),
                    content_type_prefix: None,
                    language: None,
                    min_tokens: None,
                    max_tokens: None,
                    targets: IngestionTargets::both(),
                    priority: 100,
                },
                // Ticketing/Issues: Mostly graph for relationships
                CostPolicyRule {
                    source_kind: Some(SourceKind::Ticketing),
                    content_type_prefix: None,
                    language: None,
                    min_tokens: None,
                    max_tokens: None,
                    targets: IngestionTargets::both(),
                    priority: 90,
                },
                // Chat: Graph for conversation flow, vector for search
                CostPolicyRule {
                    source_kind: Some(SourceKind::Chat),
                    content_type_prefix: None,
                    language: None,
                    min_tokens: None,
                    max_tokens: None,
                    targets: IngestionTargets::both(),
                    priority: 80,
                },
                // Documents: Vector primary, graph secondary
                CostPolicyRule {
                    source_kind: Some(SourceKind::Document),
                    content_type_prefix: None,
                    language: None,
                    min_tokens: None,
                    max_tokens: None,
                    targets: IngestionTargets::both(),
                    priority: 70,
                },
            ],
            default_targets: IngestionTargets::both(),
        }
    }

    /// Vector-first policy - prioritize semantic search, minimal graph
    pub fn vector_first() -> Self {
        Self {
            name: "vector_first".to_string(),
            description: Some("Vector-first policy - good for simple semantic search workloads".to_string()),
            rules: vec![
                // Code: Vector only (AST provides enough structure)
                CostPolicyRule {
                    source_kind: Some(SourceKind::CodeRepo),
                    content_type_prefix: None,
                    language: None,
                    min_tokens: None,
                    max_tokens: None,
                    targets: IngestionTargets::vector_only(),
                    priority: 100,
                },
                // Ticketing: Both (relationships are important)
                CostPolicyRule {
                    source_kind: Some(SourceKind::Ticketing),
                    content_type_prefix: None,
                    language: None,
                    min_tokens: None,
                    max_tokens: None,
                    targets: IngestionTargets::both(),
                    priority: 90,
                },
                // Chat: Vector only for search
                CostPolicyRule {
                    source_kind: Some(SourceKind::Chat),
                    content_type_prefix: None,
                    language: None,
                    min_tokens: None,
                    max_tokens: None,
                    targets: IngestionTargets::vector_only(),
                    priority: 80,
                },
            ],
            default_targets: IngestionTargets::vector_only(),
        }
    }

    /// Graph-first policy - prioritize relationships, minimal vector
    pub fn graph_first() -> Self {
        Self {
            name: "graph_first".to_string(),
            description: Some("Graph-first policy - good for relationship-heavy workloads".to_string()),
            rules: vec![
                // Code: Both (need semantic search for code)
                CostPolicyRule {
                    source_kind: Some(SourceKind::CodeRepo),
                    content_type_prefix: None,
                    language: None,
                    min_tokens: None,
                    max_tokens: None,
                    targets: IngestionTargets::both(),
                    priority: 100,
                },
                // Ticketing: Graph only (relationships are key)
                CostPolicyRule {
                    source_kind: Some(SourceKind::Ticketing),
                    content_type_prefix: None,
                    language: None,
                    min_tokens: None,
                    max_tokens: None,
                    targets: IngestionTargets::graph_only(),
                    priority: 90,
                },
                // Chat: Graph only (conversation flow)
                CostPolicyRule {
                    source_kind: Some(SourceKind::Chat),
                    content_type_prefix: None,
                    language: None,
                    min_tokens: None,
                    max_tokens: None,
                    targets: IngestionTargets::graph_only(),
                    priority: 80,
                },
            ],
            default_targets: IngestionTargets::graph_only(),
        }
    }

    /// Economy policy - minimize storage costs
    pub fn economy() -> Self {
        Self {
            name: "economy".to_string(),
            description: Some("Economy policy - minimize storage and compute costs".to_string()),
            rules: vec![
                // Code: Vector only for search
                CostPolicyRule {
                    source_kind: Some(SourceKind::CodeRepo),
                    content_type_prefix: None,
                    language: None,
                    min_tokens: None,
                    max_tokens: None,
                    targets: IngestionTargets::vector_only(),
                    priority: 100,
                },
                // Small chunks: Skip (too granular)
                CostPolicyRule {
                    source_kind: None,
                    content_type_prefix: None,
                    language: None,
                    min_tokens: None,
                    max_tokens: Some(50),
                    targets: IngestionTargets::none(),
                    priority: 200,
                },
            ],
            default_targets: IngestionTargets::vector_only(),
        }
    }

    /// Evaluate the policy for a given chunk context
    pub fn evaluate(
        &self,
        source_kind: &SourceKind,
        content_type: Option<&str>,
        language: Option<&str>,
        token_count: usize,
    ) -> IngestionTargets {
        // Sort rules by priority (descending)
        let mut rules: Vec<_> = self.rules.iter().collect();
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        for rule in rules {
            // Check source kind
            if let Some(ref sk) = rule.source_kind {
                if sk != source_kind {
                    continue;
                }
            }

            // Check content type prefix
            if let Some(ref prefix) = rule.content_type_prefix {
                if let Some(ct) = content_type {
                    if !ct.starts_with(prefix) {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // Check language
            if let Some(ref lang) = rule.language {
                if let Some(l) = language {
                    if l != lang {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // Check token bounds
            if let Some(min) = rule.min_tokens {
                if token_count < min {
                    continue;
                }
            }
            if let Some(max) = rule.max_tokens {
                if token_count > max {
                    continue;
                }
            }

            // Rule matches
            return rule.targets;
        }

        // No rule matched, use default
        self.default_targets
    }
}

/// Cost policy manager
pub struct CostPolicyManager {
    policies: HashMap<String, CostPolicy>,
    active_policy: String,
}

impl Default for CostPolicyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CostPolicyManager {
    pub fn new() -> Self {
        let mut policies = HashMap::new();
        policies.insert("balanced".to_string(), CostPolicy::balanced());
        policies.insert("vector_first".to_string(), CostPolicy::vector_first());
        policies.insert("graph_first".to_string(), CostPolicy::graph_first());
        policies.insert("economy".to_string(), CostPolicy::economy());

        Self {
            policies,
            active_policy: "balanced".to_string(),
        }
    }

    /// Get the active policy
    pub fn active(&self) -> &CostPolicy {
        self.policies.get(&self.active_policy)
            .unwrap_or_else(|| self.policies.get("balanced").unwrap())
    }

    /// Set active policy by name
    pub fn set_active(&mut self, name: &str) -> bool {
        if self.policies.contains_key(name) {
            self.active_policy = name.to_string();
            info!("💰 Switched to cost policy: {}", name);
            true
        } else {
            false
        }
    }

    /// Add or update a custom policy
    pub fn add_policy(&mut self, policy: CostPolicy) {
        self.policies.insert(policy.name.clone(), policy);
    }

    /// List available policy names
    pub fn list_policies(&self) -> Vec<String> {
        self.policies.keys().cloned().collect()
    }

    /// Evaluate the active policy for a chunk
    pub fn evaluate(
        &self,
        source_kind: &SourceKind,
        content_type: Option<&str>,
        language: Option<&str>,
        token_count: usize,
    ) -> IngestionTargets {
        self.active().evaluate(source_kind, content_type, language, token_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balanced_policy() {
        let policy = CostPolicy::balanced();
        
        // Code should go to both
        let targets = policy.evaluate(&SourceKind::CodeRepo, Some("text/x-rust"), Some("rust"), 100);
        assert!(targets.enable_vector);
        assert!(targets.enable_graph);
    }

    #[test]
    fn test_vector_first_policy() {
        let policy = CostPolicy::vector_first();
        
        // Code should go to vector only
        let targets = policy.evaluate(&SourceKind::CodeRepo, Some("text/x-rust"), Some("rust"), 100);
        assert!(targets.enable_vector);
        assert!(!targets.enable_graph);
    }

    #[test]
    fn test_economy_small_chunks() {
        let policy = CostPolicy::economy();
        
        // Small chunks should be skipped
        let targets = policy.evaluate(&SourceKind::Document, Some("text/plain"), None, 30);
        assert!(!targets.enable_vector);
        assert!(!targets.enable_graph);
    }

    #[test]
    fn test_policy_manager() {
        let mut manager = CostPolicyManager::new();
        assert_eq!(manager.active().name, "balanced");
        
        manager.set_active("economy");
        assert_eq!(manager.active().name, "economy");
    }
}
