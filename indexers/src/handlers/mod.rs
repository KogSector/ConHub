pub mod indexing;
pub mod search;
pub mod status;

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/index")
            .route("/repository", web::post().to(indexing::index_repository))
            .route("/documentation", web::post().to(indexing::index_documentation))
            .route("/url", web::post().to(indexing::index_url))
            .route("/file", web::post().to(indexing::index_file))
            .route("/code", web::post().to(indexing::index_code))
    )
    .service(
        web::scope("/search")
            .route("", web::post().to(search::search))
            .route("/code", web::post().to(search::search_code))
            .route("/fusion", web::post().to(search::fusion_search))
    )
    .service(
        web::scope("/status")
            .route("", web::get().to(status::get_status))
            .route("/{job_id}", web::get().to(status::get_job_status))
    );
}
