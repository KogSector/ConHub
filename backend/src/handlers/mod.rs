mod api;
mod auth;
mod health;
mod settings;
mod data_source_handlers;
mod documents;
mod agents;
pub mod ai_agents;
mod urls;
mod repositories;
mod mcp;
mod github_copilot;
mod social;
mod data_sources;

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(auth::configure)
            .configure(data_source_handlers::configure)
            .configure(data_sources::configure)
            .configure(documents::configure)
            .configure(agents::configure_routes)
            .configure(ai_agents::configure_routes)
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