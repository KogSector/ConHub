# Redis Feature Toggle

## Overview

The `Redis` feature toggle controls whether Redis connections are established throughout the ConHub codebase. This allows you to run the application without Redis for development or testing purposes.

## Configuration

### Feature Toggles File

Edit `feature-toggles.json` in the project root:

```json
{
  "Auth": true,
  "Redis": true,    // Enable/disable Redis connections
  "Heavy": false,
  "Docker": false
}
```

### Behavior

| Auth | Redis | Result |
|------|-------|--------|
| `false` | `false` | No Redis connection (Auth disabled) |
| `false` | `true` | No Redis connection (Auth disabled) |
| `true` | `false` | No Redis connection (Redis disabled) |
| `true` | `true` | Redis connection established ✅ |

**Key Point**: Redis connections are only established when **both** Auth and Redis toggles are enabled.

## Affected Services

### Services Using Redis

1. **Auth Service** (Port 3010)
   - Session management
   - Token storage
   - User session tracking

2. **Backend Service** (Port 8000)
   - API caching
   - Session validation
   - Rate limiting (if enabled)

### Services NOT Using Redis

- Billing Service
- AI/Client Service
- Data Service
- Security Service
- Webhook Service
- Frontend

## Usage Examples

### Development Without Redis

If you don't have Redis installed or want to test without it:

```json
{
  "Auth": true,
  "Redis": false,
  "Heavy": false,
  "Docker": false
}
```

**Result**:
- Services start normally
- Auth service runs without session management
- No Redis connection errors
- Stateless authentication only

### Production With Redis

For full functionality with session management:

```json
{
  "Auth": true,
  "Redis": true,
  "Heavy": false,
  "Docker": false
}
```

**Result**:
- Full session management
- Token caching
- Better performance
- Stateful authentication

## Code Implementation

### Rust Services

The feature toggle is checked via the `FeatureToggles` struct:

```rust
use conhub_config::feature_toggles::FeatureToggles;

let toggles = FeatureToggles::from_env_path();
let redis_enabled = toggles.should_connect_redis();

let redis_client_opt: Option<redis::Client> = if redis_enabled {
    // Establish Redis connection
    println!("📊 [Service] Connecting to Redis...");
    // ...
} else {
    // Skip Redis connection
    println!("⚠️  [Service] Redis feature disabled; skipping Redis connection.");
    None
};
```

### Available Methods

```rust
// Check if Redis is enabled in toggles
toggles.redis_enabled() -> bool

// Check if Redis should be connected (Auth AND Redis enabled)
toggles.should_connect_redis() -> bool
```

## Startup Messages

### With Redis Enabled

```
[SERVICES] Starting services (Auth: enabled, Redis: enabled)...
[Auth] 📊 [Auth Service] Connecting to Redis...
[Auth] ✅ [Auth Service] Redis connection established
[Backend] 📊 [Backend Service] Connecting to Redis...
[Backend] ✅ [Backend Service] Connected to Redis
```

### With Redis Disabled

```
[SERVICES] Starting services (Auth: enabled, Redis: disabled)...
[Auth] ⚠️  [Auth Service] Redis feature disabled; skipping Redis connection.
[Backend] ⚠️  [Backend Service] Redis feature disabled; skipping Redis connection.
```

### With Auth Disabled

```
[SERVICES] Starting services (Auth: disabled, Redis: enabled)...
[Auth] ⚠️  [Auth Service] Auth disabled; skipping Redis connection.
[Backend] ⚠️  [Backend Service] Auth disabled; skipping Redis connection.
```

## Environment Variables

Redis connection is configured via environment variables:

```env
# Redis URLs (service reads ENV_MODE to choose)
REDIS_URL_LOCAL=redis://localhost:6379
REDIS_URL_DOCKER=redis://redis:6379

# Or use generic REDIS_URL
REDIS_URL=redis://localhost:6379
```

## Testing

### Test Without Redis

1. Set Redis toggle to `false`:
   ```json
   { "Auth": true, "Redis": false }
   ```

2. Start services:
   ```bash
   npm start
   ```

3. Verify no Redis errors in logs

### Test With Redis

1. Ensure Redis is running:
   ```bash
   redis-cli ping
   # Should return: PONG
   ```

2. Set Redis toggle to `true`:
   ```json
   { "Auth": true, "Redis": true }
   ```

3. Start services:
   ```bash
   npm start
   ```

4. Verify Redis connections in logs

## Troubleshooting

### Redis Connection Failed (When Enabled)

**Error**:
```
⚠️  [Auth Service] Failed to connect to Redis: Connection refused
```

**Solutions**:
1. Check if Redis is running:
   ```bash
   redis-cli ping
   ```

2. Verify REDIS_URL in `.env`:
   ```env
   REDIS_URL=redis://localhost:6379
   ```

3. Or disable Redis temporarily:
   ```json
   { "Redis": false }
   ```

### Session Management Not Working

**Symptom**: Users can't maintain sessions across requests

**Check**:
1. Redis toggle is enabled
2. Redis is running
3. Auth toggle is enabled

```json
{
  "Auth": true,
  "Redis": true
}
```

### Performance Issues

If Redis is causing performance issues:

1. **Temporary**: Disable Redis
   ```json
   { "Redis": false }
   ```

2. **Permanent**: Optimize Redis configuration
   - Increase connection pool size
   - Configure Redis persistence
   - Use Redis Cluster for scaling

## Architecture Notes

### Why Separate Redis Toggle?

1. **Flexibility**: Run without Redis for development
2. **Testing**: Test stateless authentication
3. **Deployment**: Gradual rollout of Redis
4. **Cost**: Reduce infrastructure costs in dev/test

### Default Behavior

- **Default**: `Redis: true` (when Auth is enabled)
- **Fallback**: If Redis toggle is missing, defaults to Auth toggle value
- **Logic**: `redis_enabled = Redis ?? Auth`

### Session Management Without Redis

When Redis is disabled:
- Sessions are not persisted
- Each request is stateless
- JWT tokens must be validated on every request
- No session caching

## Best Practices

### Development

```json
{
  "Auth": true,
  "Redis": false,  // Simplify local dev
  "Heavy": false,
  "Docker": false
}
```

### Staging/Testing

```json
{
  "Auth": true,
  "Redis": true,   // Test full functionality
  "Heavy": false,
  "Docker": false
}
```

### Production

```json
{
  "Auth": true,
  "Redis": true,   // Full session management
  "Heavy": true,   // Enable all features
  "Docker": false
}
```

## Related Documentation

- [Feature Toggles Overview](../README.md#feature-toggles)
- [NeonDB Setup](NEONDB_SETUP.md)
- [Authentication Guide](AUTH_GUIDE.md)

## Summary

The Redis feature toggle provides fine-grained control over Redis connections:

- ✅ **Enabled**: Full session management and caching
- ❌ **Disabled**: Stateless operation, no Redis required
- 🔄 **Dynamic**: Can be changed without code modifications
- 🎯 **Scoped**: Only affects Auth and Backend services

Use it to simplify development, reduce dependencies, or test different deployment scenarios.
