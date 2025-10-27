# Stage 1: Build
FROM rust:1.75-alpine AS builder

WORKDIR /workspace

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev pkgconfig

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY shared/ ./shared/
COPY services/data-service/ ./services/data-service/

# Build only data service
RUN cargo build --release --bin data-service

# Stage 2: Runtime
FROM alpine:latest

RUN apk add --no-cache libgcc

WORKDIR /app

# Copy binary from builder
COPY --from=builder /workspace/target/release/data-service /app/

EXPOSE 3013

CMD ["./data-service"]
