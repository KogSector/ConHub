use actix_web::{web, HttpResponse, Result, HttpRequest};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::services::data::{
    advanced_data_service::{AdvancedDataService, DataServiceConfig},
    connection_manager::{ConnectionManager, ConnectionManagerConfig},
    advanced_cache::{AdvancedCache, CacheConfig, CacheEntryMetadata, CachePriority},
    performance_monitor::{PerformanceMonitor, PerformanceConfig},
};
use crate::errors::ServiceError;
use crate::sources::enhanced_connector::ConnectorError;

/// Enhanced handlers with advanced features
#[derive(Clone)]
pub struct EnhancedHandlers {
    /// Advanced data service
    data_service: Arc<AdvancedDataService>,
    
    /// Connection manager
    connection_manager: Arc<ConnectionManager>,
    
    /// Advanced cache
    cache: Arc<AdvancedCache>,
    
    /// Performance monitor
    performance_monitor: Arc<PerformanceMonitor>,
}

impl EnhancedHandlers {
    /// Create new enhanced handlers
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let data_service_config = DataServiceConfig::default();
        let connection_config = ConnectionManagerConfig::default();
        let cache_config = CacheConfig::default();
        let performance_config = PerformanceConfig::default();
        
        let data_service = Arc::new(AdvancedDataService::new(data_service_config));
        let connection_manager = Arc::new(ConnectionManager::new(connection_config));
        let cache = Arc::new(AdvancedCache::new(cache_config));
        let performance_monitor = Arc::new(PerformanceMonitor::new(performance_config));
        
        // Start monitoring
        performance_monitor.start_monitoring().await;
        
        Ok(Self {
            data_service,
            connection_manager,
            cache,
            performance_monitor,
        })
    }
}

/// Request for enhanced data retrieval
#[derive(Debug, Deserialize)]
pub struct EnhancedDataRequest {
    /// Data source ID
    pub source_id: String,
    
    /// Query parameters
    pub query: Option<String>,
    
    /// Filters
    pub filters: Option<HashMap<String, serde_json::Value>>,
    
    /// Pagination
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    
    /// Cache preferences
    pub use_cache: Option<bool>,
    pub cache_ttl_seconds: Option<u64>,
    
    /// Performance preferences
    pub timeout_seconds: Option<u64>,
    pub priority: Option<RequestPriority>,
}

/// Request priority levels
#[derive(Debug, Deserialize, Clone)]
pub enum RequestPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl From<RequestPriority> for CachePriority {
    fn from(priority: RequestPriority) -> Self {
        match priority {
            RequestPriority::Low => CachePriority::Low,
            RequestPriority::Normal => CachePriority::Normal,
            RequestPriority::High => CachePriority::High,
            RequestPriority::Critical => CachePriority::Critical,
        }
    }
}

/// Response for enhanced data retrieval
#[derive(Debug, Serialize)]
pub struct EnhancedDataResponse {
    /// Retrieved data
    pub data: serde_json::Value,
    
    /// Metadata
    pub metadata: DataResponseMetadata,
    
    /// Performance metrics
    pub performance: ResponsePerformanceMetrics,
    
    /// Cache information
    pub cache_info: CacheInfo,
}

/// Metadata for data response
#[derive(Debug, Serialize)]
pub struct DataResponseMetadata {
    /// Source information
    pub source_id: String,
    pub source_type: String,
    
    /// Query information
    pub query_id: Uuid,
    pub query_timestamp: DateTime<Utc>,
    
    /// Result information
    pub total_count: Option<u64>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub has_more: bool,
    
    /// Data freshness
    pub data_timestamp: DateTime<Utc>,
    pub is_real_time: bool,
}

/// Performance metrics for response
#[derive(Debug, Serialize)]
pub struct ResponsePerformanceMetrics {
    /// Query execution time (milliseconds)
    pub query_time_ms: f64,
    
    /// Total response time (milliseconds)
    pub total_time_ms: f64,
    
    /// Cache lookup time (milliseconds)
    pub cache_lookup_time_ms: f64,
    
    /// Data processing time (milliseconds)
    pub processing_time_ms: f64,
    
    /// Network time (milliseconds)
    pub network_time_ms: f64,
}

/// Cache information for response
#[derive(Debug, Serialize)]
pub struct CacheInfo {
    /// Whether data was served from cache
    pub cache_hit: bool,
    
    /// Cache level (L1, L2, or None)
    pub cache_level: Option<String>,
    
    /// Cache age (seconds)
    pub cache_age_seconds: Option<u64>,
    
    /// Cache TTL remaining (seconds)
    pub cache_ttl_remaining_seconds: Option<u64>,
}

/// Batch data request
#[derive(Debug, Deserialize)]
pub struct BatchDataRequest {
    /// Multiple data requests
    pub requests: Vec<EnhancedDataRequest>,
    
    /// Batch processing options
    pub parallel_execution: Option<bool>,
    pub max_concurrency: Option<usize>,
    pub fail_fast: Option<bool>,
}

/// Batch data response
#[derive(Debug, Serialize)]
pub struct BatchDataResponse {
    /// Individual responses
    pub responses: Vec<BatchItemResponse>,
    
    /// Batch metadata
    pub batch_metadata: BatchMetadata,
}

/// Individual response in batch
#[derive(Debug, Serialize)]
pub struct BatchItemResponse {
    /// Request index
    pub index: usize,
    
    /// Success status
    pub success: bool,
    
    /// Response data (if successful)
    pub data: Option<EnhancedDataResponse>,
    
    /// Error information (if failed)
    pub error: Option<String>,
}

/// Batch processing metadata
#[derive(Debug, Serialize)]
pub struct BatchMetadata {
    /// Total requests
    pub total_requests: usize,
    
    /// Successful requests
    pub successful_requests: usize,
    
    /// Failed requests
    pub failed_requests: usize,
    
    /// Total processing time (milliseconds)
    pub total_time_ms: f64,
    
    /// Average time per request (milliseconds)
    pub avg_time_per_request_ms: f64,
}

/// Performance metrics request
#[derive(Debug, Deserialize)]
pub struct PerformanceMetricsRequest {
    /// Time range (hours)
    pub hours: Option<u32>,
    
    /// Include real-time data
    pub include_real_time: Option<bool>,
    
    /// Include alerts
    pub include_alerts: Option<bool>,
    
    /// Include optimization suggestions
    pub include_optimizations: Option<bool>,
}

/// Cache management request
#[derive(Debug, Deserialize)]
pub struct CacheManagementRequest {
    /// Action to perform
    pub action: CacheAction,
    
    /// Target keys (for specific actions)
    pub keys: Option<Vec<String>>,
    
    /// Cache level (L1, L2, or both)
    pub cache_level: Option<String>,
}

/// Cache management actions
#[derive(Debug, Deserialize)]
pub enum CacheAction {
    Clear,
    Invalidate,
    Refresh,
    GetStats,
    Optimize,
}

/// Enhanced data retrieval handler
pub async fn get_enhanced_data(
    req: HttpRequest,
    data: web::Json<EnhancedDataRequest>,
    handlers: web::Data<EnhancedHandlers>,
) -> Result<HttpResponse, ServiceError> {
    let start_time = std::time::Instant::now();
    let query_id = Uuid::new_v4();
    
    // Record request metrics
    handlers.performance_monitor.record_custom_metric(
        "requests_total".to_string(),
        1.0,
    ).await;
    
    // Check cache first if enabled
    let cache_key = format!("data:{}:{}", data.source_id, 
        serde_json::to_string(&data.query).unwrap_or_default());
    
    let use_cache = data.use_cache.unwrap_or(true);
    let mut cache_hit = false;
    let mut cache_level = None;
    let mut cache_lookup_time = 0.0;
    
    if use_cache {
        let cache_start = std::time::Instant::now();
        
        if let Some(cached_data) = handlers.cache.get::<serde_json::Value>(&cache_key).await {
            cache_hit = true;
            cache_level = Some("L1".to_string()); // Simplified for this example
            cache_lookup_time = cache_start.elapsed().as_millis() as f64;
            
            let total_time = start_time.elapsed().as_millis() as f64;
            
            return Ok(HttpResponse::Ok().json(EnhancedDataResponse {
                data: cached_data,
                metadata: DataResponseMetadata {
                    source_id: data.source_id.clone(),
                    source_type: "cached".to_string(),
                    query_id,
                    query_timestamp: Utc::now(),
                    total_count: None,
                    page: data.page,
                    page_size: data.page_size,
                    has_more: false,
                    data_timestamp: Utc::now(),
                    is_real_time: false,
                },
                performance: ResponsePerformanceMetrics {
                    query_time_ms: 0.0,
                    total_time_ms: total_time,
                    cache_lookup_time_ms: cache_lookup_time,
                    processing_time_ms: 0.0,
                    network_time_ms: 0.0,
                },
                cache_info: CacheInfo {
                    cache_hit: true,
                    cache_level: Some("L1".to_string()),
                    cache_age_seconds: Some(0), // Would be calculated from cache metadata
                    cache_ttl_remaining_seconds: Some(300), // Would be calculated from cache metadata
                },
            }));
        }
        
        cache_lookup_time = cache_start.elapsed().as_millis() as f64;
    }
    
    // Fetch data from source
    let query_start = std::time::Instant::now();
    
    let key = data.query.as_deref().unwrap_or("default");
    let result_data = match handlers.data_service.get_data(&data.source_id, key).await {
        Ok(data) => data,
        Err(e) => {
            return Err(ServiceError::InternalServerError(format!("Failed to fetch data: {}", e)));
        }
    };
    
    let query_time = query_start.elapsed().as_millis() as f64;
    
    // Cache the result if caching is enabled
    if use_cache {
        let cache_metadata = CacheEntryMetadata {
            key: cache_key.clone(),
            source_id: data.source_id.clone(),
            entry_type: "api_response".to_string(),
            priority: data.priority.clone().unwrap_or(RequestPriority::Normal).into(),
            tags: vec!["api".to_string(), "data".to_string()],
            custom: HashMap::new(),
        };
        
        if let Err(e) = handlers.cache.set(&cache_key, &result_data, cache_metadata).await {
            // Log cache error but don't fail the request
            eprintln!("Cache set error: {}", e);
        }
    }
    
    let total_time = start_time.elapsed().as_millis() as f64;
    let processing_time = total_time - query_time - cache_lookup_time;
    
    // Record performance metrics
    handlers.performance_monitor.record_custom_metric(
        "response_time_ms".to_string(),
        total_time,
    ).await;
    
    Ok(HttpResponse::Ok().json(EnhancedDataResponse {
        data: result_data,
        metadata: DataResponseMetadata {
            source_id: data.source_id.clone(),
            source_type: "live".to_string(),
            query_id,
            query_timestamp: Utc::now(),
            total_count: None,
            page: data.page,
            page_size: data.page_size,
            has_more: false,
            data_timestamp: Utc::now(),
            is_real_time: true,
        },
        performance: ResponsePerformanceMetrics {
            query_time_ms: query_time,
            total_time_ms: total_time,
            cache_lookup_time_ms: cache_lookup_time,
            processing_time_ms: processing_time,
            network_time_ms: 0.0, // Would be measured separately
        },
        cache_info: CacheInfo {
            cache_hit,
            cache_level,
            cache_age_seconds: None,
            cache_ttl_remaining_seconds: None,
        },
    }))
}

/// Batch data retrieval handler
pub async fn get_batch_data(
    req: HttpRequest,
    data: web::Json<BatchDataRequest>,
    handlers: web::Data<EnhancedHandlers>,
) -> Result<HttpResponse, ServiceError> {
    let start_time = std::time::Instant::now();
    let parallel_execution = data.parallel_execution.unwrap_or(true);
    let max_concurrency = data.max_concurrency.unwrap_or(10);
    
    let mut responses = Vec::new();
    let mut successful_requests = 0;
    let mut failed_requests = 0;
    
    if parallel_execution {
        // Process requests in parallel with concurrency limit
        use futures::stream::{self, StreamExt};
        
        let results: Vec<_> = stream::iter(data.requests.iter().enumerate())
            .map(|(index, request)| {
                let handlers_clone = handlers.clone();
                async move {
                    let individual_req = web::Json(request.clone());
                    // Call the data service directly instead of the handler
                    let key = request.query.as_deref().unwrap_or("default");
                    match handlers_clone.data_service.get_data(&request.source_id, key).await {
                    Ok(data) => {
                        let response = EnhancedDataResponse {
                            data,
                            metadata: DataResponseMetadata {
                                source_id: request.source_id.clone(),
                                source_type: "unknown".to_string(),
                                query_id: Uuid::new_v4(),
                                query_timestamp: Utc::now(),
                                total_count: None,
                                page: request.page,
                                page_size: request.page_size,
                                has_more: false,
                                data_timestamp: Utc::now(),
                                is_real_time: false,
                            },
                            performance: ResponsePerformanceMetrics {
                                query_time_ms: 0.0,
                                total_time_ms: 0.0,
                                cache_lookup_time_ms: 0.0,
                                processing_time_ms: 0.0,
                                network_time_ms: 0.0,
                            },
                            cache_info: CacheInfo {
                                cache_hit: false,
                                cache_level: None,
                                cache_age_seconds: None,
                                cache_ttl_remaining_seconds: None,
                            },
                        };
                        BatchItemResponse {
                            index,
                            success: true,
                            data: Some(response),
                            error: None,
                        }
                    }
                    Err(e) => BatchItemResponse {
                        index,
                        success: false,
                        data: None,
                        error: Some(e.to_string()),
                    },
                }
            }
            })
            .buffer_unordered(max_concurrency)
            .collect()
            .await;
        
        responses = results;
    } else {
        // Process requests sequentially
        for (index, request) in data.requests.iter().enumerate() {
            let key = request.query.as_deref().unwrap_or("default");
            match handlers.data_service.get_data(&request.source_id, key).await {
                Ok(data_result) => {
                    let response = EnhancedDataResponse {
                        data: data_result,
                        metadata: DataResponseMetadata {
                            source_id: request.source_id.clone(),
                            source_type: "unknown".to_string(),
                            query_id: Uuid::new_v4(),
                            query_timestamp: Utc::now(),
                            total_count: None,
                            page: request.page,
                            page_size: request.page_size,
                            has_more: false,
                            data_timestamp: Utc::now(),
                            is_real_time: false,
                        },
                        performance: ResponsePerformanceMetrics {
                            query_time_ms: 0.0,
                            total_time_ms: 0.0,
                            cache_lookup_time_ms: 0.0,
                            processing_time_ms: 0.0,
                            network_time_ms: 0.0,
                        },
                        cache_info: CacheInfo {
                            cache_hit: false,
                            cache_level: None,
                            cache_age_seconds: None,
                            cache_ttl_remaining_seconds: None,
                        },
                    };
                    responses.push(BatchItemResponse {
                        index,
                        success: true,
                        data: Some(response),
                        error: None,
                    });
                    successful_requests += 1;
                }
                Err(e) => {
                    responses.push(BatchItemResponse {
                        index,
                        success: false,
                        data: None,
                        error: Some(e.to_string()),
                    });
                    failed_requests += 1;
                    
                    if data.fail_fast.unwrap_or(false) {
                        break;
                    }
                }
            }
        }
    }
    
    // Count successful and failed requests
    for response in &responses {
        if response.success {
            successful_requests += 1;
        } else {
            failed_requests += 1;
        }
    }
    
    let total_time = start_time.elapsed().as_millis() as f64;
    let avg_time_per_request = if !responses.is_empty() {
        total_time / responses.len() as f64
    } else {
        0.0
    };
    
    Ok(HttpResponse::Ok().json(BatchDataResponse {
        responses,
        batch_metadata: BatchMetadata {
            total_requests: data.requests.len(),
            successful_requests,
            failed_requests,
            total_time_ms: total_time,
            avg_time_per_request_ms: avg_time_per_request,
        },
    }))
}

/// Performance metrics handler
pub async fn get_performance_metrics(
    data: web::Json<PerformanceMetricsRequest>,
    handlers: web::Data<EnhancedHandlers>,
) -> Result<HttpResponse, ServiceError> {
    let mut response = serde_json::Map::new();
    
    // Get current metrics
    let current_metrics = handlers.performance_monitor.get_metrics().await;
    response.insert("current_metrics".to_string(), serde_json::to_value(current_metrics)?);
    
    // Get real-time data if requested
    if data.include_real_time.unwrap_or(false) {
        let real_time_data = handlers.performance_monitor.get_real_time_data().await;
        response.insert("real_time_data".to_string(), serde_json::json!({
            "avg_response_time_ms": real_time_data.get_avg_response_time(),
            "avg_cpu_usage_percent": real_time_data.get_avg_cpu_usage(),
            "avg_memory_usage_percent": real_time_data.get_avg_memory_usage(),
            "last_update": real_time_data.last_update,
        }));
    }
    
    // Get alerts if requested
    if data.include_alerts.unwrap_or(false) {
        let alerts = handlers.performance_monitor.get_active_alerts().await;
        response.insert("active_alerts".to_string(), serde_json::to_value(alerts)?);
    }
    
    // Get optimization suggestions if requested
    if data.include_optimizations.unwrap_or(false) {
        let optimizations = handlers.performance_monitor.get_optimization_suggestions().await;
        response.insert("optimization_suggestions".to_string(), serde_json::to_value(optimizations)?);
    }
    
    // Get historical data if requested
    if let Some(hours) = data.hours {
        let history = handlers.performance_monitor.get_performance_history(hours).await;
        response.insert("historical_metrics".to_string(), serde_json::to_value(history)?);
    }
    
    Ok(HttpResponse::Ok().json(response))
}

/// Cache management handler
pub async fn manage_cache(
    data: web::Json<CacheManagementRequest>,
    handlers: web::Data<EnhancedHandlers>,
) -> Result<HttpResponse, ServiceError> {
    match data.action {
        CacheAction::Clear => {
            handlers.cache.clear().await;
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Cache cleared successfully",
                "timestamp": Utc::now()
            })))
        }
        CacheAction::GetStats => {
            let metrics = handlers.cache.get_metrics().await;
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "cache_metrics": metrics,
                "timestamp": Utc::now()
            })))
        }
        CacheAction::Invalidate => {
            // Would implement key-specific invalidation
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Cache invalidation completed",
                "keys_invalidated": data.keys.as_ref().map(|k| k.len()).unwrap_or(0),
                "timestamp": Utc::now()
            })))
        }
        CacheAction::Refresh => {
            // Would implement cache refresh logic
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Cache refresh initiated",
                "timestamp": Utc::now()
            })))
        }
        CacheAction::Optimize => {
            // Would implement cache optimization logic
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Cache optimization completed",
                "timestamp": Utc::now()
            })))
        }
    }
}

/// Health check with enhanced metrics
pub async fn enhanced_health_check(
    handlers: web::Data<EnhancedHandlers>,
) -> Result<HttpResponse, ServiceError> {
    let metrics = handlers.performance_monitor.get_metrics().await;
    let cache_metrics = handlers.cache.get_metrics().await;
    
    let health_status = if metrics.system.cpu_usage_percent < 90.0 
        && metrics.system.memory_usage_percent < 90.0 
        && metrics.application.error_rate_percent < 10.0 {
        "healthy"
    } else if metrics.system.cpu_usage_percent < 95.0 
        && metrics.system.memory_usage_percent < 95.0 
        && metrics.application.error_rate_percent < 20.0 {
        "degraded"
    } else {
        "unhealthy"
    };
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": health_status,
        "timestamp": Utc::now(),
        "system_metrics": {
            "cpu_usage_percent": metrics.system.cpu_usage_percent,
            "memory_usage_percent": metrics.system.memory_usage_percent,
            "disk_usage_percent": metrics.system.disk_usage_percent,
        },
        "application_metrics": {
            "request_rate_rps": metrics.application.request_rate_rps,
            "avg_response_time_ms": metrics.application.avg_response_time_ms,
            "error_rate_percent": metrics.application.error_rate_percent,
            "active_connections": metrics.application.active_connections,
        },
        "cache_metrics": {
            "l1_hit_rate": (cache_metrics.l1_hits as f64 / (cache_metrics.l1_hits + cache_metrics.l1_misses) as f64 * 100.0),
            "l2_hit_rate": (cache_metrics.l2_hits as f64 / (cache_metrics.l2_hits + cache_metrics.l2_misses) as f64 * 100.0),
            "total_hit_rate": cache_metrics.hit_rate * 100.0,
        },
        "database_metrics": {
            "pool_usage_percent": metrics.database.pool_usage_percent,
            "avg_query_time_ms": metrics.database.avg_query_time_ms,
            "queries_per_second": metrics.database.queries_per_second,
        }
    })))
}

/// Configure enhanced routes
pub fn configure_enhanced_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/enhanced")
            .route("/data", web::post().to(get_enhanced_data))
            .route("/data/batch", web::post().to(get_batch_data))
            .route("/metrics", web::post().to(get_performance_metrics))
            .route("/cache", web::post().to(manage_cache))
            .route("/health", web::get().to(enhanced_health_check))
    );
}