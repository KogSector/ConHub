use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use serde_json;
use log::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeatureToggles {
    #[serde(rename = "Login")]
    pub login: bool,
    #[serde(rename = "Heavy", default)]
    pub heavy: bool,
}

impl Default for FeatureToggles {
    fn default() -> Self {
        Self {
            login: true, // Default to requiring login
            heavy: false, // Default to not being heavy
        }
    }
}

#[derive(Clone)]
pub struct FeatureToggleService {
    toggles: Arc<RwLock<FeatureToggles>>,
    config_path: String,
}

impl FeatureToggleService {
    pub fn new(config_path: &str) -> Self {
        Self {
            toggles: Arc::new(RwLock::new(FeatureToggles::default())),
            config_path: config_path.to_string(),
        }
    }
    
    pub async fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Load initial configuration
        self.reload_config().await
    }

    /// Load feature toggles from file
    pub async fn reload_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !Path::new(&self.config_path).exists() {
            warn!("Feature toggles file not found at {}, using defaults", self.config_path);
            return Ok(());
        }

        let content = fs::read_to_string(&self.config_path)?;
        let toggles: FeatureToggles = serde_json::from_str(&content)
            .map_err(|e| {
                error!("Failed to parse feature toggles: {}", e);
                e
            })?;

        // Only log if toggles actually changed
        let mut current_toggles = self.toggles.write().await;
        let changed = *current_toggles != toggles;
        *current_toggles = toggles.clone();
        
        if changed {
            info!("Feature toggles updated - Login: {}, Heavy: {}", toggles.login, toggles.heavy);
        }
        Ok(())
    }

    /// Save current feature toggles to file
    #[allow(dead_code)]
    pub async fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let toggles = self.toggles.read().await;
        let content = serde_json::to_string_pretty(&*toggles)?;
        fs::write(&self.config_path, content)?;
        
        info!("Feature toggles saved to {}", self.config_path);
        Ok(())
    }

    /// Check if login feature is enabled
    #[allow(dead_code)]
    pub async fn is_login_enabled(&self) -> bool {
        let toggles = self.toggles.read().await;
        toggles.login
    }

    /// Enable/disable login feature
    #[allow(dead_code)]
    pub async fn set_login_enabled(&self, enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut toggles = self.toggles.write().await;
            toggles.login = enabled;
        }
        self.save_config().await
    }

    /// Get all feature toggles
    #[allow(dead_code)]
    pub async fn get_all_toggles(&self) -> FeatureToggles {
        let toggles = self.toggles.read().await;
        toggles.clone()
    }

    /// Update multiple feature toggles
    #[allow(dead_code)]
    pub async fn update_toggles(&self, new_toggles: FeatureToggles) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut toggles = self.toggles.write().await;
            *toggles = new_toggles;
        }
        self.save_config().await
    }

    /// Check if authentication should be bypassed for a specific route
    #[allow(dead_code)]
    pub async fn should_bypass_auth(&self, _path: &str) -> bool {
        // If login is disabled, bypass auth for all routes except admin routes
        if !self.is_login_enabled().await {
            // Always require auth for admin/sensitive routes even when login is disabled
            // You can customize this logic based on your needs
            return true;
        }
        false
    }
}

// File watcher for hot-reloading feature toggles (optional)
pub async fn watch_feature_toggles(service: Arc<FeatureToggleService>) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // Check every 5 minutes
    
    loop {
        interval.tick().await;
        if let Err(e) = service.reload_config().await {
            error!("Failed to reload feature toggles: {}", e);
        }
    }
}