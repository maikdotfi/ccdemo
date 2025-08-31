# syntax=docker/dockerfile:1.7

ARG RUST_VERSION=1.86
ARG DEBIAN_VERSION=bookworm

FROM rust:${RUST_VERSION}-${DEBIAN_VERSION} AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy entire workspace (simple, reliable; not the most cache-efficient)
COPY . .

# Build pubber with GCP feature
RUN cargo build --release -p pubber --features gcp

FROM gcr.io/distroless/cc-debian12 AS runtime

WORKDIR /app
COPY --from=builder /app/target/release/pubber /usr/local/bin/pubber

ENV RUST_LOG=info
ENTRYPOINT ["/usr/local/bin/pubber"]
