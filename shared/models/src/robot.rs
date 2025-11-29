//! Robot and sensor models for the robot memory system
//! 
//! This module defines the core types for:
//! - Robot registration and management
//! - Sensor stream configuration
//! - Episodic and semantic memory
//! - Event ingestion from robots and CV systems

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// ROBOT TYPES
// ============================================================================

/// Type of robot or autonomous agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RobotType {
    Generic,
    Mobile,       // Ground-based mobile robot
    Arm,          // Robotic arm / manipulator
    Drone,        // Aerial robot
    Humanoid,     // Humanoid robot
    Quadruped,    // Four-legged robot
    Underwater,   // Underwater robot
    Custom(String),
}

impl Default for RobotType {
    fn default() -> Self {
        Self::Generic
    }
}

/// Robot connection status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RobotStatus {
    Registered,   // Just registered, not yet connected
    Connected,    // Actively sending data
    Disconnected, // Was connected, now offline
    Error,        // Connection error
}

impl Default for RobotStatus {
    fn default() -> Self {
        Self::Registered
    }
}

/// Robot capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RobotCapability {
    Navigation,
    Manipulation,
    Vision,
    Speech,
    Hearing,
    Touch,
    Localization,
    Mapping,
    ObjectDetection,
    FaceRecognition,
    NaturalLanguage,
    Custom(String),
}

/// Kafka connection configuration for a robot
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KafkaConfig {
    pub bootstrap_servers: String,
    pub topics: KafkaTopics,
    pub auth: Option<KafkaAuth>,
}

/// Kafka topics assigned to a robot
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KafkaTopics {
    pub raw_events: String,
    pub cv_events: String,
    pub frames: String,
    pub episodes: String,
    pub semantic_events: String,
    pub control: String,
}

/// Kafka authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaAuth {
    pub mechanism: String, // SCRAM-SHA-256, PLAIN, etc.
    pub username: String,
    pub password: String,
}

/// HTTP ingestion endpoints for a robot
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HttpEndpoints {
    pub events_url: String,
    pub cv_events_url: String,
    pub frames_url: String,
}

/// A registered robot in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Robot {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    
    pub name: String,
    pub robot_type: RobotType,
    pub description: Option<String>,
    
    pub capabilities: Vec<RobotCapability>,
    
    pub status: RobotStatus,
    pub last_heartbeat: Option<DateTime<Utc>>,
    
    pub kafka_config: Option<KafkaConfig>,
    pub http_endpoints: Option<HttpEndpoints>,
    
    pub config: HashMap<String, serde_json::Value>,
    pub metadata: HashMap<String, serde_json::Value>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// SENSOR STREAM TYPES
// ============================================================================

/// Type of sensor stream
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StreamType {
    CameraRgb,
    CameraDepth,
    CameraStereo,
    Lidar2d,
    Lidar3d,
    Imu,
    Encoder,
    Gps,
    Audio,
    Tactile,
    Force,
    Temperature,
    Custom(String),
}

/// Stream status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StreamStatus {
    Active,
    Paused,
    Error,
}

impl Default for StreamStatus {
    fn default() -> Self {
        Self::Active
    }
}

/// Schema field for stream events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaField {
    pub name: String,
    pub field_type: String, // timestamp, string, float, int, bool, json
    pub required: bool,
    pub description: Option<String>,
}

/// Event schema for a stream
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventSchema {
    pub fields: Vec<SchemaField>,
}

/// A sensor stream from a robot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotStream {
    pub id: Uuid,
    pub robot_id: Uuid,
    
    pub name: String,
    pub stream_type: StreamType,
    pub description: Option<String>,
    
    pub event_schema: EventSchema,
    pub kafka_topic: Option<String>,
    
    pub status: StreamStatus,
    
    pub sample_rate_hz: Option<f64>,
    pub buffer_size: Option<i32>,
    pub config: HashMap<String, serde_json::Value>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// EPISODE TYPES (EPISODIC MEMORY)
// ============================================================================

/// Type of episode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeType {
    General,
    Navigation,
    Manipulation,
    Interaction,
    Observation,
    Task,
    Error,
    Custom(String),
}

impl Default for EpisodeType {
    fn default() -> Self {
        Self::General
    }
}

/// Location coordinates
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocationCoordinates {
    pub x: f64,
    pub y: f64,
    pub z: Option<f64>,
    pub frame: String, // "map", "odom", "world", etc.
    pub orientation: Option<Orientation>,
}

/// Orientation (quaternion or euler)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orientation {
    pub qx: f64,
    pub qy: f64,
    pub qz: f64,
    pub qw: f64,
}

/// An observation event within an episode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub timestamp: DateTime<Utc>,
    pub sensor: String,
    pub observation_type: String,
    pub data: serde_json::Value,
    pub confidence: Option<f64>,
}

/// An action taken within an episode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub timestamp: DateTime<Utc>,
    pub action_type: String,
    pub parameters: serde_json::Value,
    pub result: Option<String>,
    pub duration_ms: Option<i64>,
}

/// Outcome of an episode
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EpisodeOutcome {
    pub success: bool,
    pub result_type: Option<String>,
    pub result_data: Option<serde_json::Value>,
    pub error_message: Option<String>,
}

/// An episode in the robot's episodic memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotEpisode {
    pub id: Uuid,
    pub robot_id: Uuid,
    pub tenant_id: Uuid,
    
    pub episode_number: i64,
    pub episode_type: EpisodeType,
    
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    
    pub location_id: Option<String>,
    pub location_name: Option<String>,
    pub location_coordinates: Option<LocationCoordinates>,
    
    /// Natural language summary for LLM context
    pub summary: Option<String>,
    pub detailed_description: Option<String>,
    
    pub observations: Vec<Observation>,
    pub actions: Vec<Action>,
    pub outcome: Option<EpisodeOutcome>,
    
    pub objects_seen: Vec<String>,
    pub people_involved: Vec<String>,
    pub tasks_related: Vec<String>,
    
    pub confidence_score: Option<f64>,
    pub quality_score: Option<f64>,
    
    pub indexed_at: Option<DateTime<Utc>>,
    pub embedding_id: Option<String>,
    pub graph_node_id: Option<String>,
    
    pub metadata: HashMap<String, serde_json::Value>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// SEMANTIC FACT TYPES (SEMANTIC MEMORY)
// ============================================================================

/// Type of semantic fact
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FactType {
    ObjectLocation,      // "The red box is on shelf A3"
    PersonPreference,    // "User prefers items placed on the left"
    RouteEfficiency,     // "Route A is 20% faster than route B"
    Affordance,          // "This door opens by pushing"
    EnvironmentProperty, // "The warehouse is cold in the morning"
    TaskPattern,         // "Deliveries to dock B usually take 5 minutes"
    Custom(String),
}

/// A semantic fact in long-term memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotSemanticFact {
    pub id: Uuid,
    pub robot_id: Option<Uuid>, // None if global fact
    pub tenant_id: Uuid,
    
    pub fact_type: FactType,
    pub subject: String,
    pub predicate: String,
    pub object_value: Option<String>,
    
    /// Natural language representation
    pub fact_text: String,
    
    pub confidence_score: f64,
    pub evidence_count: i32,
    pub first_observed_at: DateTime<Utc>,
    pub last_observed_at: DateTime<Utc>,
    
    pub source_episode_ids: Vec<Uuid>,
    
    pub indexed_at: Option<DateTime<Utc>>,
    pub embedding_id: Option<String>,
    pub graph_node_id: Option<String>,
    
    pub metadata: HashMap<String, serde_json::Value>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// EVENT TYPES (INGESTION)
// ============================================================================

/// A raw event from a robot sensor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotEvent {
    pub source: String,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub payload: serde_json::Value,
}

/// A computer vision event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CvEvent {
    pub source: String,
    pub event_type: String, // object_detection, face_recognition, pose_estimation, etc.
    pub timestamp: DateTime<Utc>,
    pub payload: CvPayload,
}

/// Payload for CV events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CvPayload {
    ObjectDetection {
        object_class: String,
        bbox: [f64; 4], // [x1, y1, x2, y2]
        confidence: f64,
        track_id: Option<String>,
    },
    FaceRecognition {
        person_id: Option<String>,
        bbox: [f64; 4],
        confidence: f64,
        landmarks: Option<Vec<[f64; 2]>>,
    },
    PoseEstimation {
        person_id: Option<String>,
        keypoints: Vec<Keypoint>,
        confidence: f64,
    },
    Segmentation {
        class_id: i32,
        class_name: String,
        mask_rle: Option<String>, // Run-length encoded mask
        confidence: f64,
    },
    Custom(serde_json::Value),
}

/// A keypoint in pose estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keypoint {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub confidence: f64,
}

/// A frame reference (image/video)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameEvent {
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub image_uri: String, // s3://bucket/path/frame.jpg
    pub metadata: FrameMetadata,
}

/// Metadata for a frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameMetadata {
    pub width: i32,
    pub height: i32,
    pub format: Option<String>, // jpeg, png, etc.
    pub encoding: Option<String>, // rgb8, bgr8, mono8, etc.
    pub camera_info: Option<serde_json::Value>,
}

// ============================================================================
// API REQUEST/RESPONSE TYPES
// ============================================================================

/// Request to register a new robot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRobotRequest {
    pub name: String,
    pub robot_type: Option<RobotType>,
    pub description: Option<String>,
    pub capabilities: Option<Vec<RobotCapability>>,
    pub sensors: Option<Vec<SensorDeclaration>>,
    pub config: Option<HashMap<String, serde_json::Value>>,
}

/// Sensor declaration during robot registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorDeclaration {
    pub name: String,
    pub sensor_type: StreamType,
    pub sample_rate_hz: Option<f64>,
}

/// Response after registering a robot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRobotResponse {
    pub robot_id: Uuid,
    pub kafka: Option<KafkaConfig>,
    pub http_ingest: HttpEndpoints,
    pub streams: Vec<StreamInfo>,
}

/// Stream info in registration response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamInfo {
    pub stream_id: Uuid,
    pub name: String,
    pub kafka_topic: Option<String>,
}

/// Request to declare a new stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclareStreamRequest {
    pub name: String,
    pub stream_type: StreamType,
    pub description: Option<String>,
    pub schema: Option<EventSchema>,
    pub sample_rate_hz: Option<f64>,
    pub buffer_size: Option<i32>,
}

/// Response after declaring a stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclareStreamResponse {
    pub stream_id: Uuid,
    pub kafka_topic: Option<String>,
}

/// Request to search robot memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchRequest {
    pub query: String,
    pub time_range: Option<TimeRange>,
    pub filters: Option<MemoryFilters>,
    pub limit: Option<i32>,
    pub include_episodes: Option<bool>,
    pub include_facts: Option<bool>,
}

/// Time range for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

/// Filters for memory search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFilters {
    pub location: Option<String>,
    pub episode_type: Option<EpisodeType>,
    pub objects: Option<Vec<String>>,
    pub people: Option<Vec<String>>,
    pub tasks: Option<Vec<String>>,
    pub min_confidence: Option<f64>,
}

/// Response from memory search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResponse {
    pub episodes: Vec<EpisodeSummary>,
    pub facts: Vec<FactSummary>,
    pub total_count: i32,
    pub query_time_ms: i64,
}

/// Summary of an episode for search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeSummary {
    pub id: Uuid,
    pub episode_number: i64,
    pub episode_type: EpisodeType,
    pub started_at: DateTime<Utc>,
    pub summary: Option<String>,
    pub location_name: Option<String>,
    pub relevance_score: f64,
}

/// Summary of a fact for search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactSummary {
    pub id: Uuid,
    pub fact_type: FactType,
    pub fact_text: String,
    pub confidence_score: f64,
    pub relevance_score: f64,
}

/// Latest context snapshot for a robot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotContextSnapshot {
    pub robot_id: Uuid,
    pub robot_name: String,
    pub status: RobotStatus,
    pub last_heartbeat: Option<DateTime<Utc>>,
    
    pub current_location: Option<LocationCoordinates>,
    pub current_task: Option<String>,
    
    pub recent_episodes: Vec<EpisodeSummary>,
    pub relevant_facts: Vec<FactSummary>,
    
    pub active_streams: Vec<String>,
    
    pub snapshot_time: DateTime<Utc>,
}

// ============================================================================
// KAFKA MESSAGE TYPES
// ============================================================================

/// Wrapper for Kafka messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaMessage<T> {
    pub robot_id: Uuid,
    pub tenant_id: Uuid,
    pub message_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub payload: T,
}

/// Episode message for Kafka (output from Flink)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeMessage {
    pub episode_number: i64,
    pub episode_type: EpisodeType,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub summary: String,
    pub observations_count: i32,
    pub actions_count: i32,
    pub outcome: Option<EpisodeOutcome>,
    pub objects_seen: Vec<String>,
    pub location_name: Option<String>,
}

/// Semantic event message for Kafka (output from Flink)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticEventMessage {
    pub event_type: String,
    pub subject: String,
    pub predicate: String,
    pub object_value: Option<String>,
    pub natural_language: String,
    pub confidence: f64,
    pub source_episode_id: Option<Uuid>,
}
