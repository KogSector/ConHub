FROM rustlang/rust:nightly AS builder
WORKDIR /app

# Copy shared dependencies first (to parent directory structure)
COPY shared/ ../shared/

# Copy service Cargo files
COPY billing/Cargo.toml ./Cargo.toml

# Copy service source code
COPY billing/src ./src

# Build
RUN cargo build --release

# Development stage
FROM builder AS development
RUN cargo install cargo-watch
CMD ["cargo", "watch", "-q", "-x", "run"]

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
# Copy feature toggles into runtime image and set path
COPY feature-toggles.json /usr/local/etc/conhub/feature-toggles.json
ENV FEATURE_TOGGLES_PATH=/usr/local/etc/conhub/feature-toggles.json
COPY --from=builder /app/target/release/billing-service /usr/local/bin/
EXPOSE 3011
CMD ["billing-service"]
