# Multi-stage build for Indexers service
FROM rust:1.75-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy shared dependencies first
COPY shared ./shared

# Copy indexers service
COPY indexers ./indexers

# Build the service
WORKDIR /app/indexers
RUN cargo build --release --bin unified-indexer

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1001 conhub

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/indexers/target/release/unified-indexer /app/unified-indexer

# Create data directories
RUN mkdir -p /app/data/index /app/data/temp && \
    chown -R conhub:conhub /app

USER conhub

# Set environment variables
ENV RUST_LOG=info
ENV INDEXER_HOST=0.0.0.0
ENV INDEXER_PORT=3020

EXPOSE 3020

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3020/health || exit 1

CMD ["/app/unified-indexer"]
