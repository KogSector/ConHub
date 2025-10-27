# Multi-stage build for the unified plugins service
FROM rust:1.75 as builder

WORKDIR /app

# Copy shared dependencies first
COPY shared/ ./shared/

# Copy plugins service
COPY plugins/ ./plugins/

# Build the plugins service
WORKDIR /app/plugins
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false conhub

# Create directories
RUN mkdir -p /app/config /app/logs
COPY --from=builder /app/plugins/target/release/conhub-plugins-service /app/
COPY --from=builder /app/plugins/config/ /app/config/

# Set ownership
RUN chown -R conhub:conhub /app

# Switch to app user
USER conhub

WORKDIR /app

# Expose port
EXPOSE 3020

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3020/health || exit 1

# Start the service
CMD ["./conhub-plugins-service"]