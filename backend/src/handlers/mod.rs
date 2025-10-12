// Organized handler modules by concern
pub mod auth;      // Authentication & OAuth
pub mod data;      // Data sources, repos, docs, URLs, indexing
pub mod ai;        // AI agents, MCP, GitHub Copilot
pub mod security;  // Rulesets, rule bank
pub mod billing;   // Billing & subscriptions
pub mod system;    // Health, settings, social

// Legacy handlers (to be refactored)
pub mod api;
