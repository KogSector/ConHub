# Multi-stage build for embedding microservice
FROM rust:latest as builder

WORKDIR /build

# Install Python for PyO3 build detection
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 \
    python3-pip \
 && rm -rf /var/lib/apt/lists/*

# Ensure PyO3 uses Python 3 during build
ENV PYO3_PYTHON=/usr/bin/python3

# Copy shared dependencies first
COPY shared/ /build/shared/

# Copy embedding service
COPY embedding/ /build/embedding/

# Build release binary
WORKDIR /build/embedding
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    python3 \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Install Python packages for HuggingFace embeddings (CPU-only by default)
RUN pip3 install --no-cache-dir --upgrade pip \
 && pip3 install --no-cache-dir sentence-transformers transformers numpy \
 && pip3 install --no-cache-dir --index-url https://download.pytorch.org/whl/cpu torch

WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/embedding/target/release/embedding /app/embedding

# Expose port
EXPOSE 8082

# Set environment
ENV RUST_LOG=info
ENV EMBEDDING_PORT=8082
ENV EMBEDDING_PROVIDER=huggingface
ENV EMBEDDING_MODEL=Qwen/Qwen3-Embedding-0.6B

# Run the service
CMD ["./embedding"]
