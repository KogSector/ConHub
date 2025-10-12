// Organized service modules by concern
pub mod auth;           // All auth: users, sessions, password reset, OAuth, local auth
pub mod data;           // Data sources, repositories, connectors, VCS
pub mod ai;             // AI service, MCP client/server
pub mod security;       // Security service, tunnels, rulesets, rules
pub mod integrations;   // Social, webhooks
pub mod infrastructure; // Cache, vector DB, indexing triggers

// Core services
pub mod billing;
pub mod email_service;
pub mod search;
pub mod health;
pub mod orchestration;
pub mod feature_toggle_service;
