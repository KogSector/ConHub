use actix_web::{web, App, HttpServer, middleware::Logger, Result as ActixResult};
use actix_cors::Cors;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};

mod handlers;
mod services;

use conhub_plugins::{
    registry::PluginRegistry,
    config::{PluginConfigManager, PluginInstanceConfig},
    PluginType,
};

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<RwLock<PluginRegistry>>,
    pub config_manager: Arc<RwLock<PluginConfigManager>>,
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    env_logger::init();

    // Load environment variables
    dotenv::dotenv().ok();

    let port = env::var("PLUGINS_SERVICE_PORT")
        .unwrap_or_else(|_| "3020".to_string())
        .parse::<u16>()
        .unwrap_or(3020);

    info!("🔌 [Plugins Service] Starting on port {}", port);

    // Initialize plugin registry
    let mut registry = PluginRegistry::new();
    
    // Register all available plugin factories
    register_plugin_factories(&mut registry).await?;

    // Load plugin configurations
    let config_path = env::var("PLUGIN_CONFIG_PATH")
        .unwrap_or_else(|_| "./config/plugins.json".to_string());
    
    let config_manager = match PluginConfigManager::load_from_file(&config_path) {
        Ok(config) => {
            info!("📋 [Plugins Service] Loaded configuration from {}", config_path);
            config
        }
        Err(_) => {
            info!("📋 [Plugins Service] Creating default configuration");
            let config = PluginConfigManager::create_default();
            if let Err(e) = config.save_to_file(&config_path) {
                error!("Failed to save default config: {}", e);
            }
            config
        }
    };

    // Auto-start enabled plugins
    let auto_start_plugins = config_manager.get_auto_start();
    for plugin_config in auto_start_plugins {
        if let Err(e) = start_plugin(&registry, plugin_config).await {
            error!("Failed to auto-start plugin {}: {}", plugin_config.instance_id, e);
        }
    }

    // Create application state
    let app_state = AppState {
        registry: Arc::new(RwLock::new(registry)),
        config_manager: Arc::new(RwLock::new(config_manager)),
    };

    info!("✅ [Plugins Service] Initialization complete");

    // Start HTTP server
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .configure(configure_routes)
            .route("/health", web::get().to(health_check))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}

/// Register all available plugin factories
async fn register_plugin_factories(registry: &mut PluginRegistry) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔧 [Plugins Service] Registering plugin factories...");
    
    // Register source plugins
    #[cfg(feature = "dropbox-plugin")]
    {
        use dropbox_plugin::DropboxPluginFactory;
        registry.register_source_factory(Box::new(DropboxPluginFactory::new()));
        info!("  ✓ Registered Dropbox source plugin");
    }

    // TODO: Other plugins need factory pattern implementation
    // #[cfg(feature = "google-drive-plugin")]
    // {
    //     use google_drive_plugin::GoogleDrivePluginFactory;
    //     registry.register_source_factory(Box::new(GoogleDrivePluginFactory::new()));
    //     info!("  ✓ Registered Google Drive source plugin");
    // }

    // Register agent plugins
    // #[cfg(feature = "cline-plugin")]
    // {
    //     use cline_plugin::ClinePluginFactory;
    //     registry.register_agent_factory(Box::new(ClinePluginFactory::new()));
    //     info!("  ✓ Registered Cline agent plugin");
    // }

    // #[cfg(feature = "amazon-q-plugin")]
    // {
    //     use amazon_q_plugin::AmazonQPluginFactory;
    //     registry.register_agent_factory(Box::new(AmazonQPluginFactory::new()));
    //     info!("  ✓ Registered Amazon Q agent plugin");
    // }

    info!("🔧 [Plugins Service] Plugin factory registration complete");
    Ok(())
}

/// Start a plugin based on its configuration
async fn start_plugin(registry: &PluginRegistry, config: &PluginInstanceConfig) -> Result<(), Box<dyn std::error::Error>> {
    info!("🚀 [Plugins Service] Starting plugin: {}", config.instance_id);

    match config.plugin_type {
        PluginType::Source => {
            registry.load_source(&config.plugin_name, &config.instance_id, config.config.clone()).await?;
        }
        PluginType::Agent => {
            registry.load_agent(&config.plugin_name, &config.instance_id, config.config.clone()).await?;
        }
    }

    info!("✅ [Plugins Service] Plugin started: {}", config.instance_id);
    Ok(())
}

/// Configure API routes
fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/plugins")
            // Plugin management
            .route("/registry/sources", web::get().to(handlers::registry::list_source_types))
            .route("/registry/agents", web::get().to(handlers::registry::list_agent_types))
            .route("/active/sources", web::get().to(handlers::registry::list_active_sources))
            .route("/active/agents", web::get().to(handlers::registry::list_active_agents))
            
            // Plugin lifecycle
            .route("/start/{instance_id}", web::post().to(handlers::lifecycle::start_plugin))
            .route("/stop/{instance_id}", web::post().to(handlers::lifecycle::stop_plugin))
            .route("/restart/{instance_id}", web::post().to(handlers::lifecycle::restart_plugin))
            .route("/status/{instance_id}", web::get().to(handlers::lifecycle::get_plugin_status))
            
            // Configuration management
            .route("/config", web::get().to(handlers::config::get_all_configs))
            .route("/config", web::post().to(handlers::config::create_config))
            .route("/config/{instance_id}", web::get().to(handlers::config::get_config))
            .route("/config/{instance_id}", web::put().to(handlers::config::update_config))
            .route("/config/{instance_id}", web::delete().to(handlers::config::delete_config))
            
            // Source plugin operations
            .route("/sources/{instance_id}/documents", web::get().to(handlers::sources::list_documents))
            .route("/sources/{instance_id}/documents/{doc_id}", web::get().to(handlers::sources::get_document))
            .route("/sources/{instance_id}/search", web::post().to(handlers::sources::search_documents))
            .route("/sources/{instance_id}/sync", web::post().to(handlers::sources::sync_source))
            
            // Agent plugin operations
            .route("/agents/{instance_id}/chat", web::post().to(handlers::agents::chat_with_agent))
            .route("/agents/{instance_id}/functions", web::get().to(handlers::agents::get_agent_functions))
            .route("/agents/{instance_id}/execute", web::post().to(handlers::agents::execute_agent_action))
    );
}

/// Health check endpoint
async fn health_check(data: web::Data<AppState>) -> ActixResult<web::Json<serde_json::Value>> {
    let registry = data.registry.read().await;
    let health_results = registry.health_check_all().await;
    
    let healthy_count = health_results.values().filter(|&&healthy| healthy).count();
    let total_count = health_results.len();
    
    Ok(web::Json(serde_json::json!({
        "status": "healthy",
        "service": "plugins-service",
        "plugins": {
            "total": total_count,
            "healthy": healthy_count,
            "unhealthy": total_count - healthy_count
        },
        "plugin_health": health_results,
        "timestamp": chrono::Utc::now()
    })))
}