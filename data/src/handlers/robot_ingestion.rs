//! Robot event ingestion handlers
//! 
//! This module provides HTTP endpoints for ingesting robot data:
//! - Raw sensor events
//! - Computer vision events
//! - Frame references
//! 
//! These endpoints act as an HTTP → Kafka bridge for robots that
//! cannot use Kafka clients directly.

use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use tracing::{info, error, warn};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::services::kafka_client::{KafkaProducer, generate_robot_topics};

// ============================================================================
// REQUEST TYPES
// ============================================================================

/// Raw sensor event from a robot
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RobotEventRequest {
    pub source: String,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub payload: serde_json::Value,
}

/// Batch of raw events
#[derive(Debug, Deserialize)]
pub struct RobotEventBatchRequest {
    pub events: Vec<RobotEventRequest>,
}

/// Computer vision event
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CvEventRequest {
    pub source: String,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub payload: CvPayload,
}

/// CV event payload
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CvPayload {
    ObjectDetection {
        object_class: String,
        bbox: [f64; 4],
        confidence: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        track_id: Option<String>,
    },
    FaceRecognition {
        #[serde(skip_serializing_if = "Option::is_none")]
        person_id: Option<String>,
        bbox: [f64; 4],
        confidence: f64,
    },
    PoseEstimation {
        #[serde(skip_serializing_if = "Option::is_none")]
        person_id: Option<String>,
        keypoints: Vec<Keypoint>,
        confidence: f64,
    },
    Segmentation {
        class_id: i32,
        class_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        mask_rle: Option<String>,
        confidence: f64,
    },
    Custom(serde_json::Value),
}

/// Keypoint for pose estimation
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Keypoint {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub confidence: f64,
}

/// Batch of CV events
#[derive(Debug, Deserialize)]
pub struct CvEventBatchRequest {
    pub events: Vec<CvEventRequest>,
}

/// Frame reference event
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FrameEventRequest {
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub image_uri: String,
    #[serde(default)]
    pub metadata: FrameMetadata,
}

/// Frame metadata
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct FrameMetadata {
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub format: Option<String>,
    pub encoding: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub camera_info: Option<serde_json::Value>,
}

// ============================================================================
// RESPONSE TYPES
// ============================================================================

/// Response for event ingestion
#[derive(Debug, Serialize)]
pub struct IngestResponse {
    pub success: bool,
    pub events_received: usize,
    pub events_published: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// KAFKA MESSAGE WRAPPER
// ============================================================================

/// Wrapper for Kafka messages with metadata
#[derive(Debug, Serialize)]
struct KafkaEventWrapper<T: Serialize> {
    pub robot_id: Uuid,
    pub tenant_id: Uuid,
    pub message_id: Uuid,
    pub received_at: DateTime<Utc>,
    pub event: T,
}

// ============================================================================
// HANDLERS
// ============================================================================

/// Ingest raw sensor events from a robot
/// 
/// POST /api/ingestion/robots/{robot_id}/events
pub async fn ingest_events(
    path: web::Path<String>,
    req: web::Json<RobotEventRequest>,
    producer: web::Data<Arc<KafkaProducer>>,
) -> Result<HttpResponse> {
    let robot_id_str = path.into_inner();
    
    let robot_id = match Uuid::parse_str(&robot_id_str) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid robot ID format"
            })));
        }
    };
    
    info!("📥 Ingesting event from robot {} - source: {}, type: {}", 
          robot_id, req.source, req.event_type);
    
    // Get topic for raw events
    let topics = generate_robot_topics(&robot_id);
    let topic = topics.get("raw_events").cloned()
        .unwrap_or_else(|| format!("robot.{}.raw_events", robot_id));
    
    // Wrap event with metadata
    // TODO: Get tenant_id from auth context
    let tenant_id = Uuid::nil(); // Placeholder
    
    let wrapped = KafkaEventWrapper {
        robot_id,
        tenant_id,
        message_id: Uuid::new_v4(),
        received_at: Utc::now(),
        event: req.into_inner(),
    };
    
    // Publish to Kafka
    match producer.publish(&topic, Some(&robot_id.to_string()), &wrapped).await {
        Ok(metadata) => {
            info!("✅ Event published to {} (offset: {})", metadata.topic, metadata.offset);
            
            Ok(HttpResponse::Ok().json(IngestResponse {
                success: true,
                events_received: 1,
                events_published: 1,
                errors: None,
                timestamp: Utc::now(),
            }))
        }
        Err(e) => {
            error!("❌ Failed to publish event: {}", e);
            
            Ok(HttpResponse::InternalServerError().json(IngestResponse {
                success: false,
                events_received: 1,
                events_published: 0,
                errors: Some(vec![e.to_string()]),
                timestamp: Utc::now(),
            }))
        }
    }
}

/// Ingest a batch of raw sensor events
/// 
/// POST /api/ingestion/robots/{robot_id}/events/batch
pub async fn ingest_events_batch(
    path: web::Path<String>,
    req: web::Json<RobotEventBatchRequest>,
    producer: web::Data<Arc<KafkaProducer>>,
) -> Result<HttpResponse> {
    let robot_id_str = path.into_inner();
    
    let robot_id = match Uuid::parse_str(&robot_id_str) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid robot ID format"
            })));
        }
    };
    
    let events = req.into_inner().events;
    let event_count = events.len();
    
    info!("📥 Ingesting {} events from robot {}", event_count, robot_id);
    
    let topics = generate_robot_topics(&robot_id);
    let topic = topics.get("raw_events").cloned()
        .unwrap_or_else(|| format!("robot.{}.raw_events", robot_id));
    
    let tenant_id = Uuid::nil(); // Placeholder
    
    let mut published = 0;
    let mut errors = Vec::new();
    
    for event in events {
        let wrapped = KafkaEventWrapper {
            robot_id,
            tenant_id,
            message_id: Uuid::new_v4(),
            received_at: Utc::now(),
            event,
        };
        
        match producer.publish(&topic, Some(&robot_id.to_string()), &wrapped).await {
            Ok(_) => published += 1,
            Err(e) => errors.push(e.to_string()),
        }
    }
    
    info!("✅ Published {}/{} events", published, event_count);
    
    Ok(HttpResponse::Ok().json(IngestResponse {
        success: errors.is_empty(),
        events_received: event_count,
        events_published: published,
        errors: if errors.is_empty() { None } else { Some(errors) },
        timestamp: Utc::now(),
    }))
}

/// Ingest computer vision events
/// 
/// POST /api/ingestion/robots/{robot_id}/cv_events
pub async fn ingest_cv_events(
    path: web::Path<String>,
    req: web::Json<CvEventRequest>,
    producer: web::Data<Arc<KafkaProducer>>,
) -> Result<HttpResponse> {
    let robot_id_str = path.into_inner();
    
    let robot_id = match Uuid::parse_str(&robot_id_str) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid robot ID format"
            })));
        }
    };
    
    info!("👁️ Ingesting CV event from robot {} - source: {}, type: {}", 
          robot_id, req.source, req.event_type);
    
    let topics = generate_robot_topics(&robot_id);
    let topic = topics.get("cv_events").cloned()
        .unwrap_or_else(|| format!("robot.{}.cv_events", robot_id));
    
    let tenant_id = Uuid::nil();
    
    let wrapped = KafkaEventWrapper {
        robot_id,
        tenant_id,
        message_id: Uuid::new_v4(),
        received_at: Utc::now(),
        event: req.into_inner(),
    };
    
    match producer.publish(&topic, Some(&robot_id.to_string()), &wrapped).await {
        Ok(metadata) => {
            info!("✅ CV event published to {} (offset: {})", metadata.topic, metadata.offset);
            
            Ok(HttpResponse::Ok().json(IngestResponse {
                success: true,
                events_received: 1,
                events_published: 1,
                errors: None,
                timestamp: Utc::now(),
            }))
        }
        Err(e) => {
            error!("❌ Failed to publish CV event: {}", e);
            
            Ok(HttpResponse::InternalServerError().json(IngestResponse {
                success: false,
                events_received: 1,
                events_published: 0,
                errors: Some(vec![e.to_string()]),
                timestamp: Utc::now(),
            }))
        }
    }
}

/// Ingest a batch of CV events
/// 
/// POST /api/ingestion/robots/{robot_id}/cv_events/batch
pub async fn ingest_cv_events_batch(
    path: web::Path<String>,
    req: web::Json<CvEventBatchRequest>,
    producer: web::Data<Arc<KafkaProducer>>,
) -> Result<HttpResponse> {
    let robot_id_str = path.into_inner();
    
    let robot_id = match Uuid::parse_str(&robot_id_str) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid robot ID format"
            })));
        }
    };
    
    let events = req.into_inner().events;
    let event_count = events.len();
    
    info!("👁️ Ingesting {} CV events from robot {}", event_count, robot_id);
    
    let topics = generate_robot_topics(&robot_id);
    let topic = topics.get("cv_events").cloned()
        .unwrap_or_else(|| format!("robot.{}.cv_events", robot_id));
    
    let tenant_id = Uuid::nil();
    
    let mut published = 0;
    let mut errors = Vec::new();
    
    for event in events {
        let wrapped = KafkaEventWrapper {
            robot_id,
            tenant_id,
            message_id: Uuid::new_v4(),
            received_at: Utc::now(),
            event,
        };
        
        match producer.publish(&topic, Some(&robot_id.to_string()), &wrapped).await {
            Ok(_) => published += 1,
            Err(e) => errors.push(e.to_string()),
        }
    }
    
    info!("✅ Published {}/{} CV events", published, event_count);
    
    Ok(HttpResponse::Ok().json(IngestResponse {
        success: errors.is_empty(),
        events_received: event_count,
        events_published: published,
        errors: if errors.is_empty() { None } else { Some(errors) },
        timestamp: Utc::now(),
    }))
}

/// Ingest frame references
/// 
/// POST /api/ingestion/robots/{robot_id}/frames
pub async fn ingest_frames(
    path: web::Path<String>,
    req: web::Json<FrameEventRequest>,
    producer: web::Data<Arc<KafkaProducer>>,
) -> Result<HttpResponse> {
    let robot_id_str = path.into_inner();
    
    let robot_id = match Uuid::parse_str(&robot_id_str) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid robot ID format"
            })));
        }
    };
    
    info!("🖼️ Ingesting frame from robot {} - source: {}, uri: {}", 
          robot_id, req.source, req.image_uri);
    
    let topics = generate_robot_topics(&robot_id);
    let topic = topics.get("frames").cloned()
        .unwrap_or_else(|| format!("robot.{}.frames", robot_id));
    
    let tenant_id = Uuid::nil();
    
    let wrapped = KafkaEventWrapper {
        robot_id,
        tenant_id,
        message_id: Uuid::new_v4(),
        received_at: Utc::now(),
        event: req.into_inner(),
    };
    
    match producer.publish(&topic, Some(&robot_id.to_string()), &wrapped).await {
        Ok(metadata) => {
            info!("✅ Frame reference published to {} (offset: {})", metadata.topic, metadata.offset);
            
            Ok(HttpResponse::Ok().json(IngestResponse {
                success: true,
                events_received: 1,
                events_published: 1,
                errors: None,
                timestamp: Utc::now(),
            }))
        }
        Err(e) => {
            error!("❌ Failed to publish frame reference: {}", e);
            
            Ok(HttpResponse::InternalServerError().json(IngestResponse {
                success: false,
                events_received: 1,
                events_published: 0,
                errors: Some(vec![e.to_string()]),
                timestamp: Utc::now(),
            }))
        }
    }
}
