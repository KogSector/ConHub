# Build stage with optimized caching
FROM rust:1.75-slim AS builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev libpq-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

# Copy workspace configuration first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Copy all workspace members' Cargo.toml files to cache dependencies
COPY backend/Cargo.toml backend/Cargo.toml
COPY auth/Cargo.toml auth/Cargo.toml
COPY billing/Cargo.toml billing/Cargo.toml
COPY client/Cargo.toml client/Cargo.toml
COPY data/Cargo.toml data/Cargo.toml
COPY embedding/Cargo.toml embedding/Cargo.toml
COPY indexers/Cargo.toml indexers/Cargo.toml
COPY plugins/Cargo.toml plugins/Cargo.toml
COPY plugins/plugins/amazon-q/Cargo.toml plugins/plugins/amazon-q/Cargo.toml
COPY plugins/plugins/cline/Cargo.toml plugins/plugins/cline/Cargo.toml
COPY plugins/plugins/dropbox/Cargo.toml plugins/plugins/dropbox/Cargo.toml
COPY plugins/plugins/google-drive/Cargo.toml plugins/plugins/google-drive/Cargo.toml
COPY security/Cargo.toml security/Cargo.toml
COPY webhook/Cargo.toml webhook/Cargo.toml
COPY shared/config/Cargo.toml shared/config/Cargo.toml
COPY shared/middleware/Cargo.toml shared/middleware/Cargo.toml
COPY shared/models/Cargo.toml shared/models/Cargo.toml
COPY shared/plugins/Cargo.toml shared/plugins/Cargo.toml
COPY shared/utils/Cargo.toml shared/utils/Cargo.toml

# Create dummy source files to cache dependencies
RUN mkdir -p backend/src && \
    echo "fn main() {}" > backend/src/main.rs && \
    # Explicitly create each directory (POSIX sh doesn't support brace expansion)
    mkdir -p shared/config/src shared/middleware/src shared/models/src shared/plugins/src shared/utils/src && \
    echo "pub fn dummy() {}" > shared/config/src/lib.rs && \
    echo "pub fn dummy() {}" > shared/middleware/src/lib.rs && \
    echo "pub fn dummy() {}" > shared/models/src/lib.rs && \
    echo "pub fn dummy() {}" > shared/plugins/src/lib.rs && \
    echo "pub fn dummy() {}" > shared/utils/src/lib.rs && \
    # Create dummy binaries for all workspace services so Cargo can parse manifests
    mkdir -p auth/src billing/src client/src data/src embedding/src indexers/src plugins/src security/src webhook/src && \
    echo "fn main() {}" > auth/src/main.rs && \
    echo "fn main() {}" > billing/src/main.rs && \
    echo "fn main() {}" > client/src/main.rs && \
    echo "fn main() {}" > data/src/main.rs && \
    echo "fn main() {}" > embedding/src/main.rs && \
    echo "fn main() {}" > indexers/src/main.rs && \
    echo "fn main() {}" > plugins/src/main.rs && \
    echo "fn main() {}" > security/src/main.rs && \
    echo "fn main() {}" > webhook/src/main.rs && \
    # Ensure optional plugin subcrates are resolvable during workspace parsing
    mkdir -p plugins/plugins/amazon-q/src plugins/plugins/cline/src plugins/plugins/dropbox/src plugins/plugins/google-drive/src && \
    echo "pub fn dummy() {}" > plugins/plugins/amazon-q/src/lib.rs && \
    echo "pub fn dummy() {}" > plugins/plugins/cline/src/lib.rs && \
    echo "pub fn dummy() {}" > plugins/plugins/dropbox/src/lib.rs && \
    echo "pub fn dummy() {}" > plugins/plugins/google-drive/src/lib.rs

# Build dependencies only (this layer will be cached)
RUN cargo build --release -p conhub-backend || true

# Copy actual source code
COPY shared/ shared/
COPY backend/src backend/src

# Build actual application with release optimizations
RUN cargo build --release -p conhub-backend

# Runtime stage - minimal image
FROM debian:bookworm-slim

# Install runtime dependencies only
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 libpq5 && \
    rm -rf /var/lib/apt/lists/*

# Create app user for security
RUN useradd -m -u 1001 appuser

# Copy feature toggles
COPY feature-toggles.json /etc/conhub/feature-toggles.json
ENV FEATURE_TOGGLES_PATH=/etc/conhub/feature-toggles.json

# Copy binary from builder
COPY --from=builder /workspace/target/release/conhub-backend /usr/local/bin/conhub-backend

# Set ownership
RUN chown -R appuser:appuser /etc/conhub

# Switch to non-root user
USER appuser

EXPOSE 8000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

CMD ["conhub-backend"]
