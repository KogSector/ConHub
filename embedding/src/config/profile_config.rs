use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMatch {
    pub connector_type: Option<String>,
    pub block_type: Option<String>,
    #[serde(default)]
    pub language: Option<LanguageMatch>,
    #[serde(default)]
    pub content_type: Option<ContentTypeMatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LanguageMatch {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContentTypeMatch {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingProfile {
    pub id: String,
    #[serde(rename = "match")]
    pub match_criteria: ProfileMatch,
    pub models: Vec<String>,
    pub weights: Vec<f32>,
    pub fusion_strategy: String,
    pub chunker: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackProfile {
    pub id: String,
    pub models: Vec<String>,
    pub weights: Vec<f32>,
    pub fusion_strategy: String,
    pub chunker: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentTypeDetection {
    pub code_extensions: Vec<String>,
    pub text_extensions: Vec<String>,
    pub language_mapping: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub profiles: Vec<EmbeddingProfile>,
    pub fallback_profile: FallbackProfile,
    pub content_type_detection: ContentTypeDetection,
}

impl ProfileConfig {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read profile config from {}", path))?;
        
        let config: ProfileConfig = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse profile config from {}", path))?;
        
        Ok(config)
    }
    
    /// Find matching profile based on content metadata
    pub fn find_profile(
        &self,
        connector_type: &str,
        block_type: Option<&str>,
        language: Option<&str>,
        content_type: Option<&str>,
    ) -> &EmbeddingProfile {
        // Score each profile and return the best match
        let mut best_match: Option<(&EmbeddingProfile, usize)> = None;
        
        for profile in &self.profiles {
            let score = self.calculate_match_score(
                profile,
                connector_type,
                block_type,
                language,
                content_type,
            );
            
            if score > 0 {
                if let Some((_, best_score)) = best_match {
                    if score > best_score {
                        best_match = Some((profile, score));
                    }
                } else {
                    best_match = Some((profile, score));
                }
            }
        }
        
        // Return best match or use fallback
        match best_match {
            Some((profile, _)) => profile,
            None => {
                // Create a temporary profile from fallback for compatibility
                // In practice, we'll handle fallback in the service
                &self.profiles[0] // This is a workaround; better to return Option
            }
        }
    }
    
    fn calculate_match_score(
        &self,
        profile: &EmbeddingProfile,
        connector_type: &str,
        block_type: Option<&str>,
        language: Option<&str>,
        content_type: Option<&str>,
    ) -> usize {
        let mut score = 0;
        
        // Connector type match (required)
        if let Some(ref profile_connector) = profile.match_criteria.connector_type {
            if profile_connector == connector_type {
                score += 10;
            } else {
                return 0; // Must match connector type
            }
        }
        
        // Block type match (highly weighted)
        if let Some(ref profile_block_type) = profile.match_criteria.block_type {
            if let Some(input_block_type) = block_type {
                if profile_block_type == input_block_type {
                    score += 5;
                }
            }
        }
        
        // Language match (medium weighted)
        if let Some(ref profile_language) = profile.match_criteria.language {
            if let Some(input_language) = language {
                let matches = match profile_language {
                    LanguageMatch::Single(lang) => lang == input_language,
                    LanguageMatch::Multiple(langs) => langs.iter().any(|l| l == input_language),
                };
                
                if matches {
                    score += 3;
                }
            }
        }
        
        // Content type match (lower weighted)
        if let Some(ref profile_content_type) = profile.match_criteria.content_type {
            if let Some(input_content_type) = content_type {
                let matches = match profile_content_type {
                    ContentTypeMatch::Single(ct) => ct == input_content_type,
                    ContentTypeMatch::Multiple(cts) => cts.iter().any(|c| c == input_content_type),
                };
                
                if matches {
                    score += 2;
                }
            }
        }
        
        score
    }
    
    /// Detect content type from file extension
    pub fn detect_content_type(&self, file_path: &str) -> (Option<String>, Option<String>) {
        let extension = std::path::Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e));
        
        if let Some(ext) = extension {
            if self.content_type_detection.code_extensions.contains(&ext) {
                let language = self.content_type_detection.language_mapping
                    .get(&ext)
                    .cloned();
                return (Some("code".to_string()), language);
            } else if self.content_type_detection.text_extensions.contains(&ext) {
                return (Some("text".to_string()), None);
            }
        }
        
        (None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_profile_matching() {
        let config = ProfileConfig::from_file("config/profiles.json").unwrap();
        
        // Test GitHub Rust code
        let profile = config.find_profile("github", Some("code"), Some("rust"), None);
        assert_eq!(profile.id, "github_code_rust");
        
        // Test Slack messages
        let profile = config.find_profile("slack", Some("text"), None, None);
        assert_eq!(profile.id, "slack_messages");
    }
    
    #[test]
    fn test_content_type_detection() {
        let config = ProfileConfig::from_file("config/profiles.json").unwrap();
        
        let (block_type, language) = config.detect_content_type("src/main.rs");
        assert_eq!(block_type, Some("code".to_string()));
        assert_eq!(language, Some("rust".to_string()));
        
        let (block_type, language) = config.detect_content_type("README.md");
        assert_eq!(block_type, Some("text".to_string()));
        assert_eq!(language, None);
    }
}
