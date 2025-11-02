use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{mpsc, RwLock};
use aws_config;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_sqs::Client as SqsClient;
use aws_config::meta::region::RegionProviderChain;
use aws_types::region::Region;

/// Enhanced S3 source specification with SQS event support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedS3Spec {
    /// S3 bucket name
    pub bucket: String,
    
    /// S3 key prefix filter
    pub prefix: Option<String>,
    
    /// S3 key suffix filter
    pub suffix: Option<String>,
    
    /// AWS region
    pub region: String,
    
    /// AWS credentials configuration
    pub credentials: AwsCredentialsConfig,
    
    /// SQS queue configuration for event notifications
    pub sqs_config: Option<SqsConfig>,
    
    /// S3 event notification configuration
    pub event_config: S3EventConfig,
    
    /// File processing configuration
    pub file_processing: FileProcessingConfig,
    
    /// Batch processing settings
    pub batch_config: BatchConfig,
    
    /// Error handling and retry configuration
    pub error_handling: ErrorHandlingConfig,
    
    /// Performance optimization settings
    pub performance: PerformanceConfig,
}

impl Default for EnhancedS3Spec {
    fn default() -> Self {
        Self {
            bucket: String::new(),
            prefix: None,
            suffix: None,
            region: "us-east-1".to_string(),
            credentials: AwsCredentialsConfig::default(),
            sqs_config: None,
            event_config: S3EventConfig::default(),
            file_processing: FileProcessingConfig::default(),
            batch_config: BatchConfig::default(),
            error_handling: ErrorHandlingConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

/// AWS credentials configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsCredentialsConfig {
    /// Access key ID (optional, can use IAM roles)
    pub access_key_id: Option<String>,
    
    /// Secret access key (optional, can use IAM roles)
    pub secret_access_key: Option<String>,
    
    /// Session token for temporary credentials
    pub session_token: Option<String>,
    
    /// AWS profile name
    pub profile: Option<String>,
    
    /// Use IAM instance profile
    pub use_instance_profile: bool,
    
    /// Custom endpoint URL (for S3-compatible services)
    pub endpoint_url: Option<String>,
    
    /// Force path-style addressing
    pub force_path_style: bool,
}

impl Default for AwsCredentialsConfig {
    fn default() -> Self {
        Self {
            access_key_id: None,
            secret_access_key: None,
            session_token: None,
            profile: None,
            use_instance_profile: true,
            endpoint_url: None,
            force_path_style: false,
        }
    }
}

/// SQS queue configuration for S3 event notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqsConfig {
    /// SQS queue URL
    pub queue_url: String,
    
    /// Maximum number of messages to receive per batch
    pub max_messages: i32,
    
    /// Message visibility timeout
    pub visibility_timeout: Duration,
    
    /// Wait time for long polling
    pub wait_time: Duration,
    
    /// Message retention period
    pub message_retention: Duration,
    
    /// Dead letter queue configuration
    pub dead_letter_queue: Option<DeadLetterQueueConfig>,
    
    /// Enable message deduplication
    pub enable_deduplication: bool,
    
    /// Message group ID for FIFO queues
    pub message_group_id: Option<String>,
}

impl Default for SqsConfig {
    fn default() -> Self {
        Self {
            queue_url: String::new(),
            max_messages: 10,
            visibility_timeout: Duration::from_secs(300),
            wait_time: Duration::from_secs(20),
            message_retention: Duration::from_secs(1209600), // 14 days
            dead_letter_queue: None,
            enable_deduplication: false,
            message_group_id: None,
        }
    }
}

/// Dead letter queue configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterQueueConfig {
    /// Dead letter queue URL
    pub queue_url: String,
    
    /// Maximum receive count before moving to DLQ
    pub max_receive_count: i32,
}

/// S3 event notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3EventConfig {
    /// Event types to process
    pub event_types: Vec<S3EventType>,
    
    /// Enable event filtering by object key patterns
    pub key_filters: Vec<KeyFilter>,
    
    /// Enable event deduplication
    pub enable_deduplication: bool,
    
    /// Deduplication window
    pub deduplication_window: Duration,
    
    /// Event processing timeout
    pub processing_timeout: Duration,
    
    /// Enable event ordering
    pub enable_ordering: bool,
    
    /// Maximum event age to process
    pub max_event_age: Duration,
}

impl Default for S3EventConfig {
    fn default() -> Self {
        Self {
            event_types: vec![
                S3EventType::ObjectCreated,
                S3EventType::ObjectRemoved,
            ],
            key_filters: Vec::new(),
            enable_deduplication: true,
            deduplication_window: Duration::from_secs(300),
            processing_timeout: Duration::from_secs(60),
            enable_ordering: false,
            max_event_age: Duration::from_secs(3600),
        }
    }
}

/// S3 event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum S3EventType {
    ObjectCreated,
    ObjectRemoved,
    ObjectRestore,
    ReducedRedundancyLostObject,
    Replication,
    LifecycleTransition,
    LifecycleExpiration,
    IntelligentTiering,
}

/// Key filter for S3 events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyFilter {
    /// Filter type
    pub filter_type: KeyFilterType,
    
    /// Filter value
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyFilterType {
    Prefix,
    Suffix,
    Regex,
    Exact,
}

/// File processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileProcessingConfig {
    /// Maximum file size to process (in bytes)
    pub max_file_size: u64,
    
    /// Minimum file size to process (in bytes)
    pub min_file_size: u64,
    
    /// Supported file extensions
    pub allowed_extensions: Vec<String>,
    
    /// Blocked file extensions
    pub blocked_extensions: Vec<String>,
    
    /// Enable MIME type detection
    pub enable_mime_detection: bool,
    
    /// Supported MIME types
    pub allowed_mime_types: Vec<String>,
    
    /// Enable content preprocessing
    pub enable_preprocessing: bool,
    
    /// Preprocessing configuration
    pub preprocessing: PreprocessingConfig,
    
    /// Enable file metadata extraction
    pub extract_metadata: bool,
    
    /// Metadata extraction configuration
    pub metadata_config: MetadataConfig,
}

impl Default for FileProcessingConfig {
    fn default() -> Self {
        Self {
            max_file_size: 100 * 1024 * 1024, // 100MB
            min_file_size: 0,
            allowed_extensions: Vec::new(),
            blocked_extensions: vec![".exe".to_string(), ".dll".to_string()],
            enable_mime_detection: true,
            allowed_mime_types: Vec::new(),
            enable_preprocessing: false,
            preprocessing: PreprocessingConfig::default(),
            extract_metadata: true,
            metadata_config: MetadataConfig::default(),
        }
    }
}

/// Preprocessing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessingConfig {
    /// Enable text extraction from documents
    pub extract_text: bool,
    
    /// Enable image processing
    pub process_images: bool,
    
    /// Enable archive extraction
    pub extract_archives: bool,
    
    /// Maximum extraction depth for nested archives
    pub max_extraction_depth: u32,
    
    /// Enable virus scanning
    pub enable_virus_scan: bool,
    
    /// Custom preprocessing pipeline
    pub custom_pipeline: Vec<PreprocessingStep>,
}

impl Default for PreprocessingConfig {
    fn default() -> Self {
        Self {
            extract_text: true,
            process_images: false,
            extract_archives: false,
            max_extraction_depth: 3,
            enable_virus_scan: false,
            custom_pipeline: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessingStep {
    /// Step name
    pub name: String,
    
    /// Step type
    pub step_type: String,
    
    /// Step configuration
    pub config: HashMap<String, String>,
    
    /// Enable step
    pub enabled: bool,
}

/// Metadata extraction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataConfig {
    /// Extract file system metadata
    pub extract_fs_metadata: bool,
    
    /// Extract EXIF data from images
    pub extract_exif: bool,
    
    /// Extract document properties
    pub extract_document_props: bool,
    
    /// Extract custom metadata
    pub extract_custom: bool,
    
    /// Custom metadata extractors
    pub custom_extractors: Vec<MetadataExtractor>,
}

impl Default for MetadataConfig {
    fn default() -> Self {
        Self {
            extract_fs_metadata: true,
            extract_exif: false,
            extract_document_props: true,
            extract_custom: false,
            custom_extractors: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataExtractor {
    /// Extractor name
    pub name: String,
    
    /// File patterns to apply to
    pub patterns: Vec<String>,
    
    /// Extraction configuration
    pub config: HashMap<String, String>,
}

/// Batch processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Batch size for processing files
    pub batch_size: usize,
    
    /// Maximum batch processing time
    pub batch_timeout: Duration,
    
    /// Enable parallel processing within batches
    pub enable_parallel: bool,
    
    /// Maximum parallel workers
    pub max_parallel_workers: usize,
    
    /// Enable batch compression
    pub enable_compression: bool,
    
    /// Compression algorithm
    pub compression_algorithm: CompressionAlgorithm,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            batch_timeout: Duration::from_secs(300),
            enable_parallel: true,
            max_parallel_workers: 4,
            enable_compression: false,
            compression_algorithm: CompressionAlgorithm::Gzip,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    Gzip,
    Zstd,
    Lz4,
    Brotli,
}

/// Error handling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandlingConfig {
    /// Maximum retry attempts
    pub max_retries: u32,
    
    /// Base retry delay
    pub base_retry_delay: Duration,
    
    /// Maximum retry delay
    pub max_retry_delay: Duration,
    
    /// Retry backoff multiplier
    pub backoff_multiplier: f64,
    
    /// Enable circuit breaker
    pub enable_circuit_breaker: bool,
    
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
    
    /// Error categorization
    pub error_categories: HashMap<String, ErrorCategory>,
    
    /// Enable error reporting
    pub enable_error_reporting: bool,
    
    /// Error reporting configuration
    pub error_reporting: ErrorReportingConfig,
}

impl Default for ErrorHandlingConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_retry_delay: Duration::from_millis(100),
            max_retry_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            enable_circuit_breaker: true,
            circuit_breaker: CircuitBreakerConfig::default(),
            error_categories: HashMap::new(),
            enable_error_reporting: true,
            error_reporting: ErrorReportingConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open circuit
    pub failure_threshold: u32,
    
    /// Success threshold to close circuit
    pub success_threshold: u32,
    
    /// Timeout before attempting to close circuit
    pub timeout: Duration,
    
    /// Maximum number of half-open requests
    pub max_half_open_requests: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            max_half_open_requests: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCategory {
    /// Category name
    pub name: String,
    
    /// Error patterns
    pub patterns: Vec<String>,
    
    /// Retry strategy
    pub retry_strategy: RetryStrategy,
    
    /// Alert configuration
    pub alert_config: Option<AlertConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryStrategy {
    Immediate,
    ExponentialBackoff,
    LinearBackoff,
    NoRetry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Alert threshold
    pub threshold: u32,
    
    /// Alert channels
    pub channels: Vec<String>,
    
    /// Alert message template
    pub message_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReportingConfig {
    /// Enable detailed error logging
    pub detailed_logging: bool,
    
    /// Enable error metrics collection
    pub collect_metrics: bool,
    
    /// Error aggregation window
    pub aggregation_window: Duration,
    
    /// Enable error sampling
    pub enable_sampling: bool,
    
    /// Sampling rate (0.0 to 1.0)
    pub sampling_rate: f64,
}

impl Default for ErrorReportingConfig {
    fn default() -> Self {
        Self {
            detailed_logging: true,
            collect_metrics: true,
            aggregation_window: Duration::from_secs(300),
            enable_sampling: false,
            sampling_rate: 0.1,
        }
    }
}

/// Performance optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable connection pooling
    pub enable_connection_pooling: bool,
    
    /// Connection pool size
    pub connection_pool_size: usize,
    
    /// Connection timeout
    pub connection_timeout: Duration,
    
    /// Request timeout
    pub request_timeout: Duration,
    
    /// Enable request pipelining
    pub enable_pipelining: bool,
    
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
    
    /// Enable caching
    pub enable_caching: bool,
    
    /// Cache configuration
    pub cache_config: CacheConfig,
    
    /// Enable compression
    pub enable_compression: bool,
    
    /// Compression threshold
    pub compression_threshold: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_connection_pooling: true,
            connection_pool_size: 10,
            connection_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(300),
            enable_pipelining: false,
            max_concurrent_requests: 100,
            enable_caching: true,
            cache_config: CacheConfig::default(),
            enable_compression: true,
            compression_threshold: 1024,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Cache size limit
    pub size_limit: usize,
    
    /// Cache TTL
    pub ttl: Duration,
    
    /// Enable LRU eviction
    pub enable_lru: bool,
    
    /// Cache hit ratio threshold for warnings
    pub hit_ratio_threshold: f64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            size_limit: 1000,
            ttl: Duration::from_secs(3600),
            enable_lru: true,
            hit_ratio_threshold: 0.8,
        }
    }
}

/// S3 event message from SQS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3EventMessage {
    /// SQS message ID
    pub message_id: String,
    
    /// SQS receipt handle
    pub receipt_handle: String,
    
    /// Message body (S3 event notification)
    pub body: String,
    
    /// Message attributes
    pub attributes: HashMap<String, String>,
    
    /// Parsed S3 events
    pub events: Vec<S3Event>,
    
    /// Message timestamp
    pub timestamp: SystemTime,
}

/// Individual S3 event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Event {
    /// Event name
    pub event_name: String,
    
    /// Event source
    pub event_source: String,
    
    /// Event time
    pub event_time: SystemTime,
    
    /// S3 bucket information
    pub s3_bucket: S3BucketInfo,
    
    /// S3 object information
    pub s3_object: S3ObjectInfo,
    
    /// Request parameters
    pub request_parameters: HashMap<String, String>,
    
    /// Response elements
    pub response_elements: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3BucketInfo {
    /// Bucket name
    pub name: String,
    
    /// Bucket ARN
    pub arn: String,
    
    /// Owner identity
    pub owner_identity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3ObjectInfo {
    /// Object key
    pub key: String,
    
    /// Object size
    pub size: Option<u64>,
    
    /// Object ETag
    pub etag: Option<String>,
    
    /// Object version ID
    pub version_id: Option<String>,
    
    /// Object sequencer
    pub sequencer: Option<String>,
}

/// Enhanced S3 executor with SQS event support
pub struct EnhancedS3Executor {
    spec: EnhancedS3Spec,
    s3_client: Arc<RwLock<Option<S3Client>>>,
    sqs_client: Arc<RwLock<Option<SqsClient>>>,
    event_tx: Option<mpsc::UnboundedSender<S3EventMessage>>,
    stats: Arc<RwLock<S3Stats>>,
    circuit_breaker: Arc<RwLock<CircuitBreakerState>>,
}

/// Statistics for S3 operations
#[derive(Debug, Clone, Default)]
pub struct S3Stats {
    /// Total objects processed
    pub objects_processed: u64,
    
    /// Total bytes processed
    pub bytes_processed: u64,
    
    /// Total SQS messages processed
    pub sqs_messages_processed: u64,
    
    /// Total S3 events processed
    pub s3_events_processed: u64,
    
    /// Error count
    pub error_count: u64,
    
    /// Average processing time
    pub avg_processing_time: Duration,
    
    /// Last successful operation
    pub last_success: Option<SystemTime>,
    
    /// Cache hit ratio
    pub cache_hit_ratio: f64,
    
    /// Current circuit breaker state
    pub circuit_breaker_state: String,
}

/// Circuit breaker state
#[derive(Debug, Clone)]
pub struct CircuitBreakerState {
    /// Current state
    pub state: CircuitState,
    
    /// Failure count
    pub failure_count: u32,
    
    /// Success count
    pub success_count: u32,
    
    /// Last failure time
    pub last_failure: Option<SystemTime>,
    
    /// Half-open requests count
    pub half_open_requests: u32,
}

#[derive(Debug, Clone)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

impl Default for CircuitBreakerState {
    fn default() -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure: None,
            half_open_requests: 0,
        }
    }
}

impl EnhancedS3Executor {
    /// Create a new enhanced S3 executor
    pub fn new(spec: EnhancedS3Spec) -> Self {
        Self {
            spec,
            s3_client: Arc::new(RwLock::new(None)),
            sqs_client: Arc::new(RwLock::new(None)),
            event_tx: None,
            stats: Arc::new(RwLock::new(S3Stats::default())),
            circuit_breaker: Arc::new(RwLock::new(CircuitBreakerState::default())),
        }
    }
    
    /// Initialize the executor
    pub async fn initialize(&mut self) -> Result<()> {
        // Initialize AWS configuration
        let region_provider = RegionProviderChain::default_provider()
            .or_else(Region::new(self.spec.region.clone()));
        
        let config = aws_config::from_env()
            .region(region_provider)
            .load()
            .await;
        
        // Initialize S3 client
        let s3_client = S3Client::new(&config);
        
        let mut s3_client_guard = self.s3_client.write().await;
        *s3_client_guard = Some(s3_client);
        drop(s3_client_guard);
        
        // Initialize SQS client if configured
        if let Some(sqs_config) = &self.spec.sqs_config {
            let sqs_client = SqsClient::new(&config);
            
            let mut sqs_client_guard = self.sqs_client.write().await;
            *sqs_client_guard = Some(sqs_client);
            drop(sqs_client_guard);
            
            // Start SQS event listener
            self.start_sqs_listener().await?;
        }
        
        Ok(())
    }
    
    /// Start SQS event listener
    async fn start_sqs_listener(&mut self) -> Result<()> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.event_tx = Some(tx.clone());
        
        let sqs_client = self.sqs_client.clone();
        let sqs_config = self.spec.sqs_config.clone().unwrap();
        let stats = self.stats.clone();
        
        tokio::spawn(async move {
            if let Err(e) = Self::sqs_event_loop(sqs_client, sqs_config, tx, stats).await {
                log::error!("SQS event loop error: {}", e);
            }
        });
        
        Ok(())
    }
    
    /// SQS event processing loop
    async fn sqs_event_loop(
        sqs_client: Arc<RwLock<Option<SqsClient>>>,
        sqs_config: SqsConfig,
        tx: mpsc::UnboundedSender<S3EventMessage>,
        stats: Arc<RwLock<S3Stats>>,
    ) -> Result<()> {
        loop {
            let client_guard = sqs_client.read().await;
            let client = client_guard.as_ref()
                .ok_or_else(|| anyhow::anyhow!("SQS client not initialized"))?;
            
            // Receive messages from SQS
            let receive_result = client
                .receive_message()
                .queue_url(&sqs_config.queue_url)
                .max_number_of_messages(sqs_config.max_messages)
                .visibility_timeout(sqs_config.visibility_timeout.as_secs() as i32)
                .wait_time_seconds(sqs_config.wait_time.as_secs() as i32)
                .send()
                .await;
            
            match receive_result {
                Ok(output) => {
                    if let Some(messages) = output.messages {
                        for message in messages {
                            if let Some(body) = message.body {
                                let event_message = S3EventMessage {
                                    message_id: message.message_id.unwrap_or_default(),
                                    receipt_handle: message.receipt_handle.unwrap_or_default(),
                                    body: body.clone(),
                                    attributes: message.attributes.unwrap_or_default(),
                                    events: Self::parse_s3_events(&body)?,
                                    timestamp: SystemTime::now(),
                                };
                                
                                if let Err(e) = tx.send(event_message) {
                                    log::error!("Failed to send S3 event message: {}", e);
                                }
                                
                                // Update statistics
                                let mut stats_guard = stats.write().await;
                                stats_guard.sqs_messages_processed += 1;
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("SQS receive error: {}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }
    
    /// Parse S3 events from SQS message body
    fn parse_s3_events(body: &str) -> Result<Vec<S3Event>> {
        // Implementation would parse the JSON message body
        // For now, this is a placeholder
        Ok(Vec::new())
    }
    
    /// Process S3 object
    pub async fn process_object(&self, bucket: &str, key: &str) -> Result<Vec<u8>> {
        let s3_client_guard = self.s3_client.read().await;
        let client = s3_client_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("S3 client not initialized"))?;
        
        // Check circuit breaker
        if !self.check_circuit_breaker().await? {
            return Err(anyhow::anyhow!("Circuit breaker is open"));
        }
        
        let start_time = std::time::Instant::now();
        
        // Get object from S3
        let result = client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await;
        
        match result {
            Ok(output) => {
                let body = output.body.collect().await?;
                let data = body.into_bytes().to_vec();
                
                // Update statistics
                let mut stats = self.stats.write().await;
                stats.objects_processed += 1;
                stats.bytes_processed += data.len() as u64;
                stats.avg_processing_time = Duration::from_nanos(
                    (stats.avg_processing_time.as_nanos() as u64 + start_time.elapsed().as_nanos() as u64) / 2
                );
                stats.last_success = Some(SystemTime::now());
                
                // Record success in circuit breaker
                self.record_success().await;
                
                Ok(data)
            }
            Err(e) => {
                // Record failure in circuit breaker
                self.record_failure().await;
                
                let mut stats = self.stats.write().await;
                stats.error_count += 1;
                
                Err(anyhow::anyhow!("S3 get object error: {}", e))
            }
        }
    }
    
    /// Check circuit breaker state
    async fn check_circuit_breaker(&self) -> Result<bool> {
        let mut breaker = self.circuit_breaker.write().await;
        let config = &self.spec.error_handling.circuit_breaker;
        
        match breaker.state {
            CircuitState::Closed => Ok(true),
            CircuitState::Open => {
                if let Some(last_failure) = breaker.last_failure {
                    if last_failure.elapsed().unwrap_or(Duration::ZERO) > config.timeout {
                        breaker.state = CircuitState::HalfOpen;
                        breaker.half_open_requests = 0;
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
            CircuitState::HalfOpen => {
                if breaker.half_open_requests < config.max_half_open_requests {
                    breaker.half_open_requests += 1;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }
    
    /// Record successful operation
    async fn record_success(&self) {
        let mut breaker = self.circuit_breaker.write().await;
        let config = &self.spec.error_handling.circuit_breaker;
        
        breaker.success_count += 1;
        breaker.failure_count = 0;
        
        match breaker.state {
            CircuitState::HalfOpen => {
                if breaker.success_count >= config.success_threshold {
                    breaker.state = CircuitState::Closed;
                    breaker.success_count = 0;
                    breaker.half_open_requests = 0;
                }
            }
            _ => {}
        }
    }
    
    /// Record failed operation
    async fn record_failure(&self) {
        let mut breaker = self.circuit_breaker.write().await;
        let config = &self.spec.error_handling.circuit_breaker;
        
        breaker.failure_count += 1;
        breaker.success_count = 0;
        breaker.last_failure = Some(SystemTime::now());
        
        match breaker.state {
            CircuitState::Closed | CircuitState::HalfOpen => {
                if breaker.failure_count >= config.failure_threshold {
                    breaker.state = CircuitState::Open;
                    breaker.half_open_requests = 0;
                }
            }
            _ => {}
        }
    }
    
    /// Get current statistics
    pub async fn get_stats(&self) -> S3Stats {
        self.stats.read().await.clone()
    }
}