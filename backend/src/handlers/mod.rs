mod api;
mod auth;
mod health;
mod settings;
mod data_source_handlers;
mod documents;
mod agents;
mod urls;
mod repositories;
mod mcp;
mod github_copilot;
mod social;
// mod rule_bank;

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // cfg.configure(rule_bank::configure_rule_bank_routes);
    cfg.service(
        web::scope("/api")
            .configure(auth::configure)
            .configure(data_source_handlers::configure)
            .configure(documents::configure)
            .configure(agents::configure)
            .configure(repositories::configure_routes)
            .configure(mcp::configure)
            .configure(github_copilot::configure_copilot_routes)
            .configure(social::configure)
    ).service(
        web::scope("")
            .configure(health::configure)
            .configure(api::configure)
            .configure(settings::configure)
    );
}