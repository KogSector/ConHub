use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

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

    // Convenience: read Auth enablement strictly from feature-toggles.json
    // Controls database connections (PostgreSQL, Qdrant, Redis) and auth/authorization
    pub fn auth_enabled(&self) -> bool {
        // Auth is now always enabled; the legacy "Auth" feature toggle is ignored.
        true
    }

    // Check if database connections should be established
    pub fn should_connect_databases(&self) -> bool {
        self.auth_enabled()
    }

    // Check if embedding service should be active (always enabled now)
    pub fn should_enable_embedding(&self) -> bool {
        true
    }

    // Check if indexing should be active (always enabled now)
    pub fn should_enable_indexing(&self) -> bool {
        true
    }

    // Convenience: read Docker enablement
    // Controls whether builds happen via Docker or locally
    pub fn docker_enabled(&self) -> bool {
        // Default to false for local development
        self.is_enabled_or("Docker", false)
    }

    // Check if Docker builds should be used
    pub fn should_use_docker(&self) -> bool {
        self.docker_enabled()
    }

    // Convenience: read Redis enablement
    // Controls whether Redis connections should be established for sessions/caching
    pub fn redis_enabled(&self) -> bool {
        // Default to true when Redis flag is missing
        self.is_enabled_or("Redis", true)
    }

    // Check if Redis connections should be established
    pub fn should_connect_redis(&self) -> bool {
        self.redis_enabled()
    }

    pub fn billing_enabled(&self) -> bool {
        true
    }

    pub fn should_use_billing(&self) -> bool {
        true
    }

    // Get all enabled features
    pub fn enabled_features(&self) -> Vec<String> {
        self.flags
            .iter()
            .filter(|(_, &enabled)| enabled)
            .map(|(name, _)| name.clone())
            .collect()
    }

    // Get all disabled features
    pub fn disabled_features(&self) -> Vec<String> {
        self.flags
            .iter()
            .filter(|(_, &enabled)| !enabled)
            .map(|(name, _)| name.clone())
            .collect()
    }
}

// Thread-safe cached feature toggles for hot reload support
lazy_static::lazy_static! {
    static ref CACHED_TOGGLES: Arc<RwLock<FeatureToggles>> = {
        Arc::new(RwLock::new(FeatureToggles::from_env_path()))
    };
}

// Get cached toggles (read-optimized)
pub fn get_cached_toggles() -> FeatureToggles {
    CACHED_TOGGLES.read().clone()
}

// Reload toggles from file (write operation)
pub fn reload_toggles() {
    let mut cache = CACHED_TOGGLES.write();
    *cache = FeatureToggles::from_env_path();
}
