# Build stage
FROM rust:1.75-alpine as builder

WORKDIR /workspace

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev pkgconfig git

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY shared/ ./shared/
COPY indexers/ ./indexers/

# Build the unified-indexer
RUN cargo build --release --bin unified-indexer

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache ca-certificates libgcc libssl3 curl git

WORKDIR /app

# Copy binary from builder
COPY --from=builder /workspace/target/release/unified-indexer /app/unified-indexer

# Create data directories
RUN mkdir -p /app/data/index /app/data/temp

# Set environment variables
ENV RUST_LOG=info
ENV INDEXER_HOST=0.0.0.0
ENV INDEXER_PORT=8080

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["/app/unified-indexer"]
