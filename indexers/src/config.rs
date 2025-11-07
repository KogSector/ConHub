use std::fmt;

#[derive(Debug, Clone)]
pub struct IndexerConfig {
    pub host: String,
    pub port: u16,
    pub backend_url: String,
    pub backend_graphql_url: String,
    pub qdrant_url: Option<String>,
    pub qdrant_api_key: Option<String>,
    pub embedding_service_url: String,
    pub openai_api_key: Option<String>,
    pub max_file_size: usize,
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub max_concurrent_indexing: usize,
    pub index_data_path: String,
    pub temp_dir: String,
}

impl IndexerConfig {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();
        
        Self {
            host: std::env::var("INDEXER_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("INDEXER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            backend_url: std::env::var("BACKEND_URL")
                .unwrap_or_else(|_| "http://localhost:3001".to_string()),
            backend_graphql_url: std::env::var("BACKEND_GRAPHQL_URL")
                .unwrap_or_else(|_| "http://localhost:8000/api/graphql".to_string()),
            qdrant_url: std::env::var("QDRANT_URL").ok(),
            qdrant_api_key: std::env::var("QDRANT_API_KEY").ok(),
            embedding_service_url: std::env::var("EMBEDDING_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:8082".to_string()),
            openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
            max_file_size: std::env::var("MAX_FILE_SIZE")
                .unwrap_or_else(|_| "10485760".to_string()) 
                .parse()
                .unwrap_or(10485760),
            chunk_size: std::env::var("CHUNK_SIZE")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            chunk_overlap: std::env::var("CHUNK_OVERLAP")
                .unwrap_or_else(|_| "200".to_string())
                .parse()
                .unwrap_or(200),
            max_concurrent_indexing: std::env::var("MAX_CONCURRENT_INDEXING")
                .unwrap_or_else(|_| "4".to_string())
                .parse()
                .unwrap_or(4),
            index_data_path: std::env::var("INDEX_DATA_PATH")
                .unwrap_or_else(|_| "./data/index".to_string()),
            temp_dir: std::env::var("TEMP_DIR")
                .unwrap_or_else(|_| "./data/temp".to_string()),
        }
    }
}

impl fmt::Display for IndexerConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "IndexerConfig {{ host: {}, port: {}, backend_url: {}, embedding_service_url: {}, qdrant_enabled: {}, openai_enabled: {} }}",
            self.host,
            self.port,
            self.backend_url,
            self.embedding_service_url,
            self.qdrant_url.is_some(),
            self.openai_api_key.is_some()
        )
    }
}
