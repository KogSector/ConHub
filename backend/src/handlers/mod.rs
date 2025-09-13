mod api;
mod health;

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .configure(health::configure)
            .configure(api::configure)
    );
}