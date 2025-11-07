use std::env;
use conhub_config::feature_toggles::FeatureToggles;

#[derive(Clone, Debug)]
pub struct AppConfig {
    // Server
    pub backend_port: u16,
    pub env_mode: String,

    // Database
    pub database_url: Option<String>,
    pub redis_url: Option<String>,
    pub qdrant_url: String,
    // Microservices
    pub embedding_service_url: String,
    // Microservice call settings
    pub embedding_request_timeout_ms: u64,
    pub embedding_request_retries: usize,
    pub embedding_max_inflight: usize,

    // Authentication
    pub jwt_secret: String,
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub github_client_id: Option<String>,
    pub github_client_secret: Option<String>,
    pub microsoft_client_id: Option<String>,
    pub microsoft_client_secret: Option<String>,

    // Email
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,

    // Billing
    pub stripe_secret_key: Option<String>,
    pub stripe_webhook_secret: Option<String>,

    // AI
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,

    // Data sources
    pub github_token: Option<String>,
    pub gitlab_token: Option<String>,
    pub notion_token: Option<String>,

    // Webhooks
    pub github_webhook_secret: Option<String>,
    pub gitlab_webhook_secret: Option<String>,

    // Indexing configuration
    pub max_file_size: usize,
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub max_concurrent_indexing: usize,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();

        // Read feature toggles to determine whether Auth is enabled
        let toggles = FeatureToggles::from_env_path();
        let auth_enabled = toggles.auth_enabled();

        Self {
            // Server
            backend_port: env::var("BACKEND_PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()
                .expect("BACKEND_PORT must be a valid port number"),
            env_mode: env::var("ENV_MODE").unwrap_or_else(|_| "development".to_string()),

            // Database
            // Make DB URLs optional to allow full startup when Auth is disabled
            database_url: env::var("DATABASE_URL").ok(),
            redis_url: env::var("REDIS_URL").ok(),
            qdrant_url: env::var("QDRANT_URL")
                .unwrap_or_else(|_| "http://localhost:6333".to_string()),
            embedding_service_url: env::var("EMBEDDING_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:8082".to_string()),
            embedding_request_timeout_ms: env::var("EMBEDDING_REQUEST_TIMEOUT_MS")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .unwrap_or(5000),
            embedding_request_retries: env::var("EMBEDDING_REQUEST_RETRIES")
                .unwrap_or_else(|_| "2".to_string())
                .parse()
                .unwrap_or(2),
            embedding_max_inflight: env::var("EMBEDDING_MAX_INFLIGHT")
                .unwrap_or_else(|_| "64".to_string())
                .parse()
                .unwrap_or(64),

            // Authentication
            // Require JWT_SECRET only when Auth is enabled; otherwise use a stub to allow startup.
            jwt_secret: if auth_enabled {
                env::var("JWT_SECRET").expect("JWT_SECRET must be set when Auth is enabled")
            } else {
                env::var("JWT_SECRET").unwrap_or_else(|_| "dev-no-auth".to_string())
            },
            google_client_id: env::var("GOOGLE_CLIENT_ID").ok(),
            google_client_secret: env::var("GOOGLE_CLIENT_SECRET").ok(),
            github_client_id: env::var("GITHUB_CLIENT_ID").ok(),
            github_client_secret: env::var("GITHUB_CLIENT_SECRET").ok(),
            microsoft_client_id: env::var("MICROSOFT_CLIENT_ID").ok(),
            microsoft_client_secret: env::var("MICROSOFT_CLIENT_SECRET").ok(),

            // Email
            smtp_host: env::var("SMTP_HOST").ok(),
            smtp_port: env::var("SMTP_PORT").ok().and_then(|p| p.parse().ok()),
            smtp_username: env::var("SMTP_USERNAME").ok(),
            smtp_password: env::var("SMTP_PASSWORD").ok(),

            // Billing
            stripe_secret_key: env::var("STRIPE_SECRET_KEY").ok(),
            stripe_webhook_secret: env::var("STRIPE_WEBHOOK_SECRET").ok(),

            // AI
            openai_api_key: env::var("OPENAI_API_KEY").ok(),
            anthropic_api_key: env::var("ANTHROPIC_API_KEY").ok(),

            // Data sources
            github_token: env::var("GITHUB_TOKEN").ok(),
            gitlab_token: env::var("GITLAB_TOKEN").ok(),
            notion_token: env::var("NOTION_TOKEN").ok(),

            // Webhooks
            github_webhook_secret: env::var("GITHUB_WEBHOOK_SECRET").ok(),
            gitlab_webhook_secret: env::var("GITLAB_WEBHOOK_SECRET").ok(),

            // Indexing configuration
            max_file_size: env::var("MAX_FILE_SIZE")
                .unwrap_or_else(|_| "10485760".to_string())
                .parse()
                .unwrap_or(10485760),
            chunk_size: env::var("CHUNK_SIZE")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            chunk_overlap: env::var("CHUNK_OVERLAP")
                .unwrap_or_else(|_| "200".to_string())
                .parse()
                .unwrap_or(200),
            max_concurrent_indexing: env::var("MAX_CONCURRENT_INDEXING")
                .unwrap_or_else(|_| "4".to_string())
                .parse()
                .unwrap_or(4),
        }
    }
}
