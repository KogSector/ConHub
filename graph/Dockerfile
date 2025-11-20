# Multi-stage build for ConHub Graph Service
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy shared dependencies first
COPY shared /app/shared

# Copy graph service
COPY graph /app/graph

# Build the service
WORKDIR /app/graph
RUN cargo build --release

# Production image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1001 conhub

WORKDIR /app

# Copy binary
COPY --from=builder /app/graph/target/release/graph /app/graph

# Set ownership
RUN chown -R conhub:conhub /app

USER conhub

EXPOSE 8006

CMD ["./graph"]
