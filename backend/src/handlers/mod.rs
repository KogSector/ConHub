mod api;
mod health;
mod settings;

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .configure(health::configure)
            .configure(api::configure)
            .configure(settings::configure)
    );
}