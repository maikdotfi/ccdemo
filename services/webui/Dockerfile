# syntax=docker/dockerfile:1.7

ARG RUST_VERSION=1.86
ARG DEBIAN_VERSION=bookworm

FROM rust:${RUST_VERSION}-${DEBIAN_VERSION} AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release -p webui

FROM gcr.io/distroless/cc-debian12 AS runtime

WORKDIR /app
COPY --from=builder /app/target/release/webui /usr/local/bin/webui

ENV RUST_LOG=info
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/webui"]

