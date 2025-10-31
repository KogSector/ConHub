FROM rustlang/rust:nightly AS builder
WORKDIR /app

# Copy shared dependencies first (to parent directory structure)
COPY shared/ ../shared/

# Copy service Cargo files
COPY client/Cargo.toml ./Cargo.toml

# Copy service source code
COPY client/src ./src

# Build
RUN cargo build --release

# Development stage
FROM builder AS development
RUN cargo install cargo-watch
CMD ["cargo", "watch", "-q", "-x", "run"]

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/client-service /usr/local/bin/
EXPOSE 3014
CMD ["client-service"]
