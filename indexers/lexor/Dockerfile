# Multi-stage build for Lexor indexing service
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests from root
COPY Cargo.toml Cargo.lock ./

# Copy lexor source
COPY indexers/lexor/src ./indexers/lexor/src

# Build lexor
RUN cargo build --release --bin lexor

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN groupadd -r lexor && useradd -r -g lexor lexor

# Create data directory
RUN mkdir -p /app/data && chown -R lexor:lexor /app

# Copy the binary
COPY --from=builder /app/target/release/lexor /usr/local/bin/lexor

USER lexor

WORKDIR /app

EXPOSE 3002

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3002/health || exit 1

CMD ["lexor"]