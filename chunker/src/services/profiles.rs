//! Chunker Profiles
//!
//! Configurable chunking profiles that control chunk size, overlap, and strategy selection
//! based on source kind, content type, and other metadata.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use conhub_models::chunking::SourceKind;

/// Chunk size configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkSizeConfig {
    /// Minimum tokens per chunk (soft limit)
    pub min_tokens: usize,
    /// Maximum tokens per chunk (hard limit)
    pub max_tokens: usize,
    /// Token overlap between chunks
    pub overlap_tokens: usize,
}

impl Default for ChunkSizeConfig {
    fn default() -> Self {
        Self {
            min_tokens: 100,
            max_tokens: 1024,
            overlap_tokens: 64,
        }
    }
}

/// Strategy configuration for a specific source kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    /// Primary chunking strategy
    pub strategy: ChunkingStrategy,
    /// Fallback strategy if primary fails
    pub fallback: Option<ChunkingStrategy>,
    /// Size configuration for this strategy
    pub size_config: ChunkSizeConfig,
    /// Whether to enable AST-based chunking for code (if available)
    pub use_ast: bool,
    /// Whether to preserve code structure (functions, classes)
    pub preserve_structure: bool,
    /// Whether to include metadata (headings, context) in chunks
    pub include_context: bool,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            strategy: ChunkingStrategy::Text,
            fallback: None,
            size_config: ChunkSizeConfig::default(),
            use_ast: true,
            preserve_structure: true,
            include_context: true,
        }
    }
}

/// Available chunking strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChunkingStrategy {
    /// Simple text-based chunking with sentence boundaries
    Text,
    /// AST-based code chunking (function/class boundaries)
    AstCode,
    /// Regex-based code chunking (fallback for unsupported languages)
    Code,
    /// Markdown heading-based chunking
    Markdown,
    /// Chat/conversation window chunking
    Chat,
    /// Issue/PR/ticket chunking
    Ticketing,
    /// HTML structure-aware chunking
    Html,
    /// Passthrough (no chunking, for small documents)
    Passthrough,
}

impl ChunkingStrategy {
    pub fn from_source_kind(kind: &SourceKind, content_type: Option<&str>) -> Self {
        match kind {
            SourceKind::CodeRepo => ChunkingStrategy::AstCode,
            SourceKind::Document => {
                if let Some(ct) = content_type {
                    if ct.contains("markdown") || ct.contains("text/x-markdown") {
                        ChunkingStrategy::Markdown
                    } else if ct.contains("html") {
                        ChunkingStrategy::Html
                    } else {
                        ChunkingStrategy::Text
                    }
                } else {
                    ChunkingStrategy::Text
                }
            }
            SourceKind::Chat => ChunkingStrategy::Chat,
            SourceKind::Ticketing => ChunkingStrategy::Ticketing,
            SourceKind::Wiki => ChunkingStrategy::Markdown,
            SourceKind::Email => ChunkingStrategy::Text,
            SourceKind::Other => ChunkingStrategy::Text,
        }
    }
}

/// A complete chunker profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkerProfile {
    /// Profile name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Strategy configuration per source kind
    #[serde(default)]
    pub strategies: HashMap<String, StrategyConfig>,
    /// Default strategy if no match found
    #[serde(default)]
    pub default_strategy: StrategyConfig,
    /// Language-specific overrides for code chunking
    #[serde(default)]
    pub language_overrides: HashMap<String, ChunkSizeConfig>,
}

impl Default for ChunkerProfile {
    fn default() -> Self {
        Self::standard()
    }
}

impl ChunkerProfile {
    /// Standard profile for balanced chunking
    pub fn standard() -> Self {
        let mut strategies = HashMap::new();

        // Code repositories: AST-based with structure preservation
        strategies.insert(
            "code_repo".to_string(),
            StrategyConfig {
                strategy: ChunkingStrategy::AstCode,
                fallback: Some(ChunkingStrategy::Code),
                size_config: ChunkSizeConfig {
                    min_tokens: 50,
                    max_tokens: 1024,
                    overlap_tokens: 32,
                },
                use_ast: true,
                preserve_structure: true,
                include_context: true,
            },
        );

        // Documents: Markdown/heading-aware
        strategies.insert(
            "document".to_string(),
            StrategyConfig {
                strategy: ChunkingStrategy::Markdown,
                fallback: Some(ChunkingStrategy::Text),
                size_config: ChunkSizeConfig {
                    min_tokens: 100,
                    max_tokens: 2048,
                    overlap_tokens: 128,
                },
                use_ast: false,
                preserve_structure: true,
                include_context: true,
            },
        );

        // Chat: Conversation window chunking
        strategies.insert(
            "chat".to_string(),
            StrategyConfig {
                strategy: ChunkingStrategy::Chat,
                fallback: None,
                size_config: ChunkSizeConfig {
                    min_tokens: 50,
                    max_tokens: 512,
                    overlap_tokens: 64,
                },
                use_ast: false,
                preserve_structure: false,
                include_context: true,
            },
        );

        // Ticketing: Issue/PR structure
        strategies.insert(
            "ticketing".to_string(),
            StrategyConfig {
                strategy: ChunkingStrategy::Ticketing,
                fallback: Some(ChunkingStrategy::Text),
                size_config: ChunkSizeConfig {
                    min_tokens: 100,
                    max_tokens: 2048,
                    overlap_tokens: 64,
                },
                use_ast: false,
                preserve_structure: true,
                include_context: true,
            },
        );

        // Web: HTML-aware
        strategies.insert(
            "web".to_string(),
            StrategyConfig {
                strategy: ChunkingStrategy::Html,
                fallback: Some(ChunkingStrategy::Text),
                size_config: ChunkSizeConfig {
                    min_tokens: 100,
                    max_tokens: 1536,
                    overlap_tokens: 96,
                },
                use_ast: false,
                preserve_structure: true,
                include_context: true,
            },
        );

        Self {
            name: "standard".to_string(),
            description: Some("Standard balanced chunking profile".to_string()),
            strategies,
            default_strategy: StrategyConfig::default(),
            language_overrides: HashMap::new(),
        }
    }

    /// High-quality profile for comprehensive indexing (larger chunks, more overlap)
    pub fn high_quality() -> Self {
        let mut profile = Self::standard();
        profile.name = "high_quality".to_string();
        profile.description = Some("High-quality chunking with larger chunks and more context".to_string());

        // Increase sizes and overlap
        for (_, config) in profile.strategies.iter_mut() {
            config.size_config.max_tokens = (config.size_config.max_tokens as f64 * 1.5) as usize;
            config.size_config.overlap_tokens = (config.size_config.overlap_tokens as f64 * 1.5) as usize;
        }

        profile
    }

    /// Fast profile for quick indexing (smaller chunks, less overlap)
    pub fn fast() -> Self {
        let mut profile = Self::standard();
        profile.name = "fast".to_string();
        profile.description = Some("Fast chunking with smaller chunks".to_string());

        // Decrease sizes and overlap
        for (_, config) in profile.strategies.iter_mut() {
            config.size_config.max_tokens = (config.size_config.max_tokens as f64 * 0.6) as usize;
            config.size_config.overlap_tokens = (config.size_config.overlap_tokens as f64 * 0.5) as usize;
        }

        profile
    }

    /// Get strategy config for a source kind
    pub fn get_strategy(&self, source_kind: &SourceKind) -> &StrategyConfig {
        let key = match source_kind {
            SourceKind::CodeRepo => "code_repo",
            SourceKind::Document => "document",
            SourceKind::Chat => "chat",
            SourceKind::Ticketing => "ticketing",
            SourceKind::Wiki => "document",
            SourceKind::Email => "document",
            SourceKind::Other => "document",
        };

        self.strategies.get(key).unwrap_or(&self.default_strategy)
    }

    /// Get chunk size config for a language (with fallback to source kind default)
    pub fn get_size_config(&self, source_kind: &SourceKind, language: Option<&str>) -> ChunkSizeConfig {
        if let Some(lang) = language {
            if let Some(override_config) = self.language_overrides.get(lang) {
                return override_config.clone();
            }
        }
        self.get_strategy(source_kind).size_config.clone()
    }
}

/// Profile manager for loading and caching profiles
pub struct ProfileManager {
    profiles: HashMap<String, ChunkerProfile>,
    active_profile: String,
}

impl Default for ProfileManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileManager {
    pub fn new() -> Self {
        let mut profiles = HashMap::new();
        profiles.insert("standard".to_string(), ChunkerProfile::standard());
        profiles.insert("high_quality".to_string(), ChunkerProfile::high_quality());
        profiles.insert("fast".to_string(), ChunkerProfile::fast());

        Self {
            profiles,
            active_profile: "standard".to_string(),
        }
    }

    /// Get the active profile
    pub fn active(&self) -> &ChunkerProfile {
        self.profiles.get(&self.active_profile)
            .unwrap_or_else(|| self.profiles.get("standard").unwrap())
    }

    /// Set active profile by name
    pub fn set_active(&mut self, name: &str) -> bool {
        if self.profiles.contains_key(name) {
            self.active_profile = name.to_string();
            true
        } else {
            false
        }
    }

    /// Add or update a custom profile
    pub fn add_profile(&mut self, profile: ChunkerProfile) {
        self.profiles.insert(profile.name.clone(), profile);
    }

    /// Get a profile by name
    pub fn get_profile(&self, name: &str) -> Option<&ChunkerProfile> {
        self.profiles.get(name)
    }

    /// List available profile names
    pub fn list_profiles(&self) -> Vec<String> {
        self.profiles.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_profile() {
        let profile = ChunkerProfile::standard();
        assert_eq!(profile.name, "standard");
        assert!(profile.strategies.contains_key("code_repo"));
        assert!(profile.strategies.contains_key("document"));
    }

    #[test]
    fn test_strategy_selection() {
        let profile = ChunkerProfile::standard();
        
        let code_config = profile.get_strategy(&SourceKind::CodeRepo);
        assert_eq!(code_config.strategy, ChunkingStrategy::AstCode);
        
        let doc_config = profile.get_strategy(&SourceKind::Document);
        assert_eq!(doc_config.strategy, ChunkingStrategy::Markdown);
    }

    #[test]
    fn test_profile_manager() {
        let mut manager = ProfileManager::new();
        assert_eq!(manager.active().name, "standard");
        
        manager.set_active("fast");
        assert_eq!(manager.active().name, "fast");
    }
}
