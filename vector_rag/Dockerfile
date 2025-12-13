# Multi-stage build for Vector RAG service
FROM rust:1.75-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy shared dependencies first
COPY shared ./shared

# Copy vector_rag service
COPY vector_rag ./vector_rag

# Build the service
WORKDIR /app/vector_rag
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
COPY --from=builder /app/vector_rag/target/release/vector_rag /app/vector_rag

# Set ownership
RUN chown -R conhub:conhub /app

USER conhub

# Expose port
EXPOSE 3017

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3017/health || exit 1

CMD ["./vector_rag"]
