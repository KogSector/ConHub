use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Token bucket rate limiter for controlling API request rates per data source
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
    configs: HashMap<String, RateLimitConfig>,
}

/// Configuration for rate limiting a specific data source
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed per window
    pub max_requests: u32,

    /// Time window for rate limiting
    pub window: Duration,

    /// Whether to enable automatic backoff on rate limit
    pub auto_backoff: bool,

    /// Initial backoff duration
    pub initial_backoff: Duration,

    /// Maximum backoff duration
    pub max_backoff: Duration,
}

impl RateLimitConfig {
    /// GitLab rate limit: 600 requests/minute (authenticated)
    pub fn gitlab() -> Self {
        Self {
            max_requests: 600,
            window: Duration::from_secs(60),
            auto_backoff: true,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
        }
    }

    /// Dropbox rate limit: 180 requests/minute
    pub fn dropbox() -> Self {
        Self {
            max_requests: 180,
            window: Duration::from_secs(60),
            auto_backoff: true,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(30),
        }
    }

    /// OneDrive rate limit: 20,000 requests/hour
    pub fn onedrive() -> Self {
        Self {
            max_requests: 20000,
            window: Duration::from_secs(3600),
            auto_backoff: true,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
        }
    }

    /// GitHub rate limit: 5000 requests/hour (authenticated)
    pub fn github() -> Self {
        Self {
            max_requests: 5000,
            window: Duration::from_secs(3600),
            auto_backoff: true,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(120),
        }
    }

    /// Generic conservative rate limit for unknown services
    pub fn generic() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_secs(60),
            auto_backoff: true,
            initial_backoff: Duration::from_secs(2),
            max_backoff: Duration::from_secs(30),
        }
    }
}

/// Token bucket for tracking rate limit state
#[derive(Debug, Clone)]
struct TokenBucket {
    tokens: u32,
    max_tokens: u32,
    last_refill: Instant,
    refill_rate: Duration,
    backoff_until: Option<Instant>,
    consecutive_429s: u32,
}

impl TokenBucket {
    fn new(config: &RateLimitConfig) -> Self {
        Self {
            tokens: config.max_requests,
            max_tokens: config.max_requests,
            last_refill: Instant::now(),
            refill_rate: config.window,
            backoff_until: None,
            consecutive_429s: 0,
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);

        if elapsed >= self.refill_rate {
            // Full refill
            self.tokens = self.max_tokens;
            self.last_refill = now;
        } else {
            // Partial refill based on elapsed time
            let refill_ratio = elapsed.as_secs_f64() / self.refill_rate.as_secs_f64();
            let tokens_to_add = (self.max_tokens as f64 * refill_ratio) as u32;

            if tokens_to_add > 0 {
                self.tokens = std::cmp::min(self.tokens + tokens_to_add, self.max_tokens);
                self.last_refill = now;
            }
        }
    }

    fn consume(&mut self, count: u32) -> bool {
        self.refill();

        if self.tokens >= count {
            self.tokens -= count;
            self.consecutive_429s = 0; // Reset on success
            true
        } else {
            false
        }
    }

    fn available(&mut self) -> u32 {
        self.refill();
        self.tokens
    }

    fn is_in_backoff(&self) -> bool {
        if let Some(backoff_until) = self.backoff_until {
            Instant::now() < backoff_until
        } else {
            false
        }
    }

    fn apply_backoff(&mut self, config: &RateLimitConfig) {
        self.consecutive_429s += 1;

        // Exponential backoff: initial * 2^(consecutive_429s - 1)
        let backoff_multiplier = 2u32.pow(self.consecutive_429s.saturating_sub(1));
        let backoff_duration = config.initial_backoff * backoff_multiplier;
        let capped_backoff = std::cmp::min(backoff_duration, config.max_backoff);

        self.backoff_until = Some(Instant::now() + capped_backoff);

        tracing::warn!(
            "Rate limit backoff applied: {:?} (consecutive 429s: {})",
            capped_backoff,
            self.consecutive_429s
        );
    }
}

#[derive(Debug)]
pub enum RateLimitError {
    RateLimitExceeded {
        available_in: Duration,
    },
    InBackoff {
        remaining: Duration,
    },
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitError::RateLimitExceeded { available_in } => {
                write!(f, "Rate limit exceeded. Try again in {:?}", available_in)
            }
            RateLimitError::InBackoff { remaining } => {
                write!(f, "In backoff period. Retry after {:?}", remaining)
            }
        }
    }
}

impl std::error::Error for RateLimitError {}

impl RateLimiter {
    /// Create a new rate limiter with default configurations
    pub fn new() -> Self {
        let mut configs = HashMap::new();

        configs.insert("gitlab".to_string(), RateLimitConfig::gitlab());
        configs.insert("dropbox".to_string(), RateLimitConfig::dropbox());
        configs.insert("onedrive".to_string(), RateLimitConfig::onedrive());
        configs.insert("github".to_string(), RateLimitConfig::github());
        configs.insert("bitbucket".to_string(), RateLimitConfig::github()); // Similar to GitHub
        configs.insert("googledrive".to_string(), RateLimitConfig::onedrive()); // Similar to OneDrive

        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            configs,
        }
    }

    /// Add or update a rate limit configuration for a data source
    pub fn set_config(&mut self, source_type: &str, config: RateLimitConfig) {
        self.configs.insert(source_type.to_string(), config);
    }

    /// Check if a request is allowed for a given data source
    pub fn check_rate_limit(&self, source_type: &str, source_id: &str) -> Result<(), RateLimitError> {
        let key = format!("{}:{}", source_type, source_id);

        let mut buckets = self.buckets
            .lock()
            .map_err(|_| RateLimitError::InBackoff { remaining: Duration::from_secs(1) })?;

        // Get or create bucket
        let default_config = RateLimitConfig::generic();
        let config = self.configs
            .get(source_type)
            .unwrap_or(&default_config);

        let bucket = buckets
            .entry(key.clone())
            .or_insert_with(|| TokenBucket::new(config));

        // Check backoff
        if bucket.is_in_backoff() {
            let remaining = bucket.backoff_until
                .map(|until| until.duration_since(Instant::now()))
                .unwrap_or(Duration::from_secs(0));

            return Err(RateLimitError::InBackoff { remaining });
        }

        // Try to consume token
        if !bucket.consume(1) {
            let available_in = bucket.refill_rate
                .checked_sub(Instant::now().duration_since(bucket.last_refill))
                .unwrap_or(Duration::from_secs(1));

            return Err(RateLimitError::RateLimitExceeded { available_in });
        }

        Ok(())
    }

    /// Record a 429 (Too Many Requests) response and apply backoff
    pub fn record_429(&self, source_type: &str, source_id: &str, retry_after: Option<Duration>) {
        let key = format!("{}:{}", source_type, source_id);

        if let Ok(mut buckets) = self.buckets.lock() {
            let default_config = RateLimitConfig::generic();
            let config = self.configs
                .get(source_type)
                .unwrap_or(&default_config);

            if let Some(bucket) = buckets.get_mut(&key) {
                if config.auto_backoff {
                    if let Some(retry_after) = retry_after {
                        // Use server-provided retry-after
                        bucket.backoff_until = Some(Instant::now() + retry_after);
                        tracing::warn!("Server requested backoff: {:?}", retry_after);
                    } else {
                        // Apply exponential backoff
                        bucket.apply_backoff(config);
                    }
                }
            }
        }
    }

    /// Get available tokens for a data source
    pub fn available_tokens(&self, source_type: &str, source_id: &str) -> Option<u32> {
        let key = format!("{}:{}", source_type, source_id);

        if let Ok(mut buckets) = self.buckets.lock() {
            buckets.get_mut(&key).map(|bucket| bucket.available())
        } else {
            None
        }
    }

    /// Check if approaching rate limit (>80% used)
    pub fn is_near_limit(&self, source_type: &str, source_id: &str) -> bool {
        let key = format!("{}:{}", source_type, source_id);

        if let Ok(mut buckets) = self.buckets.lock() {
            if let Some(bucket) = buckets.get_mut(&key) {
                let available = bucket.available();
                let usage_ratio = 1.0 - (available as f32 / bucket.max_tokens as f32);
                return usage_ratio > 0.8;
            }
        }

        false
    }

    /// Reset rate limit for a specific data source
    pub fn reset(&self, source_type: &str, source_id: &str) {
        let key = format!("{}:{}", source_type, source_id);

        if let Ok(mut buckets) = self.buckets.lock() {
            buckets.remove(&key);
        }
    }

    /// Get statistics for a data source
    pub fn get_stats(&self, source_type: &str, source_id: &str) -> Option<RateLimitStats> {
        let key = format!("{}:{}", source_type, source_id);

        if let Ok(mut buckets) = self.buckets.lock() {
            buckets.get_mut(&key).map(|bucket| {
                let available = bucket.available();
                RateLimitStats {
                    available_tokens: available,
                    max_tokens: bucket.max_tokens,
                    usage_percent: ((bucket.max_tokens - available) as f32 / bucket.max_tokens as f32) * 100.0,
                    in_backoff: bucket.is_in_backoff(),
                    consecutive_429s: bucket.consecutive_429s,
                }
            })
        } else {
            None
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for rate limit tracking
#[derive(Debug, Clone)]
pub struct RateLimitStats {
    pub available_tokens: u32,
    pub max_tokens: u32,
    pub usage_percent: f32,
    pub in_backoff: bool,
    pub consecutive_429s: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_configs() {
        let gitlab_config = RateLimitConfig::gitlab();
        assert_eq!(gitlab_config.max_requests, 600);
        assert_eq!(gitlab_config.window, Duration::from_secs(60));

        let dropbox_config = RateLimitConfig::dropbox();
        assert_eq!(dropbox_config.max_requests, 180);
    }

    #[test]
    fn test_token_bucket_consume() {
        let config = RateLimitConfig::generic();
        let mut bucket = TokenBucket::new(&config);

        // Should be able to consume
        assert!(bucket.consume(1));
        assert_eq!(bucket.tokens, config.max_requests - 1);

        // Consume all remaining
        assert!(bucket.consume(config.max_requests - 1));
        assert_eq!(bucket.tokens, 0);

        // Should fail when empty
        assert!(!bucket.consume(1));
    }

    #[test]
    fn test_rate_limiter_check() {
        let limiter = RateLimiter::new();

        // First request should succeed
        assert!(limiter.check_rate_limit("github", "test-source").is_ok());

        // Should have available tokens
        let available = limiter.available_tokens("github", "test-source");
        assert!(available.is_some());
        assert!(available.unwrap() < 5000); // One token consumed
    }

    #[test]
    fn test_near_limit_detection() {
        let limiter = RateLimiter::new();

        // Initially not near limit
        assert!(!limiter.is_near_limit("github", "test-source"));
    }
}
