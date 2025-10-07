use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use std::env;

mod models;
mod handlers;
mod services;
mod middleware;
mod errors;

use handlers::auth::configure_auth_routes;
use handlers::billing::configure_billing_routes;
mod rulesets {
    pub use crate::handlers::rulesets::configure;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv::dotenv().ok();

    let port = env::var("BACKEND_PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()
        .unwrap_or(3001);

    println!("🚀 Starting ConHub Backend on port {}", port);

    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .configure(configure_auth_routes)
            .configure(configure_billing_routes)
            .configure(rulesets::configure)
            .route("/health", web::get().to(health_check))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

async fn health_check() -> actix_web::Result<web::Json<serde_json::Value>> {
    Ok(web::Json(serde_json::json!({
        "status": "healthy",
        "service": "conhub-backend"
    })))
}