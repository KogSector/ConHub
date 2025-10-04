mod config;
mod indexer;
mod search_simple;
mod parser;
mod analyzer;
mod xref;
mod history;
mod web;
mod types;
mod utils;
mod enhanced_search;
mod ai_integration;
mod performance;

use search_simple as search;

use actix_web::{web::{Data, get, post, scope}, App, HttpResponse, HttpServer, Result, middleware::Logger};
use actix_cors::Cors;
use serde_json::json;
use config::LexorConfig;
use std::sync::Arc;
use tantivy::{Index, schema::Schema};


pub struct LexorService {
    pub config: LexorConfig,
    pub indexer: indexer::IndexerEngine,
    pub search_engine: search::SearchEngine,
    pub enhanced_search: enhanced_search::EnhancedSearchEngine,
    pub ai_context: ai_integration::AIContextEngine,
    pub performance: performance::PerformanceOptimizer,
}

impl LexorService {
    pub fn new(config: LexorConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Create a simple in-memory index for now
        let schema = create_default_schema();
        let index = Index::create_in_ram(schema.clone());
        
        let indexer = indexer::IndexerEngine::new(config.clone())?;
        let search_engine = search::SearchEngine::new(index.clone(), schema.clone())?;
        let enhanced_search = enhanced_search::EnhancedSearchEngine::new(index.clone(), schema.clone())?;
        let ai_context = ai_integration::AIContextEngine::new();
        let performance = performance::PerformanceOptimizer::new();
        
        Ok(Self {
            config,
            indexer,
            search_engine,
            enhanced_search,
            ai_context,
            performance,
        })
    }
}

fn create_default_schema() -> Schema {
    use tantivy::schema::*;
    
    let mut schema_builder = Schema::builder();
    
    schema_builder.add_text_field("file_id", TEXT | STORED);
    schema_builder.add_text_field("project_id", TEXT | STORED);
    schema_builder.add_text_field("file_path", TEXT | STORED);
    schema_builder.add_text_field("file_name", TEXT | STORED);
    schema_builder.add_text_field("content", TEXT);
    schema_builder.add_text_field("language", TEXT | STORED);
    schema_builder.add_text_field("file_type", TEXT | STORED);
    schema_builder.add_u64_field("file_size", STORED);
    schema_builder.add_u64_field("line_count", STORED);
    schema_builder.add_text_field("symbol_name", TEXT | STORED);
    schema_builder.add_text_field("symbol_type", TEXT | STORED);
    schema_builder.add_u64_field("symbol_line", STORED);
    schema_builder.add_text_field("reference_type", TEXT | STORED);
    schema_builder.add_text_field("commit_message", TEXT);
    schema_builder.add_text_field("author", TEXT | STORED);
    schema_builder.add_text_field("project_name", TEXT | STORED);
    
    schema_builder.build()
}

async fn index() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "message": "Lexor - Code Indexing and Search Service",
        "version": "1.0.0",
        "features": [
            "Source code indexing",
            "Full-text search",
            "Symbol cross-referencing",
            "Git history analysis",
            "Multi-language support"
        ]
    })))
}

async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "ok",
        "service": "Lexor"
    })))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    println!("Starting Lexor service on http://localhost:3002");
    
    // Initialize Lexor
    let config = LexorConfig::default();
    let lexor_service = Arc::new(
        LexorService::new(config)
            .expect("Failed to initialize Lexor service")
    );
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec!["Content-Type", "Authorization"]);
            
        App::new()
            .app_data(Data::new(lexor_service.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .route("/", get().to(index))
            .route("/health", get().to(health))
            .service(
                scope("/api")
                    .route("/search", post().to(web::search_handler))
                    .route("/projects", get().to(web::get_projects_handler))
                    .route("/projects", post().to(web::add_project_handler))
                    .route("/projects/{id}/index", post().to(web::index_project_handler))
                    .route("/stats", get().to(web::get_stats_handler))
            )
            .service(
                scope("/index")
                    .route("/repository", post().to(web::index_repository_handler))
            )
    })
    .bind("127.0.0.1:3002")?
    .run()
    .await
}