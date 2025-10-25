FROM rust:1.75 as builder

WORKDIR /app

# Copy workspace members
COPY backend/ backend/
COPY auth/ auth/
COPY billing/ billing/
COPY ai/ ai/
COPY security/ security/
COPY indexers/ indexers/
COPY shared/ shared/
COPY Cargo.toml Cargo.lock ./

# Build backend
WORKDIR /app/backend
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/conhub-backend /app/backend

EXPOSE 8000

CMD ["/app/backend"]
