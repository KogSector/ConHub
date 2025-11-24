# Multi-stage build for Chunker service
FROM rust:1.75-bullseye as builder

WORKDIR /app

# Copy workspace files
COPY shared ./shared
COPY database ./database
COPY chunker ./chunker

# Build the service
WORKDIR /app/chunker
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
COPY --from=builder /app/chunker/target/release/chunker /app/chunker

# Expose port
EXPOSE 3017

# Run the service
CMD ["./chunker"]
