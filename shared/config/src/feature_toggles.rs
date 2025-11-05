use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Default)]
pub struct FeatureToggles {
    #[serde(flatten)]
    pub flags: HashMap<String, bool>,
}

impl FeatureToggles {
    // Load from a provided path or env var FEATURE_TOGGLES_PATH, defaulting to ./feature-toggles.json
    pub fn from_path(path: Option<String>) -> Self {
        let default_path = std::env::var("FEATURE_TOGGLES_PATH")
            .unwrap_or_else(|_| "feature-toggles.json".to_string());
        let path = path.unwrap_or(default_path);

        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => FeatureToggles::default(),
        }
    }

    pub fn from_env_path() -> Self {
        Self::from_path(None)
    }

    pub fn is_enabled(&self, name: &str) -> bool {
        self.flags.get(name).copied().unwrap_or(false)
    }

    pub fn is_enabled_or(&self, name: &str, default: bool) -> bool {
        self.flags.get(name).copied().unwrap_or(default)
    }
}