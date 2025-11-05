pub mod embed;
pub mod rerank;
pub mod health;
pub mod disabled;

pub use embed::embed_handler;
pub use rerank::rerank_handler;
pub use health::health_handler;
pub use disabled::disabled_handler;
