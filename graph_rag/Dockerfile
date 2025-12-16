# Multi-stage build for ConHub Graph Service
FROM rust:1.75-slim AS builder

# Install build dependencies (libpq for Postgres)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy shared dependencies first
COPY shared ./shared

# Copy graph_rag service
COPY graph_rag ./graph_rag

# Build the service
WORKDIR /app/graph_rag
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1001 conhub

WORKDIR /app

# Copy binary
COPY --from=builder /app/graph_rag/target/release/graph_rag /app/graph_rag

# Set ownership
RUN chown -R conhub:conhub /app

USER conhub

EXPOSE 3018

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3018/health || exit 1

CMD ["./graph_rag"]
