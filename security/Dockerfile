# Multi-stage build for Security service
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

# Copy security service
COPY security ./security

# Build the service
WORKDIR /app/security
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

# Copy feature toggles
COPY feature-toggles.json /etc/conhub/feature-toggles.json
ENV FEATURE_TOGGLES_PATH=/etc/conhub/feature-toggles.json

# Copy binary from builder
COPY --from=builder /app/security/target/release/security-service /app/security-service

# Set ownership
RUN chown -R conhub:conhub /app /etc/conhub

USER conhub

EXPOSE 3014

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3014/health || exit 1

CMD ["./security-service"]
