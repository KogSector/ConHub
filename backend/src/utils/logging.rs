use tracing::{info, warn, debug, Level};
use tracing_subscriber::{
    fmt::{self, time::ChronoUtc},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Registry,
};
use tracing_appender::{non_blocking, rolling};
use std::env;
use std::path::PathBuf;
use sysinfo::System;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Logging configuration for the ConHub application
pub struct LoggingConfig {
    pub level: Level,
    pub json_format: bool,
    pub file_logging: bool,
    pub log_directory: PathBuf,
    pub service_name: String,
    #[allow(dead_code)]
    pub enable_performance_logging: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            json_format: false,
            file_logging: true,
            log_directory: PathBuf::from("logs"),
            service_name: "conhub-backend".to_string(),
            enable_performance_logging: true,
        }
    }
}

impl LoggingConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        // Set log level from environment
        if let Ok(level_str) = env::var("RUST_LOG_LEVEL") {
            config.level = match level_str.to_lowercase().as_str() {
                "trace" => Level::TRACE,
                "debug" => Level::DEBUG,
                "info" => Level::INFO,
                "warn" => Level::WARN,
                "error" => Level::ERROR,
                _ => Level::INFO,
            };
        }
        
        // Check if we're in production
        if env::var("NODE_ENV").unwrap_or_default() == "production" {
            config.json_format = true;
            config.level = Level::INFO; // More conservative in production
        }
        
        // Check if we're in development
        if env::var("NODE_ENV").unwrap_or_default() == "development" {
            config.level = Level::DEBUG;
        }
        
        // Service name from environment
        if let Ok(service) = env::var("SERVICE_NAME") {
            config.service_name = service;
        }
        
        config
    }
}

/// Initialize comprehensive logging for ConHub
pub fn init_logging(config: LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Create logs directory if it doesn't exist
    std::fs::create_dir_all(&config.log_directory)?;
    
    // Create file appender for all logs
    let file_appender = rolling::daily(&config.log_directory, "conhub.log");
    let (non_blocking_file, _guard) = non_blocking(file_appender);
    
    // Create error-specific file appender
    let error_file_appender = rolling::daily(&config.log_directory, "conhub-errors.log");
    let (_non_blocking_error, _error_guard) = non_blocking(error_file_appender);
    
    // Base filter for environment
    let env_filter = EnvFilter::new(format!("{}={}", config.service_name.replace("-", "_"), config.level));
    
    let registry = Registry::default().with(env_filter);
    
    if config.json_format {
        // Structured JSON logging for production
        let json_layer = fmt::layer()
            .json()
            .with_current_span(true)
            .with_span_list(true)
            .with_timer(ChronoUtc::rfc_3339())
            .with_target(true)
            .with_level(true)
            .with_file(true)
            .with_line_number(true)
            .with_writer(non_blocking_file);
        
        registry.with(json_layer).init();
    } else {
        // Pretty console logging for development
        let console_layer = fmt::layer()
            .pretty()
            .with_timer(ChronoUtc::rfc_3339())
            .with_target(true)
            .with_level(true)
            .with_file(true)
            .with_line_number(true);
        
        let file_layer = fmt::layer()
            .with_timer(ChronoUtc::rfc_3339())
            .with_target(true)
            .with_level(true)
            .with_file(true)
            .with_line_number(true)
            .with_writer(non_blocking_file);
        
        registry
            .with(console_layer)
            .with(file_layer)
            .init();
    }
    
    info!(
        service = %config.service_name,
        log_level = %config.level,
        json_format = config.json_format,
        file_logging = config.file_logging,
        "Logging initialized"
    );
    
    Ok(())
}

/// Performance monitoring and logging
#[derive(Clone)]
pub struct PerformanceMonitor {
    system: Arc<RwLock<System>>,
    request_metrics: Arc<RwLock<HashMap<String, RequestMetrics>>>,
}

#[derive(Debug, Clone)]
pub struct RequestMetrics {
    pub count: u64,
    pub total_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub errors: u64,
}

impl Default for RequestMetrics {
    fn default() -> Self {
        Self {
            count: 0,
            total_duration: Duration::from_millis(0),
            min_duration: Duration::from_secs(u64::MAX),
            max_duration: Duration::from_millis(0),
            errors: 0,
        }
    }
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            system: Arc::new(RwLock::new(System::new_all())),
            request_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Log system performance metrics
    pub async fn log_system_metrics(&self) {
        let mut system = self.system.write().await;
        system.refresh_all();
        
        let cpu_usage: f32 = system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / system.cpus().len() as f32;
        let memory_usage = system.used_memory();
        let total_memory = system.total_memory();
        let memory_percentage = (memory_usage as f64 / total_memory as f64) * 100.0;
        
        info!(
            cpu_usage_percent = %format!("{:.2}", cpu_usage),
            memory_used_mb = memory_usage / 1024 / 1024,
            memory_total_mb = total_memory / 1024 / 1024,
            memory_usage_percent = %format!("{:.2}", memory_percentage),
            "System performance metrics"
        );
    }
    
    /// Record request metrics
    #[allow(dead_code)]
    pub async fn record_request(&self, endpoint: &str, duration: Duration, is_error: bool) {
        let mut metrics = self.request_metrics.write().await;
        let endpoint_metrics = metrics.entry(endpoint.to_string()).or_default();
        
        endpoint_metrics.count += 1;
        endpoint_metrics.total_duration += duration;
        
        if duration < endpoint_metrics.min_duration {
            endpoint_metrics.min_duration = duration;
        }
        
        if duration > endpoint_metrics.max_duration {
            endpoint_metrics.max_duration = duration;
        }
        
        if is_error {
            endpoint_metrics.errors += 1;
        }
        
        // Log slow requests (> 1 second)
        if duration > Duration::from_secs(1) {
            warn!(
                endpoint = endpoint,
                duration_ms = duration.as_millis(),
                "Slow request detected"
            );
        }
        
        // Log request details
        debug!(
            endpoint = endpoint,
            duration_ms = duration.as_millis(),
            is_error = is_error,
            "Request completed"
        );
    }
    
    /// Log aggregated request metrics
    pub async fn log_request_metrics(&self) {
        let metrics = self.request_metrics.read().await;
        
        for (endpoint, metric) in metrics.iter() {
            if metric.count > 0 {
                let avg_duration = metric.total_duration / metric.count as u32;
                let error_rate = (metric.errors as f64 / metric.count as f64) * 100.0;
                
                info!(
                    endpoint = endpoint,
                    request_count = metric.count,
                    avg_duration_ms = avg_duration.as_millis(),
                    min_duration_ms = metric.min_duration.as_millis(),
                    max_duration_ms = metric.max_duration.as_millis(),
                    error_count = metric.errors,
                    error_rate_percent = %format!("{:.2}", error_rate),
                    "Request metrics summary"
                );
            }
        }
    }
    
    /// Start periodic monitoring
    pub async fn start_monitoring(&self) {
        let monitor = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60)); // Log every minute
            
            loop {
                interval.tick().await;
                monitor.log_system_metrics().await;
                monitor.log_request_metrics().await;
            }
        });
    }
}

/// Request timing middleware helper
#[allow(dead_code)]
pub struct RequestTimer {
    pub start_time: Instant,
    pub endpoint: String,
}

#[allow(dead_code)]
impl RequestTimer {
    pub fn new(endpoint: String) -> Self {
        Self {
            start_time: Instant::now(),
            endpoint,
        }
    }
    
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

/// Log application startup information
pub fn log_startup_info() {
    let version = env!("CARGO_PKG_VERSION");
    let build_target = env::consts::ARCH;
    let os = env::consts::OS;
    
    info!(
        version = version,
        arch = build_target,
        os = os,
        pid = std::process::id(),
        "ConHub Backend starting"
    );
}

/// Log configuration information
pub fn log_config_info() {
    let env_name = env::var("NODE_ENV").unwrap_or_else(|_| "development".to_string());
    let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    
    info!(
        environment = env_name,
        rust_log_level = rust_log,
        "Configuration loaded"
    );
}