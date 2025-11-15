#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

console.log('🔧 Temporarily fixing data service for startup...');

// Create a minimal working version of the data service
const minimalMain = `
use actix_web::{web, App, HttpServer, HttpResponse, Result};
use tracing::{info, warn};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let port = env::var("PORT").unwrap_or_else(|_| "3013".to_string());
    let bind_addr = format!("0.0.0.0:{}", port);
    
    info!("🚀 [Data Service] Starting on port {}", port);
    info!("⚠️  [Data Service] Running in minimal mode - database features disabled");
    
    HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(health_check))
            .route("/status", web::get().to(status_check))
    })
    .bind(&bind_addr)?
    .run()
    .await
}

async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "data-service",
        "mode": "minimal",
        "message": "Database features temporarily disabled"
    })))
}

async fn status_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "service": "data-service",
        "version": "0.1.0",
        "status": "running",
        "features": {
            "database": false,
            "connectors": false,
            "embedding": false
        }
    })))
}
`;

// Write the minimal main.rs
const dataServicePath = path.join(__dirname, '../data/src');
const mainPath = path.join(dataServicePath, 'main.rs');

// Backup original main.rs
if (fs.existsSync(mainPath)) {
    fs.copyFileSync(mainPath, mainPath + '.backup');
}

fs.writeFileSync(mainPath, minimalMain);

// Update Cargo.toml to remove problematic dependencies temporarily
const cargoTomlPath = path.join(__dirname, '../data/Cargo.toml');
const cargoContent = fs.readFileSync(cargoTomlPath, 'utf8');

const minimalCargoToml = `[package]
name = "data-service"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
`;

// Backup original Cargo.toml
fs.copyFileSync(cargoTomlPath, cargoTomlPath + '.backup');
fs.writeFileSync(cargoTomlPath, minimalCargoToml);

console.log('✅ Data service temporarily fixed for startup');
console.log('📝 Original files backed up with .backup extension');
console.log('🔄 Run "npm run restore-data-service" to restore full functionality');