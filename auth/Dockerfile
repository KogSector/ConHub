# Stage 1: Build
FROM rust:1.75-alpine AS builder

WORKDIR /workspace

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev pkgconfig

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY shared/ ./shared/
COPY services/auth-service/ ./services/auth-service/

# Build only auth service
RUN cargo build --release --bin auth-service

# Stage 2: Runtime
FROM alpine:latest

RUN apk add --no-cache libgcc

WORKDIR /app

# Copy binary from builder
COPY --from=builder /workspace/target/release/auth-service /app/

EXPOSE 3010

CMD ["./auth-service"]
