pub mod health;
pub mod search;
pub mod ai;
pub mod ai_agents;
pub mod auth_service;
pub mod session_service;
pub mod feature_toggle_service;
pub mod rule_bank;
// Optimization services integrated into core services above
pub mod data_source_proxy;
pub mod orchestration;
pub mod social_integration_service;
pub mod platform_data_fetcher;
pub mod repository;
pub mod datasource;
pub mod vcs_detector;
pub mod vcs_connector;
pub mod mcp_server;
pub mod mcp_client;
pub mod github_copilot_integration;
pub mod repository_service;
pub mod legacy_repository;
// Legacy connectors - use sources module instead
// pub mod connectors;
// pub mod data_source_service;
pub mod indexing_orchestrator;
pub mod ai_service;
pub mod vector_db;
