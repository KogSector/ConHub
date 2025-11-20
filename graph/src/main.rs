use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber;

mod models;
mod services;
mod handlers;
mod graph_db;
mod entity_resolution;
mod extractors;
mod errors;
mod knowledge_fusion;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL_NEON")
        .expect("DATABASE_URL_NEON must be set");
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to database");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(web::scope("/api/graph"))
    })
    .bind(("0.0.0.0", 8006))?
    .run()
    .await
}
