//! Kafka client for robot event ingestion
//! 
//! This module provides a Kafka producer client for publishing robot events
//! to Apache Kafka topics. It supports:
//! - Raw sensor events
//! - Computer vision events
//! - Frame references
//! - Control messages

use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use uuid::Uuid;
use chrono::Utc;

/// Kafka producer configuration
#[derive(Debug, Clone)]
pub struct KafkaProducerConfig {
    pub bootstrap_servers: String,
    pub client_id: String,
    pub acks: String,
    pub retries: i32,
    pub batch_size: i32,
    pub linger_ms: i32,
    pub compression_type: String,
    pub sasl_mechanism: Option<String>,
    pub sasl_username: Option<String>,
    pub sasl_password: Option<String>,
}

impl Default for KafkaProducerConfig {
    fn default() -> Self {
        Self {
            bootstrap_servers: std::env::var("KAFKA_BOOTSTRAP_SERVERS")
                .unwrap_or_else(|_| "localhost:9092".to_string()),
            client_id: "conhub-data-service".to_string(),
            acks: "all".to_string(),
            retries: 3,
            batch_size: 16384,
            linger_ms: 5,
            compression_type: "lz4".to_string(),
            sasl_mechanism: std::env::var("KAFKA_SASL_MECHANISM").ok(),
            sasl_username: std::env::var("KAFKA_SASL_USERNAME").ok(),
            sasl_password: std::env::var("KAFKA_SASL_PASSWORD").ok(),
        }
    }
}

/// Result type for Kafka operations
pub type KafkaResult<T> = Result<T, KafkaError>;

/// Kafka error types
#[derive(Debug, thiserror::Error)]
pub enum KafkaError {
    #[error("Kafka connection error: {0}")]
    ConnectionError(String),
    
    #[error("Kafka producer error: {0}")]
    ProducerError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Topic not found: {0}")]
    TopicNotFound(String),
    
    #[error("Kafka not enabled")]
    NotEnabled,
}

/// Message metadata returned after successful publish
#[derive(Debug, Clone)]
pub struct MessageMetadata {
    pub topic: String,
    pub partition: i32,
    pub offset: i64,
    pub timestamp: i64,
}

/// Kafka producer client
/// 
/// This is a placeholder implementation that can be swapped with a real
/// Kafka client (e.g., rdkafka) when Apache Kafka is deployed.
pub struct KafkaProducer {
    config: KafkaProducerConfig,
    enabled: bool,
    /// In-memory buffer for when Kafka is not available (dev mode)
    buffer: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl KafkaProducer {
    /// Create a new Kafka producer
    pub fn new(config: KafkaProducerConfig) -> Self {
        let enabled = std::env::var("KAFKA_ENABLED")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);
        
        if enabled {
            info!("🚀 Kafka producer initialized with servers: {}", config.bootstrap_servers);
        } else {
            warn!("⚠️ Kafka disabled - using in-memory buffer for development");
        }
        
        Self {
            config,
            enabled,
            buffer: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create a producer from environment variables
    pub fn from_env() -> Self {
        Self::new(KafkaProducerConfig::default())
    }
    
    /// Check if Kafka is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Publish a message to a topic
    pub async fn publish<T: Serialize>(
        &self,
        topic: &str,
        key: Option<&str>,
        message: &T,
    ) -> KafkaResult<MessageMetadata> {
        let json = serde_json::to_string(message)
            .map_err(|e| KafkaError::SerializationError(e.to_string()))?;
        
        if self.enabled {
            // TODO: Replace with actual rdkafka producer call
            // For now, we simulate success
            self.publish_to_kafka(topic, key, &json).await
        } else {
            // Buffer locally for development
            self.buffer_message(topic, &json).await
        }
    }
    
    /// Publish a batch of messages
    pub async fn publish_batch<T: Serialize>(
        &self,
        topic: &str,
        messages: &[(Option<String>, T)],
    ) -> KafkaResult<Vec<MessageMetadata>> {
        let mut results = Vec::with_capacity(messages.len());
        
        for (key, message) in messages {
            let result = self.publish(topic, key.as_deref(), message).await?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Internal: publish to actual Kafka
    async fn publish_to_kafka(
        &self,
        topic: &str,
        _key: Option<&str>,
        json: &str,
    ) -> KafkaResult<MessageMetadata> {
        // TODO: Implement actual Kafka producer using rdkafka
        // 
        // Example with rdkafka:
        // ```
        // use rdkafka::producer::{FutureProducer, FutureRecord};
        // use rdkafka::ClientConfig;
        // 
        // let producer: FutureProducer = ClientConfig::new()
        //     .set("bootstrap.servers", &self.config.bootstrap_servers)
        //     .set("message.timeout.ms", "5000")
        //     .create()
        //     .expect("Producer creation failed");
        // 
        // let record = FutureRecord::to(topic)
        //     .payload(json)
        //     .key(key.unwrap_or(""));
        // 
        // let delivery_status = producer.send(record, Duration::from_secs(5)).await;
        // ```
        
        info!("📤 [Kafka] Publishing to topic '{}': {} bytes", topic, json.len());
        
        // Simulate successful publish
        Ok(MessageMetadata {
            topic: topic.to_string(),
            partition: 0,
            offset: Utc::now().timestamp_millis(),
            timestamp: Utc::now().timestamp_millis(),
        })
    }
    
    /// Internal: buffer message locally (dev mode)
    async fn buffer_message(&self, topic: &str, json: &str) -> KafkaResult<MessageMetadata> {
        let mut buffer = self.buffer.write().await;
        
        buffer
            .entry(topic.to_string())
            .or_insert_with(Vec::new)
            .push(json.to_string());
        
        let offset = buffer.get(topic).map(|v| v.len() as i64).unwrap_or(0);
        
        info!("📦 [Buffer] Stored message for topic '{}' (offset: {})", topic, offset);
        
        Ok(MessageMetadata {
            topic: topic.to_string(),
            partition: 0,
            offset,
            timestamp: Utc::now().timestamp_millis(),
        })
    }
    
    /// Get buffered messages for a topic (dev mode only)
    pub async fn get_buffered_messages(&self, topic: &str) -> Vec<String> {
        let buffer = self.buffer.read().await;
        buffer.get(topic).cloned().unwrap_or_default()
    }
    
    /// Clear buffered messages for a topic (dev mode only)
    pub async fn clear_buffer(&self, topic: Option<&str>) {
        let mut buffer = self.buffer.write().await;
        
        if let Some(topic) = topic {
            buffer.remove(topic);
        } else {
            buffer.clear();
        }
    }
}

/// Generate Kafka topic names for a robot
pub fn generate_robot_topics(robot_id: &Uuid) -> HashMap<String, String> {
    let prefix = format!("robot.{}", robot_id);
    
    let mut topics = HashMap::new();
    topics.insert("raw_events".to_string(), format!("{}.raw_events", prefix));
    topics.insert("cv_events".to_string(), format!("{}.cv_events", prefix));
    topics.insert("frames".to_string(), format!("{}.frames", prefix));
    topics.insert("episodes".to_string(), format!("{}.episodes", prefix));
    topics.insert("semantic_events".to_string(), format!("{}.semantic_events", prefix));
    topics.insert("control".to_string(), format!("{}.control", prefix));
    topics.insert("anomalies".to_string(), format!("{}.anomalies", prefix));
    
    topics
}

/// Generate a Kafka topic name for a specific stream
pub fn generate_stream_topic(robot_id: &Uuid, stream_name: &str) -> String {
    format!("robot.{}.stream.{}", robot_id, stream_name.to_lowercase().replace(' ', "_"))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_kafka_producer_buffer_mode() {
        let producer = KafkaProducer::from_env();
        
        // Should be disabled by default in tests
        assert!(!producer.is_enabled());
        
        // Publish a test message
        let message = serde_json::json!({"test": "data"});
        let result = producer.publish("test.topic", Some("key1"), &message).await;
        
        assert!(result.is_ok());
        
        // Check buffer
        let buffered = producer.get_buffered_messages("test.topic").await;
        assert_eq!(buffered.len(), 1);
    }
    
    #[test]
    fn test_generate_robot_topics() {
        let robot_id = Uuid::new_v4();
        let topics = generate_robot_topics(&robot_id);
        
        assert!(topics.contains_key("raw_events"));
        assert!(topics.contains_key("cv_events"));
        assert!(topics.contains_key("episodes"));
        
        let raw_topic = topics.get("raw_events").unwrap();
        assert!(raw_topic.starts_with("robot."));
        assert!(raw_topic.ends_with(".raw_events"));
    }
}
