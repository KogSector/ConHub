use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use sqlx::{PgPool, postgres::{PgPoolOptions, PgConnectOptions}};
use std::str::FromStr;
use std::env;
use conhub_observability::{init_tracing, TracingConfig, observability, info, warn, error};

mod services;
mod handlers;

use services::role_auth_middleware;
use conhub_middleware::auth::AuthMiddlewareFactory;
use conhub_config::feature_toggles::FeatureToggles;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize observability with structured logging
    init_tracing(TracingConfig::for_service("auth-service"));

    // Load environment variables
    dotenv::dotenv().ok();

    // Service port
    let port = env::var("AUTH_SERVICE_PORT")
        .unwrap_or_else(|_| "3010".to_string())
        .parse::<u16>()
        .unwrap_or(3010);

    // Feature toggles
    let toggles = FeatureToggles::from_env_path();

    // Initialize authentication middleware (must succeed; Auth is always required)
    let auth_middleware = AuthMiddlewareFactory::new()
        .map_err(|e| {
            tracing::error!("Failed to initialize auth middleware: {}", e);
            e
        })?;

    // Database connection
    // When Auth is enabled: required
    // When Auth is disabled: optional (connect if DB URL is present for dev user seeding)
    let database_url_opt = env::var("DATABASE_URL_NEON")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| env::var("DATABASE_URL").ok().filter(|v| !v.trim().is_empty()));

    let database_url = database_url_opt
        .unwrap_or_else(|| "postgresql://conhub:conhub_password@localhost:5432/conhub".to_string());

    if env::var("DATABASE_URL_NEON").ok().filter(|v| !v.trim().is_empty()).is_some() {
        tracing::info!("📊 [Auth Service] Connecting to Neon DB...");
    } else {
        tracing::info!("📊 [Auth Service] Connecting to database...");
    }
    tracing::info!(
        "🔗 [Auth Service] Database URL: {}",
        database_url.split('@').last().unwrap_or("hidden")
    );

    // Disable server-side prepared statements for pgbouncer/Neon transaction pooling
    let connect_options = PgConnectOptions::from_str(&database_url)?
        .statement_cache_capacity(0);

    tracing::info!("🔌 [Auth Service] Attempting database connection...");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect_with(connect_options)
        .await?;
    tracing::info!("✅ [Auth Service] Database connection established successfully");
    tracing::info!("📊 [Auth Service] Database pool created with max 10 connections");
    let db_pool_opt: Option<PgPool> = Some(pool);

    // Redis connection for sessions (gated by Auth and Redis toggles)
    let redis_enabled = toggles.should_connect_redis();
    let redis_client_opt: Option<redis::Client> = if redis_enabled {
        let redis_url = env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        println!("📊 [Auth Service] Connecting to Redis...");
        match redis::Client::open(redis_url.clone()) {
            Ok(client) => {
                match client.get_async_connection().await {
                    Ok(mut conn) => {
                        match redis::cmd("PING").query_async::<_, String>(&mut conn).await {
                            Ok(_) => Some(client),
                            Err(_) => None,
                        }
                    }
                    Err(_) => None,
                }
            }
            Err(_) => None,
        }
    } else {
        tracing::warn!("[Auth Service] Redis feature disabled; skipping Redis connection.");
        None
    };

    println!("🚀 [Auth Service] Starting on port {}", port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        let mut app = App::new()
            .app_data(web::Data::new(toggles.clone()))
            // Optional PgPool for handlers that take web::Data<Option<PgPool>>
            .app_data(web::Data::new(db_pool_opt.clone()))
            .wrap(cors)
            .wrap(observability("auth-service"));

        // Non-optional PgPool for legacy handlers that expect web::Data<PgPool>
        if let Some(pool) = db_pool_opt.clone() {
            app = app.app_data(web::Data::new(pool));
        }

        if let Some(redis_client) = redis_client_opt.clone() {
            app = app.app_data(web::Data::new(redis_client));
        }

        app
            .route("/health", web::get().to(health_check))
            .configure(|cfg| configure_routes(cfg, auth_middleware.clone()))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}

fn configure_routes(cfg: &mut web::ServiceConfig, auth_middleware: AuthMiddlewareFactory) {
    use conhub_models::auth::UserRole;
    
    let mut scope = web::scope("/api/auth")
        .route("/forgot-password", web::post().to(handlers::auth::forgot_password))
        .route("/reset-password", web::post().to(handlers::auth::reset_password))
        // OAuth routes (public)
        .route("/oauth/url", web::get().to(handlers::auth::oauth_url))
        .route("/oauth/{provider}", web::get().to(handlers::oauth::oauth_login))
        .route("/oauth/{provider}/callback", web::get().to(handlers::oauth::oauth_callback))
        // Auth0 exchange endpoint (public)
        .route("/auth0/exchange", web::post().to(handlers::auth0::auth0_exchange));
    scope = scope
        .route("/login", web::post().to(handlers::auth::login))
        .route("/register", web::post().to(handlers::auth::register))
        .route("/connections/current", web::get().to(handlers::auth::list_auth_connections_current))
        .service(
            web::scope("")
                .wrap(auth_middleware)
                .route("/logout", web::post().to(handlers::auth::logout))
                .route("/me", web::get().to(handlers::auth::get_current_user))
                .route("/verify", web::post().to(handlers::auth::verify_token))
                .route("/refresh", web::post().to(handlers::auth::refresh_token))
                .route("/profile", web::get().to(handlers::auth::get_profile))
                .route("/connections", web::get().to(handlers::auth::list_auth_connections))
                .route("/connections/{id}", web::delete().to(handlers::auth::disconnect_auth_connection))
                .route("/oauth/exchange", web::post().to(handlers::auth::oauth_exchange))
                .route("/repos/github", web::get().to(handlers::auth::list_github_repos))
                .route("/repos/github/branches", web::get().to(handlers::auth::list_github_branches))
                .route("/repos/bitbucket", web::get().to(handlers::auth::list_bitbucket_repos))
                .route("/repos/bitbucket/branches", web::get().to(handlers::auth::list_bitbucket_branches))
                .route("/repos/check", web::post().to(handlers::auth::check_repo))
                .service(
                    web::scope("/admin")
                        .wrap(role_auth_middleware(vec![UserRole::Admin]))
                        .route("/users", web::get().to(handlers::auth::list_users))
                )
        );

    cfg.service(scope);

    // Internal service-to-service endpoints (no auth middleware - protected by network/service mesh)
    cfg.service(
        web::scope("/internal")
            .route("/oauth/{provider}/token", web::get().to(handlers::auth::internal_get_oauth_token))
            .route("/oauth/{provider}/status", web::get().to(handlers::auth::internal_check_oauth_status))
    );
}

async fn health_check(pool_opt: web::Data<Option<PgPool>>) -> actix_web::Result<web::Json<serde_json::Value>> {
    // Check database connection
    let db_status = match pool_opt.get_ref() {
        Some(pool) => match sqlx::query("SELECT 1 as test").fetch_one(pool).await {
            Ok(_) => "connected",
            Err(e) => {
                tracing::error!("[Auth Service] Database health check failed: {}", e);
                "disconnected"
            }
        },
        None => "disabled",
    };

    Ok(web::Json(serde_json::json!({
        "status": "healthy",
        "service": "auth-service",
        "database": db_status,
        "timestamp": chrono::Utc::now()
    })))
}
