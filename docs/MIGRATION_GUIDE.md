# Migration Guide: Optimized ConHub Architecture

This guide helps you migrate from the previous architecture to the newly optimized ConHub setup.

## Overview of Changes

1. ✅ **Cargo Workspace**: All Rust services now use a unified workspace
2. ✅ **GraphQL Integration**: Inter-service communication via GraphQL
3. ✅ **Enhanced Feature Toggles**: `Auth` and `Heavy` flags with hot reload
4. ✅ **Optimized Docker Builds**: Multi-stage builds with dependency caching
5. ✅ **Connection Pool Manager**: Shared, intelligent connection pooling
6. ✅ **Azure Deployment Fixes**: Consistent ports and environment variables

---

## Step 1: Update Dependencies

### Remove Old Cargo.lock Files (Optional)
```bash
# Backup existing locks
find . -name "Cargo.lock" -exec cp {} {}.backup \;

# Remove service-level locks (workspace will manage)
find . -path "*/target" -prune -o -name "Cargo.lock" -not -path "./Cargo.lock" -delete
```

### Rebuild with Workspace
```bash
# Clean previous builds
cargo clean

# Build entire workspace
cargo build --workspace --release

# Or build specific service
cargo build --release -p conhub-backend
cargo build --release -p auth-service
cargo build --release -p data-service
```

---

## Step 2: Update Service Cargo.toml Files

### Before (Individual Service)
```toml
[dependencies]
actix-web = "4.4"
sqlx = { version = "0.7", features = ["..."] }
tokio = { version = "1.0", features = ["full"] }
```

### After (Workspace Member)
```toml
[dependencies]
actix-web = { workspace = true }
sqlx = { workspace = true }
tokio = { workspace = true }
```

### Migration Command (Automated)
```bash
# Update all service Cargo.toml files
for service in auth backend billing client data embedding indexers plugins security webhook; do
  cd $service
  # Replace version-specific deps with workspace = true
  # Manual review recommended
  cd ..
done
```

---

## Step 3: Feature Toggles Migration

### Update feature-toggles.json
```json
{
  "Auth": false,
  "Heavy": false
}
```

**Development Modes:**

| Mode | Auth | Heavy | Use Case |
|------|------|-------|----------|
| Minimal | false | false | Frontend development |
| Auth Testing | true | false | Auth flows, no heavy ops |
| Full Stack | true | true | Production-like |

### Update Service Code

#### Before
```rust
let db_pool = PgPoolOptions::new()
    .connect(&database_url)
    .await?;
```

#### After
```rust
use conhub_config::feature_toggles::FeatureToggles;

let toggles = FeatureToggles::from_env_path();

let db_pool_opt = if toggles.auth_enabled() {
    Some(PgPoolOptions::new()
        .connect(&database_url)
        .await?)
} else {
    None
};
```

---

## Step 4: GraphQL Integration

### Backend Service Update

#### Add to main.rs
```rust
use crate::graphql::schema::build_schema;

// In HttpServer::new
let schema = build_schema(config.clone(), toggles.clone());

App::new()
    .app_data(web::Data::new(schema))
    .configure(graphql::configure_graphql_routes)
```

### Use Shared GraphQL Types

#### Before (Local Types)
```rust
#[derive(SimpleObject)]
pub struct EmbedResult {
    pub embeddings: Vec<Vec<f32>>,
    // ...
}
```

#### After (Shared Types)
```rust
use conhub_models::graphql::{EmbeddingResult, RerankResult};

// Use shared types directly
```

---

## Step 5: Connection Pool Migration

### Replace Direct Connections

#### Before
```rust
let pool = PgPoolOptions::new()
    .connect(&database_url)
    .await?;
```

#### After
```rust
use conhub_utils::connection_pool::get_pool_manager;

let pool = get_pool_manager()
    .get_pg_pool(&database_url)
    .await?;
```

### Benefits
- Automatic connection reuse
- Stale connection cleanup
- Thread-safe sharing
- Built-in monitoring

---

## Step 6: Docker Build Migration

### Update Build Commands

#### Before
```bash
docker build -t conhub-backend ./backend
```

#### After (Use Build Context from Root)
```bash
# Build from root with workspace context
docker build -f backend/Dockerfile -t conhub-backend .

# Or use docker-compose
docker-compose build backend
```

### Update CI/CD Pipelines

#### GitHub Actions Example
```yaml
- name: Build Docker Images
  run: |
    docker build -f backend/Dockerfile -t ${{ env.REGISTRY }}/conhub-backend:${{ github.sha }} .
    docker build -f auth/Dockerfile -t ${{ env.REGISTRY }}/conhub-auth:${{ github.sha }} .
```

---

## Step 7: Azure Deployment Migration

### Update Environment Variables

#### Before
```yaml
env:
- name: PORT
  value: "3010"
```

#### After
```yaml
env:
- name: AUTH_SERVICE_PORT
  value: "8001"
- name: FEATURE_TOGGLES_PATH
  value: "/etc/conhub/feature-toggles.json"
- name: DATABASE_URL
  valueFrom:
    secretKeyRef:
      name: conhub-secrets
      key: database-url
```

### Deploy Updated Configuration
```bash
# Update secrets
kubectl create secret generic conhub-secrets \
  --from-literal=database-url=$DATABASE_URL \
  --from-literal=redis-url=$REDIS_URL \
  --from-literal=jwt-secret=$JWT_SECRET \
  --dry-run=client -o yaml | kubectl apply -f -

# Deploy services
kubectl apply -f azure-container-apps.yml
```

---

## Step 8: Testing Migration

### Unit Tests
```bash
# Test entire workspace
cargo test --workspace

# Test specific service
cargo test -p conhub-backend
cargo test -p auth-service
```

### Integration Tests
```bash
# Start services with docker-compose
docker-compose up -d

# Wait for health checks
sleep 30

# Run integration tests
npm run test:docker
```

### GraphQL Endpoint Test
```bash
# Test GraphQL endpoint
curl -X POST http://localhost:8000/api/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "{ health }"}'

# Expected response
{"data":{"health":"healthy"}}
```

---

## Step 9: Performance Verification

### Build Time Comparison
```bash
# Clean build
time cargo clean && cargo build --workspace --release

# Incremental build (change one file)
touch backend/src/main.rs
time cargo build --release -p conhub-backend
```

**Expected Results:**
- Initial build: ~6 minutes (was ~15 minutes)
- Incremental: ~20 seconds (was ~3 minutes)

### Docker Build Comparison
```bash
# First build
time docker build -f backend/Dockerfile -t conhub-backend .

# Rebuild after code change (cached layers)
touch backend/src/main.rs
time docker build -f backend/Dockerfile -t conhub-backend .
```

**Expected Results:**
- First build: ~5 minutes
- Cached rebuild: ~45 seconds (was ~8 minutes)

---

## Step 10: Monitoring Migration

### Add Health Check Monitoring

#### Update Health Check Endpoint
```rust
async fn health_check(
    pool_opt: web::Data<Option<PgPool>>,
    toggles: web::Data<FeatureToggles>,
) -> actix_web::Result<web::Json<serde_json::Value>> {
    Ok(web::Json(serde_json::json!({
        "status": "healthy",
        "service": "data-service",
        "database": pool_opt.is_some(),
        "features": toggles.enabled_features(),
        "timestamp": chrono::Utc::now()
    })))
}
```

### Connection Pool Monitoring
```rust
use conhub_utils::connection_pool::get_pool_manager;

let stats = get_pool_manager().get_stats();
tracing::info!(
    "Pool stats - PG: {}, Redis: {}, Qdrant: {}",
    stats.pg_pools,
    stats.redis_clients,
    stats.qdrant_clients
);
```

---

## Rollback Plan

If you encounter issues, here's how to rollback:

### 1. Restore Cargo.lock Files
```bash
find . -name "Cargo.lock.backup" -exec sh -c 'mv "$1" "${1%.backup}"' _ {} \;
```

### 2. Use Old Docker Builds
```bash
# Tag and use previous images
docker tag conhub-backend:previous conhub-backend:latest
```

### 3. Revert Deployment
```bash
kubectl rollout undo deployment/conhub-backend
kubectl rollout undo deployment/conhub-auth
```

### 4. Disable Feature Toggles
```json
{
  "Auth": true,
  "Heavy": true
}
```

---

## Common Issues and Solutions

### Issue 1: Workspace Build Errors

**Error:**
```
error: package conhub-backend cannot be built because it requires rustc 1.75 or newer
```

**Solution:**
```bash
rustup update stable
rustup default stable
```

### Issue 2: GraphQL Schema Not Found

**Error:**
```
error[E0433]: failed to resolve: use of undeclared crate or module `conhub_models`
```

**Solution:**
```bash
# Ensure workspace dependency is correct
cargo build -p conhub-models
cargo build --workspace
```

### Issue 3: Feature Toggles File Not Found

**Error:**
```
WARN Feature toggles file not found, using defaults
```

**Solution:**
```bash
# Ensure feature-toggles.json exists
cat > feature-toggles.json << EOF
{
  "Auth": false,
  "Heavy": false
}
EOF

# Set environment variable
export FEATURE_TOGGLES_PATH=/path/to/feature-toggles.json
```

### Issue 4: Connection Pool Errors

**Error:**
```
error: failed to acquire connection from pool
```

**Solution:**
```rust
// Increase pool size in config
PoolConfig {
    max_connections: 50,  // Increase from 20
    acquire_timeout: Duration::from_secs(60),  // Increase timeout
    ..Default::default()
}
```

---

## Verification Checklist

After migration, verify:

- [ ] All services build successfully with `cargo build --workspace`
- [ ] Docker images build without errors
- [ ] Feature toggles work in both modes (Auth on/off, Heavy on/off)
- [ ] GraphQL playground accessible at `/api/graphql`
- [ ] Health checks return 200 OK for all services
- [ ] Connection pool statistics show proper values
- [ ] Azure deployment succeeds with new configuration
- [ ] Build times improved as expected
- [ ] All integration tests pass

---

## Next Steps

1. **Monitor Production**: Watch for performance improvements
2. **Tune Parameters**: Adjust connection pool sizes based on load
3. **Enable Features Gradually**: Use feature toggles for controlled rollout
4. **Update Documentation**: Document service-specific configurations
5. **Train Team**: Ensure developers understand new architecture

---

## Support

If you encounter issues:

1. Check `OPTIMIZATIONS.md` for detailed architecture info
2. Review service logs: `docker-compose logs -f <service-name>`
3. Check GraphQL playground for debugging
4. Monitor connection pool stats
5. Verify feature toggle configuration

For questions, contact the architecture team or create an issue in the repository.
