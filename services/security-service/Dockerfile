# Stage 1: Build
FROM rust:1.75-alpine AS builder

WORKDIR /workspace

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev pkgconfig

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY shared/ ./shared/
COPY services/security-service/ ./services/security-service/

# Build only security service
RUN cargo build --release --bin security-service

# Stage 2: Runtime
FROM alpine:latest

RUN apk add --no-cache libgcc

WORKDIR /app

# Copy binary from builder
COPY --from=builder /workspace/target/release/security-service /app/

EXPOSE 3014

CMD ["./security-service"]
