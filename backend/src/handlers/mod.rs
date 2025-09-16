mod api;
mod health;
mod settings;
mod urls;
mod documents;
mod agents;

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .configure(health::configure)
            .configure(api::configure)
            .configure(settings::configure)
            .configure(urls::configure)
            .configure(documents::configure)
            .configure(agents::configure)
    );
}