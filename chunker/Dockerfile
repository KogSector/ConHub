# Multi-stage build for Chunker service
FROM rust:1.75-slim AS builder

# Install build dependencies (tree-sitter needs C compiler, OpenSSL for reqwest)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    cmake \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy shared dependencies first
COPY shared ./shared

# Copy chunker service
COPY chunker ./chunker

# Build the service
WORKDIR /app/chunker
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1001 conhub

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/chunker/target/release/chunker /app/chunker

# Set ownership
RUN chown -R conhub:conhub /app

USER conhub

# Expose port
EXPOSE 3016

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3016/health || exit 1

CMD ["./chunker"]
