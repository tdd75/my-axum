FROM rust:bullseye AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends cmake libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Cache dependencies before copying the full source tree.
COPY Cargo.toml Cargo.lock ./
COPY pkg/Cargo.toml ./pkg/Cargo.toml
COPY migration/Cargo.toml ./migration/Cargo.toml
COPY src/lib.rs ./src/lib.rs
COPY src/main.rs ./src/main.rs
COPY pkg/src/lib.rs ./pkg/src/lib.rs
RUN cargo fetch --locked

COPY src ./src
COPY pkg/src ./pkg/src
COPY migration/src ./migration/src

RUN cargo build --release --locked --workspace --bins

FROM debian:bullseye-slim AS runtime

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl1.1 libsasl2-2 zlib1g \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /app/logs

COPY --from=builder /app/target/release/my-axum /app/my-axum
COPY --from=builder /app/target/release/worker /app/worker
COPY --from=builder /app/target/release/runbook /app/runbook
COPY --from=builder /app/target/release/migration /app/migration
COPY --from=builder /app/src/core/template /app/src/core/template
