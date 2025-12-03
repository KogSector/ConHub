//! Convenience macros for common logging patterns in ConHub.
//!
//! These macros provide consistent logging with trace context and domain events.

/// Log a function entry with arguments (for debugging)
#[macro_export]
macro_rules! log_fn_entry {
    ($fn_name:expr) => {
        tracing::debug!(target: "function", fn_name = $fn_name, "→ entering");
    };
    ($fn_name:expr, $($key:ident = $value:expr),* $(,)?) => {
        tracing::debug!(target: "function", fn_name = $fn_name, $($key = ?$value),*, "→ entering");
    };
}

/// Log a function exit with optional result
#[macro_export]
macro_rules! log_fn_exit {
    ($fn_name:expr) => {
        tracing::debug!(target: "function", fn_name = $fn_name, "← exiting");
    };
    ($fn_name:expr, $result:expr) => {
        tracing::debug!(target: "function", fn_name = $fn_name, result = ?$result, "← exiting");
    };
}

/// Log a timed operation (measures and logs duration)
#[macro_export]
macro_rules! log_timed {
    ($name:expr, $block:expr) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration_ms = start.elapsed().as_millis() as u64;
        tracing::info!(target: "timing", operation = $name, duration_ms = duration_ms, "operation completed");
        result
    }};
}

/// Log an API handler entry
#[macro_export]
macro_rules! log_handler {
    ($handler_name:expr, $req:expr) => {{
        let trace_ctx = $crate::get_trace_context($req);
        tracing::info!(
            target: "handler",
            handler = $handler_name,
            trace_id = %trace_ctx.trace_id,
            span_id = %trace_ctx.span_id,
            "handling request"
        );
        trace_ctx
    }};
}

/// Log a database operation
#[macro_export]
macro_rules! log_db {
    ($operation:expr, $table:expr) => {
        tracing::debug!(target: "database", operation = $operation, table = $table, "db operation");
    };
    ($operation:expr, $table:expr, $id:expr) => {
        tracing::debug!(target: "database", operation = $operation, table = $table, id = ?$id, "db operation");
    };
    ($operation:expr, $table:expr, $id:expr, $duration_ms:expr) => {
        tracing::debug!(target: "database", operation = $operation, table = $table, id = ?$id, duration_ms = $duration_ms, "db operation");
    };
}

/// Log an external service call
#[macro_export]
macro_rules! log_external_call {
    ($service:expr, $endpoint:expr) => {
        tracing::debug!(target: "external", service = $service, endpoint = $endpoint, "calling external service");
    };
    ($service:expr, $endpoint:expr, $duration_ms:expr, $status:expr) => {
        tracing::info!(target: "external", service = $service, endpoint = $endpoint, duration_ms = $duration_ms, status = $status, "external call completed");
    };
}

/// Log a cache operation
#[macro_export]
macro_rules! log_cache {
    (hit, $key:expr) => {
        tracing::debug!(target: "cache", operation = "hit", key = $key, "cache hit");
    };
    (miss, $key:expr) => {
        tracing::debug!(target: "cache", operation = "miss", key = $key, "cache miss");
    };
    (set, $key:expr) => {
        tracing::debug!(target: "cache", operation = "set", key = $key, "cache set");
    };
    (evict, $key:expr) => {
        tracing::debug!(target: "cache", operation = "evict", key = $key, "cache evict");
    };
}

/// Log a business rule check
#[macro_export]
macro_rules! log_rule {
    ($rule:expr, pass) => {
        tracing::debug!(target: "rules", rule = $rule, result = "pass", "rule check passed");
    };
    ($rule:expr, fail, $reason:expr) => {
        tracing::warn!(target: "rules", rule = $rule, result = "fail", reason = $reason, "rule check failed");
    };
}

/// Log a feature toggle check
#[macro_export]
macro_rules! log_feature {
    ($feature:expr, $enabled:expr) => {
        tracing::debug!(target: "features", feature = $feature, enabled = $enabled, "feature check");
    };
}

/// Log a retry attempt
#[macro_export]
macro_rules! log_retry {
    ($operation:expr, $attempt:expr, $max_attempts:expr) => {
        tracing::warn!(target: "retry", operation = $operation, attempt = $attempt, max_attempts = $max_attempts, "retrying operation");
    };
    ($operation:expr, $attempt:expr, $max_attempts:expr, $error:expr) => {
        tracing::warn!(target: "retry", operation = $operation, attempt = $attempt, max_attempts = $max_attempts, error = %$error, "retrying after error");
    };
}

/// Log a security event
#[macro_export]
macro_rules! log_security {
    ($event:expr) => {
        tracing::warn!(target: "security", event = $event, "security event");
    };
    ($event:expr, $($key:ident = $value:expr),* $(,)?) => {
        tracing::warn!(target: "security", event = $event, $($key = ?$value),*, "security event");
    };
}

/// Log a deprecation warning
#[macro_export]
macro_rules! log_deprecated {
    ($feature:expr) => {
        tracing::warn!(target: "deprecation", feature = $feature, "deprecated feature used");
    };
    ($feature:expr, $alternative:expr) => {
        tracing::warn!(target: "deprecation", feature = $feature, alternative = $alternative, "deprecated feature used, consider using alternative");
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_macros_compile() {
        // Just verify macros compile
        log_fn_entry!("test_fn");
        log_fn_entry!("test_fn", arg1 = 42, arg2 = "hello");
        log_fn_exit!("test_fn");
        log_fn_exit!("test_fn", Ok::<_, ()>(42));
        
        log_db!("SELECT", "users");
        log_db!("INSERT", "users", "123");
        
        log_cache!(hit, "user:123");
        log_cache!(miss, "user:456");
        
        log_rule!("max_documents", pass);
        log_rule!("rate_limit", fail, "exceeded 100 requests/minute");
        
        log_feature!("new_search_algorithm", true);
        
        log_retry!("fetch_github", 2, 3);
        
        log_security!("invalid_token");
        log_security!("access_denied", user_id = "123", resource = "billing");
    }
}
