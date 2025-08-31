# syntax=docker/dockerfile:1.7

ARG RUST_VERSION=1.86
ARG DEBIAN_VERSION=bookworm

# 1) Planner stage: compute dependency recipe
FROM rust:${RUST_VERSION}-${DEBIAN_VERSION} AS chef
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# 2) Builder stage: cook dependencies, then build the binary
FROM rust:${RUST_VERSION}-${DEBIAN_VERSION} AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked
WORKDIR /app

# Rebuild dependency layers only when recipe changes
COPY --from=chef /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json

# Now bring in the actual source and build the target binary
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/app/target \
    cargo build --release -p subber --features gcp \
    && cp target/release/subber /app/subber

# 3) Runtime stage: minimal image with the compiled binary
FROM gcr.io/distroless/cc-debian12 AS runtime

WORKDIR /app
COPY --from=builder /app/subber /usr/local/bin/subber

ENV RUST_LOG=info
ENTRYPOINT ["/usr/local/bin/subber"]
