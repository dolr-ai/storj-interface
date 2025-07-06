# Multi-stage build
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev pkgconfig openssl-dev

WORKDIR /app

# Copy cargo files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the release binary
RUN cargo build --release

# Final runtime image
FROM alpine:3.21.3

WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache curl ca-certificates

# Copy the binary from builder stage
COPY --from=builder /app/target/release/sia-interface .

EXPOSE 3000

ENTRYPOINT ["./sia-interface"]

