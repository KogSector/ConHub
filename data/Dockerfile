FROM rustlang/rust:nightly AS builder
WORKDIR /app

# Development stage
FROM builder AS development
RUN cargo install cargo-watch
CMD ["cargo", "watch", "-q", "-x", "run"]

# Copy shared dependencies first (to parent directory structure)
COPY shared/ ../shared/

# Copy service Cargo files
COPY data/Cargo.toml ./Cargo.toml

# Copy service source code
COPY data/src ./src

# Build
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/data-service /usr/local/bin/
EXPOSE 3013
CMD ["data-service"]
