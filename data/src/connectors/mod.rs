pub mod traits;
pub mod types;
pub mod manager;
pub mod local_file;
pub mod github;
pub mod gitlab;
pub mod google_drive;
pub mod bitbucket;
pub mod url_scraper;
pub mod dropbox;
pub mod slack;
pub mod error;

pub use traits::Connector;
pub use types::*;
pub use manager::ConnectorManager;
pub use error::ConnectorError;
