# Multi-stage build for Decision Engine service
FROM rust:1.75-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy shared dependencies first
COPY shared ./shared

# Copy decision_engine service
COPY decision_engine ./decision_engine

# Build the service
WORKDIR /app/decision_engine
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

# Copy binary from builder
COPY --from=builder /app/decision_engine/target/release/decision_engine /app/decision_engine

# Set ownership
RUN chown -R conhub:conhub /app

USER conhub

# Expose port
EXPOSE 3019

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3019/health || exit 1

CMD ["./decision_engine"]
