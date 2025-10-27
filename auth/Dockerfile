FROM rustlang/rust:nightly AS builder
WORKDIR /app

# Copy shared dependencies first (to parent directory structure)
COPY shared/ ../shared/

# Copy service Cargo files
COPY auth/Cargo.toml ./Cargo.toml

# Copy service source code
COPY auth/src ./src

# Build
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/auth-service /usr/local/bin/
EXPOSE 3010
CMD ["auth-service"]
