pub mod auth;
pub mod billing;
pub mod data;
pub mod indexing;
pub mod security;
pub mod webhooks;
pub mod health;

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(auth::configure_auth_routes)
            .configure(billing::configure_billing_routes)
            .configure(data::configure_data_routes)
            .configure(indexing::configure_indexing_routes)
            .configure(security::configure_security_routes)
            .configure(webhooks::configure_webhook_routes)
            .configure(crate::graphql::configure_graphql_routes)
    )
    .route("/health", web::get().to(health::health_check))
    .route("/ready", web::get().to(health::readiness_check));
}
