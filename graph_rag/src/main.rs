use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber;
use std::env;
use std::sync::Arc;

mod models;
mod services;
mod handlers;
mod graph_db;
mod entity_resolution;
mod extractors;
mod errors;
mod knowledge_fusion;

use graph_db::Neo4jClient;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let db_url = env::var("DATABASE_URL_NEON")
        .expect("DATABASE_URL_NEON must be set");
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to database");
    
    // Initialize Neo4j client
    let neo4j_uri = env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let neo4j_user = env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let neo4j_password = env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "password".to_string());
    
    tracing::info!("🔷 [Graph Service] Connecting to Neo4j at {}...", neo4j_uri);
    let neo4j_client = match Neo4jClient::new(&neo4j_uri, &neo4j_user, &neo4j_password).await {
        Ok(client) => {
            tracing::info!("✅ [Graph Service] Neo4j connection established");
            Some(Arc::new(client))
        }
        Err(e) => {
            tracing::error!("Failed to connect to Neo4j: {}", e);
            tracing::warn!("Graph operations will be limited");
            None
        }
    };
    
    let port = env::var("GRAPH_PORT")
        .unwrap_or_else(|_| "8006".to_string())
        .parse::<u16>()
        .expect("Invalid port number");
    
    tracing::info!("🚀 [Graph Service] Starting on port {}", port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();
        
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(neo4j_client.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .service(
                web::scope("/api/graph")
                    // Entity endpoints
                    .route("/entities/{id}", web::get().to(handlers::get_entity))
                    .route("/entities", web::post().to(handlers::create_entity))
                    
                    // Graph query endpoints
                    .route("/entities/{id}/neighbors", web::get().to(handlers::get_neighbors))
                    .route("/paths", web::get().to(handlers::find_paths))
                    .route("/traverse", web::post().to(handlers::traverse_graph))
                    
                    // Unified query endpoints
                    .route("/query", web::post().to(handlers::unified_query))
                    .route("/cross_source", web::post().to(handlers::cross_source_query))
                    .route("/semantic_search", web::post().to(handlers::semantic_search))
                    
                    // Statistics
                    .route("/statistics", web::get().to(handlers::get_statistics))
            )
            // Chunk ingestion (from chunker service)
            .route("/graph/chunks", web::post().to(handlers::ingest_chunks))
            .route("/health", web::get().to(|| async { 
                actix_web::HttpResponse::Ok().json(serde_json::json!({
                    "status": "healthy",
                    "service": "graph"
                }))
            }))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
