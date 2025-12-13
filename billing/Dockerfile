# Multi-stage build for Billing service
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

# Copy billing service
COPY billing ./billing

# Build the service
WORKDIR /app/billing
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
COPY --from=builder /app/billing/target/release/billing-service /app/billing-service

# Set ownership
RUN chown -R conhub:conhub /app /etc/conhub

USER conhub

EXPOSE 3011

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3011/health || exit 1

CMD ["./billing-service"]
