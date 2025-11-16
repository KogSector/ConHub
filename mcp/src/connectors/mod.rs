// Connector Architecture - Unified interface for all data sources
pub mod trait_def;
pub mod manager;
pub mod github;
pub mod gitlab;
pub mod bitbucket;
pub mod local_fs;
pub mod google_drive;
pub mod dropbox;
pub mod notion;

pub use trait_def::Connector;
pub use manager::ConnectorManager;
