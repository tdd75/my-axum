# Build stage
FROM rust:bullseye AS builder

WORKDIR /app

# Install cmake for building certain dependencies
RUN apt-get update && apt-get install -y cmake

# Cache dependencies
COPY migration/Cargo.toml migration/Cargo.lock ./migration/
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

# Copy source code
COPY bin bin
COPY src src

# Build the application
RUN cargo build --release

# ------------------------------------------------

# Final stage
FROM rust:bullseye

WORKDIR /app

# Install cmake for building certain dependencies
RUN apt-get update && apt-get install -y cmake

# Install sea-orm-cli for database migrations
RUN cargo install sea-orm-cli@^2.0.0-rc

# Copy the built binaries from the builder stage
COPY --from=builder /app/target/release/my-axum /app/target/release/worker /app/target/release/seed ./

# Copy migration files
COPY migration migration

# Copy source code
COPY src src
