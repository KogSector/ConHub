use actix_web::web;
use crate::handlers;

pub fn configure_rag_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/rag")
            .route("/query", web::post().to(handlers::rag_query))
            .route("/vector", web::post().to(handlers::rag_vector))
            .route("/hybrid", web::post().to(handlers::rag_hybrid))
            .route("/agentic", web::post().to(handlers::rag_agentic))
    );
}
