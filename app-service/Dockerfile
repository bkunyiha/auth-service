# Multi-stage Docker build for Rust app-service
# Stage 1: Chef - Base image with Rust toolchain for dependency caching
FROM rust:1.85-alpine AS chef
USER root
# Install cargo-chef for dependency caching
RUN apk add --no-cache musl-dev && cargo install cargo-chef
WORKDIR /app

# Stage 2: Planner - Analyze dependencies without building
FROM chef AS planner
COPY . .
# Generate dependency recipe
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Builder - Build the actual application
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies (cached layer)
RUN cargo chef cook --release --recipe-path recipe.json
# Copy source code
COPY . .
# Build the Rust binary
RUN cargo build --release --bin app-service

# Stage 4: Runtime - Minimal production image
FROM debian:buster-slim AS runtime
WORKDIR /app
# Copy compiled binary
COPY --from=builder /app/target/release/app-service /usr/local/bin
# Copy static assets (JS, CSS, images)
COPY --from=builder /app/assets /app/assets
# Auth service hostname for token validation
ENV AUTH_SERVICE_HOST_NAME=auth-service
# Set binary as entrypoint
ENTRYPOINT ["/usr/local/bin/app-service"]