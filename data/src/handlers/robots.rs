//! Robot registration and management handlers
//! 
//! This module provides HTTP endpoints for:
//! - Robot registration
//! - Stream declaration
//! - Robot status and configuration

use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use tracing::{info, error, warn};
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;

use crate::services::kafka_client::{generate_robot_topics, generate_stream_topic};

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

/// Request to register a new robot
#[derive(Debug, Deserialize)]
pub struct RegisterRobotRequest {
    pub name: String,
    #[serde(default)]
    pub robot_type: String,
    pub description: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub sensors: Vec<SensorDeclaration>,
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
}

/// Sensor declaration during registration
#[derive(Debug, Deserialize)]
pub struct SensorDeclaration {
    pub name: String,
    #[serde(rename = "type")]
    pub sensor_type: String,
    pub sample_rate_hz: Option<f64>,
}

/// Response after registering a robot
#[derive(Debug, Serialize)]
pub struct RegisterRobotResponse {
    pub robot_id: String,
    pub kafka: KafkaConnectionInfo,
    pub http_ingest: HttpIngestEndpoints,
    pub streams: Vec<StreamInfo>,
}

/// Kafka connection info
#[derive(Debug, Serialize)]
pub struct KafkaConnectionInfo {
    pub bootstrap_servers: String,
    pub topics: HashMap<String, String>,
    pub auth: Option<KafkaAuthInfo>,
}

/// Kafka auth info
#[derive(Debug, Serialize)]
pub struct KafkaAuthInfo {
    pub mechanism: String,
    pub username: String,
    pub password: String,
}

/// HTTP ingestion endpoints
#[derive(Debug, Serialize)]
pub struct HttpIngestEndpoints {
    pub events_url: String,
    pub cv_events_url: String,
    pub frames_url: String,
}

/// Stream info in response
#[derive(Debug, Serialize)]
pub struct StreamInfo {
    pub stream_id: String,
    pub name: String,
    pub kafka_topic: Option<String>,
}

/// Request to declare a new stream
#[derive(Debug, Deserialize)]
pub struct DeclareStreamRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub stream_type: String,
    pub description: Option<String>,
    pub schema: Option<StreamSchema>,
    pub sample_rate_hz: Option<f64>,
    pub buffer_size: Option<i32>,
}

/// Stream schema
#[derive(Debug, Deserialize, Serialize)]
pub struct StreamSchema {
    pub fields: Vec<SchemaField>,
}

/// Schema field
#[derive(Debug, Deserialize, Serialize)]
pub struct SchemaField {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default)]
    pub required: bool,
    pub description: Option<String>,
}

/// Response after declaring a stream
#[derive(Debug, Serialize)]
pub struct DeclareStreamResponse {
    pub stream_id: String,
    pub kafka_topic: Option<String>,
}

/// Robot info response
#[derive(Debug, Serialize)]
pub struct RobotInfoResponse {
    pub robot_id: String,
    pub name: String,
    pub robot_type: String,
    pub status: String,
    pub capabilities: Vec<String>,
    pub kafka: KafkaConnectionInfo,
    pub http_ingest: HttpIngestEndpoints,
    pub streams: Vec<StreamInfo>,
    pub created_at: String,
    pub last_heartbeat: Option<String>,
}

// ============================================================================
// HANDLERS
// ============================================================================

/// Register a new robot
/// 
/// POST /api/robots/register
pub async fn register_robot(
    req: web::Json<RegisterRobotRequest>,
) -> Result<HttpResponse> {
    info!("🤖 Registering new robot: {}", req.name);
    
    // Generate robot ID
    let robot_id = Uuid::new_v4();
    
    // Generate Kafka topics
    let kafka_topics = generate_robot_topics(&robot_id);
    
    // Get Kafka config from environment
    let kafka_bootstrap = std::env::var("KAFKA_BOOTSTRAP_SERVERS")
        .unwrap_or_else(|_| "localhost:9092".to_string());
    
    let kafka_enabled = std::env::var("KAFKA_ENABLED")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);
    
    // Build HTTP endpoints
    let base_url = format!("/api/ingestion/robots/{}", robot_id);
    let http_endpoints = HttpIngestEndpoints {
        events_url: format!("{}/events", base_url),
        cv_events_url: format!("{}/cv_events", base_url),
        frames_url: format!("{}/frames", base_url),
    };
    
    // Process declared sensors into streams
    let mut streams = Vec::new();
    for sensor in &req.sensors {
        let stream_id = Uuid::new_v4();
        let kafka_topic = if kafka_enabled {
            Some(generate_stream_topic(&robot_id, &sensor.name))
        } else {
            None
        };
        
        streams.push(StreamInfo {
            stream_id: stream_id.to_string(),
            name: sensor.name.clone(),
            kafka_topic,
        });
    }
    
    // Build Kafka connection info
    let kafka_auth = if kafka_enabled {
        std::env::var("KAFKA_SASL_USERNAME").ok().map(|username| {
            KafkaAuthInfo {
                mechanism: std::env::var("KAFKA_SASL_MECHANISM")
                    .unwrap_or_else(|_| "SCRAM-SHA-256".to_string()),
                username,
                password: std::env::var("KAFKA_SASL_PASSWORD")
                    .unwrap_or_default(),
            }
        })
    } else {
        None
    };
    
    let kafka_info = KafkaConnectionInfo {
        bootstrap_servers: kafka_bootstrap,
        topics: kafka_topics,
        auth: kafka_auth,
    };
    
    // TODO: Store robot in database
    // For now, we just return the registration info
    
    info!("✅ Robot registered successfully: {} ({})", req.name, robot_id);
    
    let response = RegisterRobotResponse {
        robot_id: robot_id.to_string(),
        kafka: kafka_info,
        http_ingest: http_endpoints,
        streams,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

/// Get robot information
/// 
/// GET /api/robots/{robot_id}
pub async fn get_robot(
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let robot_id_str = path.into_inner();
    
    info!("🔍 Getting robot info: {}", robot_id_str);
    
    let robot_id = match Uuid::parse_str(&robot_id_str) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid robot ID format"
            })));
        }
    };
    
    // TODO: Fetch from database
    // For now, return a mock response
    
    let kafka_topics = generate_robot_topics(&robot_id);
    let kafka_bootstrap = std::env::var("KAFKA_BOOTSTRAP_SERVERS")
        .unwrap_or_else(|_| "localhost:9092".to_string());
    
    let response = RobotInfoResponse {
        robot_id: robot_id.to_string(),
        name: "Unknown Robot".to_string(),
        robot_type: "generic".to_string(),
        status: "registered".to_string(),
        capabilities: vec![],
        kafka: KafkaConnectionInfo {
            bootstrap_servers: kafka_bootstrap,
            topics: kafka_topics,
            auth: None,
        },
        http_ingest: HttpIngestEndpoints {
            events_url: format!("/api/ingestion/robots/{}/events", robot_id),
            cv_events_url: format!("/api/ingestion/robots/{}/cv_events", robot_id),
            frames_url: format!("/api/ingestion/robots/{}/frames", robot_id),
        },
        streams: vec![],
        created_at: Utc::now().to_rfc3339(),
        last_heartbeat: None,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

/// Declare a new stream for a robot
/// 
/// POST /api/robots/{robot_id}/streams
pub async fn declare_stream(
    path: web::Path<String>,
    req: web::Json<DeclareStreamRequest>,
) -> Result<HttpResponse> {
    let robot_id_str = path.into_inner();
    
    info!("📡 Declaring stream '{}' for robot {}", req.name, robot_id_str);
    
    let robot_id = match Uuid::parse_str(&robot_id_str) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid robot ID format"
            })));
        }
    };
    
    let stream_id = Uuid::new_v4();
    
    let kafka_enabled = std::env::var("KAFKA_ENABLED")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);
    
    let kafka_topic = if kafka_enabled {
        Some(generate_stream_topic(&robot_id, &req.name))
    } else {
        None
    };
    
    // TODO: Store stream in database
    
    info!("✅ Stream declared: {} -> {:?}", req.name, kafka_topic);
    
    let response = DeclareStreamResponse {
        stream_id: stream_id.to_string(),
        kafka_topic,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

/// List all robots for the current user/tenant
/// 
/// GET /api/robots
pub async fn list_robots() -> Result<HttpResponse> {
    info!("📋 Listing robots");
    
    // TODO: Fetch from database with tenant filtering
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "robots": [],
        "total": 0
    })))
}

/// Update robot status (heartbeat)
/// 
/// POST /api/robots/{robot_id}/heartbeat
pub async fn robot_heartbeat(
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let robot_id_str = path.into_inner();
    
    info!("💓 Heartbeat from robot: {}", robot_id_str);
    
    // TODO: Update last_heartbeat in database
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "timestamp": Utc::now().to_rfc3339()
    })))
}

/// Delete a robot
/// 
/// DELETE /api/robots/{robot_id}
pub async fn delete_robot(
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let robot_id_str = path.into_inner();
    
    info!("🗑️ Deleting robot: {}", robot_id_str);
    
    // TODO: Delete from database
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "deleted": true,
        "robot_id": robot_id_str
    })))
}
