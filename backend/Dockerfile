FROM rustlang/rust:nightly AS builder
WORKDIR /app

# Copy shared dependencies first (to parent directory structure)
COPY shared/ ../shared/

# Copy library crates that backend depends on (to parent directory structure)
COPY auth/ ../auth/
COPY billing/ ../billing/
COPY security/ ../security/
COPY indexers/ ../indexers/

# Copy backend Cargo files
COPY backend/Cargo.toml ./Cargo.toml

# Copy backend source code
COPY backend/src ./src

# Build
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 libpq5 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/conhub-backend /usr/local/bin/
EXPOSE 8000
CMD ["conhub-backend"]
