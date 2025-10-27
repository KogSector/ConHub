FROM rust:1.75 AS builder
WORKDIR /app

# Copy service Cargo files
COPY webhook/Cargo.toml ./Cargo.toml

# Copy shared dependencies
COPY shared/models/ ./shared/models/
COPY shared/utils/ ./shared/utils/
COPY shared/middleware/ ./shared/middleware/
COPY shared/config/ ./shared/config/

# Copy service source code
COPY webhook/src ./src

# Build
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/webhook-service /usr/local/bin/
EXPOSE 3015
CMD ["webhook-service"]
