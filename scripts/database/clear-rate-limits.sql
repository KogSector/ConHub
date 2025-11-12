-- Clear all rate limits
DELETE FROM rate_limits;

-- Clear all security audit logs (optional - for clean slate)
DELETE FROM security_audit_log;

-- Reset failed login attempts for all users
UPDATE users SET 
    failed_login_attempts = 0,
    is_locked = FALSE,
    locked_until = NULL
WHERE failed_login_attempts > 0 OR is_locked = TRUE OR locked_until IS NOT NULL;

-- Show current rate limits (should be empty after cleanup)
SELECT 
    identifier,
    action,
    attempts,
    window_start,
    blocked_until,
    created_at,
    updated_at
FROM rate_limits
ORDER BY created_at DESC;

-- Show users with any security issues
SELECT 
    id,
    email,
    failed_login_attempts,
    is_locked,
    locked_until,
    last_login_at
FROM users
WHERE failed_login_attempts > 0 OR is_locked = TRUE OR locked_until IS NOT NULL;