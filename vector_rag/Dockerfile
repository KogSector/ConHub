# Multi-stage build for Vector RAG service
FROM rust:1.75-bullseye as builder

WORKDIR /app

# Copy workspace files
COPY shared ./shared
COPY vector_rag ./vector_rag

# Build the service
WORKDIR /app/vector_rag
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/vector_rag/target/release/vector_rag /app/vector_rag

# Expose port
EXPOSE 8082

# Run the service
CMD ["./vector_rag"]
