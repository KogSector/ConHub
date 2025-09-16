mod api;
mod health;
mod settings;
mod data_source_handlers;
mod documents;
mod agents;
mod urls;
mod repositories;

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(data_source_handlers::configure)
            .configure(documents::configure)
            .configure(agents::configure)
            .configure(repositories::configure_routes)
    ).service(
        web::scope("")
            .configure(health::configure)
            .configure(api::configure)
            .configure(settings::configure)
    );
}