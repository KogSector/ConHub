# Multi-stage build for Decision Engine service
FROM rust:1.75-bullseye as builder

WORKDIR /app

# Copy workspace files
COPY shared ./shared
COPY decision_engine ./decision_engine

# Build the service
WORKDIR /app/decision_engine
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
COPY --from=builder /app/decision_engine/target/release/decision_engine /app/decision_engine

# Expose port
EXPOSE 3018

# Run the service
CMD ["./decision_engine"]
