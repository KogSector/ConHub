mod config;
mod handlers;
mod models;
mod services;

use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use config::AppConfig;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let config = AppConfig::from_env();
    let app_state = web::Data::new(config.clone());
    
    println!("Starting ConHub Backend on http://localhost:3001");
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec!["Content-Type", "Authorization"]);
            
        App::new()
            .app_data(app_state.clone())
            .wrap(cors)
            .wrap(Logger::default())
            .configure(handlers::configure_routes)
    })
    .bind("127.0.0.1:3001")?
    .run()
    .await
}