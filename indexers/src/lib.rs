// Indexers library - provides indexing functionality without HTTP server
// This is a pure library that can be used by the backend service

// Define the api_bail macro that wraps anyhow::bail
#[macro_export]
macro_rules! api_bail {
    ($($arg:tt)*) => {
        anyhow::bail!($($arg)*)
    };
}

// Define the api_error macro that creates anyhow::Error
#[macro_export]
macro_rules! api_error {
    ($($arg:tt)*) => {
        anyhow::anyhow!($($arg)*)
    };
}

pub mod prelude;

pub mod base;
pub mod py;
pub use base::value;
pub use base::schema;
pub use base::spec;
pub mod setup;
pub mod builder;
pub mod config;
pub mod execution;
pub mod handlers;
pub mod models;
pub mod monitoring;
pub mod ops;
pub mod schema;
pub mod services;
pub mod utils;
pub mod utils_functions;

// Re-export main types for convenience
pub use config::IndexerConfig;
pub use services::code::CodeIndexingService;
pub use services::document::DocumentIndexingService;
pub use services::web::WebIndexingService;
pub use services::embedding::EmbeddingProcessor;

// Convenience function to create all indexers
pub fn create_indexers(config: IndexerConfig) -> (
    CodeIndexingService,
    DocumentIndexingService,
    WebIndexingService,
) {
    let code_indexer = CodeIndexingService::new(config.clone())
        .expect("Failed to create code indexer");
    let doc_indexer = DocumentIndexingService::new(config.clone())
        .expect("Failed to create document indexer");
    let web_indexer = WebIndexingService::new(config)
        .expect("Failed to create web indexer");

    (code_indexer, doc_indexer, web_indexer)
}
