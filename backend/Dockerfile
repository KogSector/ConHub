# Multi-stage build for Rust backend
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY backend/src ./backend/src
COPY backend/migrations ./backend/migrations

# Build dependencies (this is the caching Docker layer!)
RUN cargo build --release --bin conhub-backend

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN groupadd -r conhub && useradd -r -g conhub conhub

# Copy the binary
COPY --from=builder /app/target/release/conhub-backend /usr/local/bin/conhub-backend

# Copy migrations
COPY --from=builder /app/backend/migrations /app/migrations

# Set ownership
RUN chown -R conhub:conhub /app

USER conhub

EXPOSE 3001

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3001/health || exit 1

CMD ["conhub-backend"]