use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureToggles {
    toggles: HashMap<String, bool>,
}

impl FeatureToggles {
    pub fn new() -> Self {
        Self {
            toggles: HashMap::new(),
        }
    }

    pub fn from_env_path() -> Self {
        let path = std::env::var("FEATURE_TOGGLES_PATH")
            .unwrap_or_else(|_| "../feature-toggles.json".to_string());
        
        Self::from_file(&path).unwrap_or_else(|_| Self::new())
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let toggles: HashMap<String, bool> = serde_json::from_str(&content)?;
        
        Ok(Self { toggles })
    }

    pub fn is_enabled(&self, feature: &str) -> bool {
        self.toggles.get(feature).copied().unwrap_or(false)
    }

    pub fn enable(&mut self, feature: &str) {
        self.toggles.insert(feature.to_string(), true);
    }

    pub fn disable(&mut self, feature: &str) {
        self.toggles.insert(feature.to_string(), false);
    }

    pub fn get_all(&self) -> &HashMap<String, bool> {
        &self.toggles
    }
}

impl Default for FeatureToggles {
    fn default() -> Self {
        Self::new()
    }
}
