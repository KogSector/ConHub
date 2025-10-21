use conhub::config;
use conhub::handlers;
use conhub::models;
use conhub::services;
use conhub::utils;
use conhub::execution;
use conhub::monitoring;
use conhub::schema;
use actix_web::{web, App, HttpResponse, HttpServer, middleware::Logger};
use actix_cors::Cors;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

// Enhanced state with comprehensive indexing capabilities
pub struct IndexerState {
    pub code_indexer: Arc<services::code::CodeIndexingService>,
    pub doc_indexer: Arc<services::document::DocumentIndexingService>,
    pub web_indexer: Arc<services::web::WebIndexingService>,
    pub config: config::IndexerConfig,
    
    // Enhanced components
    pub error_handler: Arc<execution::error_handling::ErrorHandler>,
    pub metrics_collector: Arc<RwLock<monitoring::metrics::MetricsCollector>>,
    pub schema_manager: Arc<RwLock<schema::evolution::SchemaEvolutionManager>>,
    pub live_indexer: Option<Arc<services::live::LiveIndexingService>>,
    pub multi_format_processor: Arc<services::processing::MultiFormatProcessor>,
    pub embedding_processor: Arc<services::embedding::EmbeddingProcessor>,
}

async fn health() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "healthy",
        "service": "unified-indexer",
        "version": "1.0.0",
        "features": [
            "Code repository indexing",
            "Document indexing",
            "Web content indexing",
            "Full-text search",
            "Symbol cross-referencing",
            "Vector embeddings"
        ]
    })))
}

async fn index() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "message": "ConHub Unified Indexing Service",
        "version": "1.0.0",
        "endpoints": {
            "health": "/health",
            "code": "/api/index/code",
            "repository": "/api/index/repository",
            "documentation": "/api/index/documentation",
            "url": "/api/index/url",
            "file": "/api/index/file",
            "search": "/api/search",
            "status": "/api/status"
        }
    })))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    log::info!("Starting Unified Indexer Service");
    
    
    let config = config::IndexerConfig::from_env();
    let bind_address = format!("{}:{}", config.host, config.port);
    
    log::info!("Configuration loaded: {}", config);
    
    
    let code_indexer = Arc::new(
        services::code::CodeIndexingService::new(config.clone())
            .expect("Failed to initialize code indexer")
    );
    
    let doc_indexer = Arc::new(
        services::document::DocumentIndexingService::new(config.clone())
            .expect("Failed to initialize document indexer")
    );
    
    let web_indexer = Arc::new(
        services::web::WebIndexingService::new(config.clone())
            .expect("Failed to initialize web indexer")
    );
    
    let state = web::Data::new(IndexerState {
        code_indexer,
        doc_indexer,
        web_indexer,
        config: config.clone(),
    });
    
    log::info!("All indexing services initialized successfully");
    log::info!("Starting HTTP server on {}", bind_address);
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://localhost:3001")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec!["Content-Type", "Authorization"])
            .max_age(3600);
        
        App::new()
            .app_data(state.clone())
            .wrap(cors)
            .wrap(Logger::default())
            .route("/", web::get().to(index))
            .route("/health", web::get().to(health))
            .service(
                web::scope("/api")
                    .configure(handlers::configure_routes)
            )
    })
    .bind(&bind_address)?
    .run()
    .await
}
