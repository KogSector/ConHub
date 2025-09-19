use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, atomic::{AtomicU64, AtomicUsize, Ordering}};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, Mutex};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use uuid::Uuid;

/// Core performance metrics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: u64,
    pub memory_usage_percent: f64,
    pub disk_io_read_bytes: u64,
    pub disk_io_write_bytes: u64,
    pub network_in_bytes: u64,
    pub network_out_bytes: u64,
    pub active_connections: usize,
    pub request_count: u64,
    pub error_count: u64,
    pub avg_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
}

/// Application-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationMetrics {
    pub timestamp: DateTime<Utc>,
    pub database_query_count: u64,
    pub database_avg_query_time_ms: f64,
    pub cache_hit_rate: f64,
    pub cache_miss_rate: f64,
    pub active_sessions: usize,
    pub authentication_failures: u64,
    pub rate_limit_violations: u64,
    pub search_queries_count: u64,
    pub search_avg_time_ms: f64,
    pub background_tasks_active: usize,
    pub background_tasks_completed: u64,
    pub background_tasks_failed: u64,
}

/// Custom metric entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMetric {
    pub name: String,
    pub value: f64,
    pub tags: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub metric_name: String,
    pub condition: AlertCondition,
    pub threshold: f64,
    pub duration_seconds: u64,
    pub severity: AlertSeverity,
    pub enabled: bool,
    pub actions: Vec<AlertAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    GreaterThan,
    LessThan,
    Equals,
    NotEquals,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertAction {
    Email { recipients: Vec<String> },
    Webhook { url: String },
    Log { level: String },
}

/// Active alert instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub rule_id: String,
    pub triggered_at: DateTime<Utc>,
    pub current_value: f64,
    pub threshold: f64,
    pub severity: AlertSeverity,
    pub message: String,
    pub acknowledged: bool,
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
}

/// Time-series data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
}

/// Response time tracker
#[derive(Debug)]
pub struct ResponseTimeTracker {
    samples: VecDeque<f64>,
    max_samples: usize,
}

impl ResponseTimeTracker {
    pub fn new(max_samples: usize) -> Self {
        Self {
            samples: VecDeque::new(),
            max_samples,
        }
    }

    pub fn add_sample(&mut self, response_time_ms: f64) {
        self.samples.push_back(response_time_ms);
        if self.samples.len() > self.max_samples {
            self.samples.pop_front();
        }
    }

    pub fn avg(&self) -> f64 {
        if self.samples.is_empty() {
            0.0
        } else {
            self.samples.iter().sum::<f64>() / self.samples.len() as f64
        }
    }

    pub fn percentile(&self, p: f64) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }

        let mut sorted: Vec<f64> = self.samples.iter().cloned().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let index = (p / 100.0 * (sorted.len() - 1) as f64).round() as usize;
        sorted[index.min(sorted.len() - 1)]
    }
}

/// Main performance monitoring service
pub struct PerformanceMonitoringService {
    // Atomic counters for high-frequency metrics
    request_count: AtomicU64,
    error_count: AtomicU64,
    authentication_failures: AtomicU64,
    rate_limit_violations: AtomicU64,
    active_connections: AtomicUsize,
    active_sessions: AtomicUsize,
    
    // Time-series storage
    performance_history: Arc<RwLock<VecDeque<PerformanceMetrics>>>,
    application_history: Arc<RwLock<VecDeque<ApplicationMetrics>>>,
    custom_metrics: Arc<DashMap<String, VecDeque<TimeSeriesPoint>>>,
    
    // Response time tracking
    response_times: Arc<Mutex<ResponseTimeTracker>>,
    database_query_times: Arc<Mutex<ResponseTimeTracker>>,
    search_query_times: Arc<Mutex<ResponseTimeTracker>>,
    
    // Alerting system
    alert_rules: Arc<RwLock<HashMap<String, AlertRule>>>,
    active_alerts: Arc<RwLock<HashMap<String, Alert>>>,
    alert_history: Arc<RwLock<VecDeque<Alert>>>,
    
    // Configuration
    max_history_size: usize,
    collection_interval: Duration,
    
    // Background task handles
    collection_task_handle: Option<tokio::task::JoinHandle<()>>,
    alert_task_handle: Option<tokio::task::JoinHandle<()>>,
}

impl PerformanceMonitoringService {
    pub fn new(max_history_size: usize, collection_interval: Duration) -> Self {
        Self {
            request_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            authentication_failures: AtomicU64::new(0),
            rate_limit_violations: AtomicU64::new(0),
            active_connections: AtomicUsize::new(0),
            active_sessions: AtomicUsize::new(0),
            performance_history: Arc::new(RwLock::new(VecDeque::new())),
            application_history: Arc::new(RwLock::new(VecDeque::new())),
            custom_metrics: Arc::new(DashMap::new()),
            response_times: Arc::new(Mutex::new(ResponseTimeTracker::new(1000))),
            database_query_times: Arc::new(Mutex::new(ResponseTimeTracker::new(1000))),
            search_query_times: Arc::new(Mutex::new(ResponseTimeTracker::new(1000))),
            alert_rules: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(VecDeque::new())),
            max_history_size,
            collection_interval,
            collection_task_handle: None,
            alert_task_handle: None,
        }
    }

    /// Start background monitoring tasks
    pub async fn start(&mut self) {
        // Start metrics collection task
        let service_clone = self.clone_for_task();
        self.collection_task_handle = Some(tokio::spawn(async move {
            service_clone.metrics_collection_task().await;
        }));

        // Start alerting task
        let service_clone = self.clone_for_task();
        self.alert_task_handle = Some(tokio::spawn(async move {
            service_clone.alerting_task().await;
        }));

        log::info!("Performance monitoring service started");
    }

    /// Stop background monitoring tasks
    pub async fn stop(&mut self) {
        if let Some(handle) = self.collection_task_handle.take() {
            handle.abort();
        }
        if let Some(handle) = self.alert_task_handle.take() {
            handle.abort();
        }
        log::info!("Performance monitoring service stopped");
    }

    /// Increment request counter
    pub fn increment_requests(&self) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment error counter
    pub fn increment_errors(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Record response time
    pub async fn record_response_time(&self, response_time_ms: f64) {
        let mut tracker = self.response_times.lock().await;
        tracker.add_sample(response_time_ms);
    }

    /// Record database query time
    pub async fn record_database_query_time(&self, query_time_ms: f64) {
        let mut tracker = self.database_query_times.lock().await;
        tracker.add_sample(query_time_ms);
    }

    /// Record search query time
    pub async fn record_search_query_time(&self, query_time_ms: f64) {
        let mut tracker = self.search_query_times.lock().await;
        tracker.add_sample(query_time_ms);
    }

    /// Record custom metric
    pub fn record_custom_metric(&self, name: String, value: f64, tags: HashMap<String, String>) {
        let point = TimeSeriesPoint {
            timestamp: Utc::now(),
            value,
        };

        let mut metrics = self.custom_metrics.entry(name).or_insert_with(VecDeque::new);
        metrics.push_back(point);
        
        // Keep only recent data
        if metrics.len() > self.max_history_size {
            metrics.pop_front();
        }
    }

    /// Set active connections count
    pub fn set_active_connections(&self, count: usize) {
        self.active_connections.store(count, Ordering::Relaxed);
    }

    /// Set active sessions count
    pub fn set_active_sessions(&self, count: usize) {
        self.active_sessions.store(count, Ordering::Relaxed);
    }

    /// Add alert rule
    pub async fn add_alert_rule(&self, rule: AlertRule) {
        let mut rules = self.alert_rules.write().await;
        rules.insert(rule.id.clone(), rule);
    }

    /// Remove alert rule
    pub async fn remove_alert_rule(&self, rule_id: &str) {
        let mut rules = self.alert_rules.write().await;
        rules.remove(rule_id);
    }

    /// Get current performance metrics
    pub async fn get_current_metrics(&self) -> PerformanceMetrics {
        let response_tracker = self.response_times.lock().await;
        
        PerformanceMetrics {
            timestamp: Utc::now(),
            cpu_usage_percent: self.get_cpu_usage().await,
            memory_usage_bytes: self.get_memory_usage().await,
            memory_usage_percent: self.get_memory_usage_percent().await,
            disk_io_read_bytes: 0, // Would integrate with system metrics
            disk_io_write_bytes: 0,
            network_in_bytes: 0,
            network_out_bytes: 0,
            active_connections: self.active_connections.load(Ordering::Relaxed),
            request_count: self.request_count.load(Ordering::Relaxed),
            error_count: self.error_count.load(Ordering::Relaxed),
            avg_response_time_ms: response_tracker.avg(),
            p95_response_time_ms: response_tracker.percentile(95.0),
            p99_response_time_ms: response_tracker.percentile(99.0),
        }
    }

    /// Get current application metrics
    pub async fn get_current_application_metrics(&self) -> ApplicationMetrics {
        let db_tracker = self.database_query_times.lock().await;
        let search_tracker = self.search_query_times.lock().await;
        
        ApplicationMetrics {
            timestamp: Utc::now(),
            database_query_count: 0, // Would integrate with database service
            database_avg_query_time_ms: db_tracker.avg(),
            cache_hit_rate: 0.0, // Would integrate with cache service
            cache_miss_rate: 0.0,
            active_sessions: self.active_sessions.load(Ordering::Relaxed),
            authentication_failures: self.authentication_failures.load(Ordering::Relaxed),
            rate_limit_violations: self.rate_limit_violations.load(Ordering::Relaxed),
            search_queries_count: search_tracker.samples.len() as u64,
            search_avg_time_ms: search_tracker.avg(),
            background_tasks_active: 0, // Would integrate with task system
            background_tasks_completed: 0,
            background_tasks_failed: 0,
        }
    }

    /// Get performance history
    pub async fn get_performance_history(&self, limit: Option<usize>) -> Vec<PerformanceMetrics> {
        let history = self.performance_history.read().await;
        let limit = limit.unwrap_or(history.len());
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Get application metrics history
    pub async fn get_application_history(&self, limit: Option<usize>) -> Vec<ApplicationMetrics> {
        let history = self.application_history.read().await;
        let limit = limit.unwrap_or(history.len());
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Get custom metric history
    pub fn get_custom_metric_history(&self, metric_name: &str, limit: Option<usize>) -> Option<Vec<TimeSeriesPoint>> {
        self.custom_metrics.get(metric_name).map(|entry| {
            let limit = limit.unwrap_or(entry.len());
            entry.iter().rev().take(limit).cloned().collect()
        })
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let alerts = self.active_alerts.read().await;
        alerts.values().cloned().collect()
    }

    /// Acknowledge alert
    pub async fn acknowledge_alert(&self, alert_id: &str) -> bool {
        let mut alerts = self.active_alerts.write().await;
        if let Some(alert) = alerts.get_mut(alert_id) {
            alert.acknowledged = true;
            true
        } else {
            false
        }
    }

    /// Get system health summary
    pub async fn get_health_summary(&self) -> HashMap<String, String> {
        let mut summary = HashMap::new();
        
        let current_metrics = self.get_current_metrics().await;
        let app_metrics = self.get_current_application_metrics().await;
        
        // Overall health assessment
        let health_score = self.calculate_health_score(&current_metrics, &app_metrics).await;
        summary.insert("health_score".to_string(), format!("{:.2}", health_score));
        
        if health_score > 0.8 {
            summary.insert("status".to_string(), "healthy".to_string());
        } else if health_score > 0.6 {
            summary.insert("status".to_string(), "warning".to_string());
        } else {
            summary.insert("status".to_string(), "critical".to_string());
        }
        
        summary.insert("cpu_usage".to_string(), format!("{:.2}%", current_metrics.cpu_usage_percent));
        summary.insert("memory_usage".to_string(), format!("{:.2}%", current_metrics.memory_usage_percent));
        summary.insert("avg_response_time".to_string(), format!("{:.2}ms", current_metrics.avg_response_time_ms));
        summary.insert("active_connections".to_string(), current_metrics.active_connections.to_string());
        summary.insert("error_rate".to_string(), format!("{:.2}%", self.calculate_error_rate(&current_metrics)));
        
        summary
    }

    // Background task for metrics collection
    async fn metrics_collection_task(&self) {
        let mut interval = tokio::time::interval(self.collection_interval);
        
        loop {
            interval.tick().await;
            
            // Collect performance metrics
            let perf_metrics = self.get_current_metrics().await;
            {
                let mut history = self.performance_history.write().await;
                history.push_back(perf_metrics);
                if history.len() > self.max_history_size {
                    history.pop_front();
                }
            }
            
            // Collect application metrics
            let app_metrics = self.get_current_application_metrics().await;
            {
                let mut history = self.application_history.write().await;
                history.push_back(app_metrics);
                if history.len() > self.max_history_size {
                    history.pop_front();
                }
            }
        }
    }

    // Background task for alerting
    async fn alerting_task(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(30)); // Check every 30 seconds
        
        loop {
            interval.tick().await;
            self.check_alerts().await;
        }
    }

    async fn check_alerts(&self) {
        let rules = self.alert_rules.read().await;
        let current_metrics = self.get_current_metrics().await;
        let app_metrics = self.get_current_application_metrics().await;
        
        for rule in rules.values() {
            if !rule.enabled {
                continue;
            }
            
            let current_value = self.get_metric_value(&rule.metric_name, &current_metrics, &app_metrics);
            
            if self.evaluate_condition(&rule.condition, current_value, rule.threshold) {
                self.trigger_alert(rule.clone(), current_value).await;
            } else {
                self.resolve_alert(&rule.id).await;
            }
        }
    }

    fn get_metric_value(&self, metric_name: &str, perf: &PerformanceMetrics, app: &ApplicationMetrics) -> f64 {
        match metric_name {
            "cpu_usage_percent" => perf.cpu_usage_percent,
            "memory_usage_percent" => perf.memory_usage_percent,
            "avg_response_time_ms" => perf.avg_response_time_ms,
            "error_rate" => self.calculate_error_rate(perf),
            "database_avg_query_time_ms" => app.database_avg_query_time_ms,
            "cache_hit_rate" => app.cache_hit_rate,
            _ => 0.0,
        }
    }

    fn evaluate_condition(&self, condition: &AlertCondition, current: f64, threshold: f64) -> bool {
        match condition {
            AlertCondition::GreaterThan => current > threshold,
            AlertCondition::LessThan => current < threshold,
            AlertCondition::Equals => (current - threshold).abs() < f64::EPSILON,
            AlertCondition::NotEquals => (current - threshold).abs() > f64::EPSILON,
        }
    }

    async fn trigger_alert(&self, rule: AlertRule, current_value: f64) {
        let alert_id = format!("alert_{}", Uuid::new_v4());
        let alert = Alert {
            id: alert_id.clone(),
            rule_id: rule.id.clone(),
            triggered_at: Utc::now(),
            current_value,
            threshold: rule.threshold,
            severity: rule.severity.clone(),
            message: format!("Alert: {} - Current value: {:.2}, Threshold: {:.2}", 
                           rule.name, current_value, rule.threshold),
            acknowledged: false,
            resolved: false,
            resolved_at: None,
        };
        
        // Add to active alerts
        {
            let mut active_alerts = self.active_alerts.write().await;
            active_alerts.insert(alert_id.clone(), alert.clone());
        }
        
        // Execute alert actions
        for action in &rule.actions {
            self.execute_alert_action(action, &alert).await;
        }
        
        log::warn!("Alert triggered: {}", alert.message);
    }

    async fn resolve_alert(&self, rule_id: &str) {
        let mut active_alerts = self.active_alerts.write().await;
        let mut resolved_alerts = Vec::new();
        
        active_alerts.retain(|_, alert| {
            if alert.rule_id == rule_id && !alert.resolved {
                let mut resolved_alert = alert.clone();
                resolved_alert.resolved = true;
                resolved_alert.resolved_at = Some(Utc::now());
                resolved_alerts.push(resolved_alert);
                false // Remove from active alerts
            } else {
                true // Keep in active alerts
            }
        });
        
        // Add to alert history
        if !resolved_alerts.is_empty() {
            let mut history = self.alert_history.write().await;
            for alert in resolved_alerts {
                log::info!("Alert resolved: {}", alert.message);
                history.push_back(alert);
                if history.len() > self.max_history_size {
                    history.pop_front();
                }
            }
        }
    }

    async fn execute_alert_action(&self, action: &AlertAction, alert: &Alert) {
        match action {
            AlertAction::Email { recipients: _ } => {
                // Would integrate with email service
                log::info!("Email alert action for: {}", alert.message);
            }
            AlertAction::Webhook { url: _ } => {
                // Would make HTTP request to webhook
                log::info!("Webhook alert action for: {}", alert.message);
            }
            AlertAction::Log { level } => {
                match level.as_str() {
                    "error" => log::error!("Alert: {}", alert.message),
                    "warn" => log::warn!("Alert: {}", alert.message),
                    _ => log::info!("Alert: {}", alert.message),
                }
            }
        }
    }

    fn calculate_error_rate(&self, metrics: &PerformanceMetrics) -> f64 {
        if metrics.request_count == 0 {
            0.0
        } else {
            (metrics.error_count as f64 / metrics.request_count as f64) * 100.0
        }
    }

    async fn calculate_health_score(&self, perf: &PerformanceMetrics, app: &ApplicationMetrics) -> f64 {
        let mut score = 1.0;
        
        // CPU usage impact
        if perf.cpu_usage_percent > 80.0 {
            score -= 0.3;
        } else if perf.cpu_usage_percent > 60.0 {
            score -= 0.1;
        }
        
        // Memory usage impact
        if perf.memory_usage_percent > 85.0 {
            score -= 0.3;
        } else if perf.memory_usage_percent > 70.0 {
            score -= 0.1;
        }
        
        // Error rate impact
        let error_rate = self.calculate_error_rate(perf);
        if error_rate > 5.0 {
            score -= 0.4;
        } else if error_rate > 1.0 {
            score -= 0.2;
        }
        
        // Response time impact
        if perf.avg_response_time_ms > 2000.0 {
            score -= 0.2;
        } else if perf.avg_response_time_ms > 1000.0 {
            score -= 0.1;
        }
        
        score.max(0.0)
    }

    // System metrics collection methods (simplified - would integrate with system libraries)
    async fn get_cpu_usage(&self) -> f64 {
        // Placeholder - would use system metrics library
        rand::random::<f64>() * 100.0
    }

    async fn get_memory_usage(&self) -> u64 {
        // Placeholder - would use system metrics library
        1024 * 1024 * 512 // 512 MB
    }

    async fn get_memory_usage_percent(&self) -> f64 {
        // Placeholder - would use system metrics library
        rand::random::<f64>() * 100.0
    }

    // Helper method for creating service clone for tasks
    fn clone_for_task(&self) -> PerformanceMonitoringServiceClone {
        PerformanceMonitoringServiceClone {
            performance_history: Arc::clone(&self.performance_history),
            application_history: Arc::clone(&self.application_history),
            alert_rules: Arc::clone(&self.alert_rules),
            active_alerts: Arc::clone(&self.active_alerts),
            alert_history: Arc::clone(&self.alert_history),
            response_times: Arc::clone(&self.response_times),
            database_query_times: Arc::clone(&self.database_query_times),
            search_query_times: Arc::clone(&self.search_query_times),
            max_history_size: self.max_history_size,
            collection_interval: self.collection_interval,
        }
    }
}

// Helper struct for background tasks
#[derive(Clone)]
struct PerformanceMonitoringServiceClone {
    performance_history: Arc<RwLock<VecDeque<PerformanceMetrics>>>,
    application_history: Arc<RwLock<VecDeque<ApplicationMetrics>>>,
    alert_rules: Arc<RwLock<HashMap<String, AlertRule>>>,
    active_alerts: Arc<RwLock<HashMap<String, Alert>>>,
    alert_history: Arc<RwLock<VecDeque<Alert>>>,
    response_times: Arc<Mutex<ResponseTimeTracker>>,
    database_query_times: Arc<Mutex<ResponseTimeTracker>>,
    search_query_times: Arc<Mutex<ResponseTimeTracker>>,
    max_history_size: usize,
    collection_interval: Duration,
}

impl PerformanceMonitoringServiceClone {
    async fn metrics_collection_task(&self) {
        // Implementation would mirror the main service's method
    }

    async fn alerting_task(&self) {
        // Implementation would mirror the main service's method
    }
}

/// HTTP handlers for performance monitoring API
pub mod handlers {
    use super::*;
    use actix_web::{web, HttpResponse, Result as ActixResult};

    pub async fn get_current_metrics(
        monitoring: web::Data<Arc<PerformanceMonitoringService>>,
    ) -> ActixResult<HttpResponse> {
        let metrics = monitoring.get_current_metrics().await;
        Ok(HttpResponse::Ok().json(metrics))
    }

    pub async fn get_health_summary(
        monitoring: web::Data<Arc<PerformanceMonitoringService>>,
    ) -> ActixResult<HttpResponse> {
        let summary = monitoring.get_health_summary().await;
        Ok(HttpResponse::Ok().json(summary))
    }

    pub async fn get_performance_history(
        monitoring: web::Data<Arc<PerformanceMonitoringService>>,
        query: web::Query<HashMap<String, String>>,
    ) -> ActixResult<HttpResponse> {
        let limit = query.get("limit")
            .and_then(|l| l.parse().ok());
        
        let history = monitoring.get_performance_history(limit).await;
        Ok(HttpResponse::Ok().json(history))
    }

    pub async fn get_active_alerts(
        monitoring: web::Data<Arc<PerformanceMonitoringService>>,
    ) -> ActixResult<HttpResponse> {
        let alerts = monitoring.get_active_alerts().await;
        Ok(HttpResponse::Ok().json(alerts))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_monitoring() {
        let mut service = PerformanceMonitoringService::new(100, Duration::from_secs(1));
        
        // Test metric recording
        service.increment_requests();
        service.record_response_time(150.0).await;
        
        let metrics = service.get_current_metrics().await;
        assert_eq!(metrics.request_count, 1);
        
        // Test health summary
        let summary = service.get_health_summary().await;
        assert!(summary.contains_key("health_score"));
    }

    #[test]
    fn test_response_time_tracker() {
        let mut tracker = ResponseTimeTracker::new(5);
        
        tracker.add_sample(100.0);
        tracker.add_sample(200.0);
        tracker.add_sample(150.0);
        
        assert_eq!(tracker.avg(), 150.0);
        assert_eq!(tracker.percentile(50.0), 150.0);
    }
}