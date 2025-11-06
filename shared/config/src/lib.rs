pub mod mcp_servers;
pub mod feature_toggles;

use feature_toggles::FeatureToggles;

use reqwest::Client;

#[derive(Clone)]
pub struct AppConfig {
    pub http_client: Client,
    pub langchain_url: String,
    pub haystack_url: String,
    pub unified_indexer_url: String,
    pub feature_toggles: FeatureToggles,
}

impl AppConfig {
    pub fn from_env() -> Self {
        // Load feature toggles from default path or env var
        let toggles = FeatureToggles::from_env_path();

        Self {
            http_client: Client::new(),
            langchain_url: std::env::var("LANGCHAIN_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3002".to_string()),
            haystack_url: std::env::var("HAYSTACK_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:8001".to_string()),
            unified_indexer_url: std::env::var("UNIFIED_INDEXER_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            feature_toggles: toggles,
        }
    }
}