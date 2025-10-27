FROM rust:1.75 AS builder
WORKDIR /app

# Copy service Cargo files
COPY security/Cargo.toml ./Cargo.toml

# Copy shared dependencies
COPY shared/models/ ./shared/models/
COPY shared/utils/ ./shared/utils/
COPY shared/middleware/ ./shared/middleware/
COPY shared/config/ ./shared/config/

# Copy service source code
COPY security/src ./src

# Build
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/security-service /usr/local/bin/
EXPOSE 3012
CMD ["security-service"]
