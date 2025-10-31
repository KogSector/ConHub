# Multi-stage build for embedding microservice
FROM rust:latest as builder

WORKDIR /build

# Copy embedding service
COPY embedding/ /build/embedding/

# Build release binary
WORKDIR /build/embedding
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/embedding/target/release/embedding /app/embedding

# Expose port
EXPOSE 8082

# Set environment
ENV RUST_LOG=info
ENV EMBEDDING_PORT=8082

# Run the service
CMD ["./embedding"]
