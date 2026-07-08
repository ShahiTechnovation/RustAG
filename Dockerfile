# Multi-stage build for the RustAG binaries: the `rustag` stagenet runtime and
# the `rustag-cloud` control plane. TLS is rustls (no OpenSSL), and SQLite is
# bundled, so the runtime image needs only ca-certificates.

# Base image matches rust-toolchain.toml (1.96.0) so no toolchain re-download.
FROM rust:1.96-bookworm AS build
WORKDIR /app
COPY . .
RUN cargo build --release -p rustag-cli -p rustag-cloud

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/rustag /usr/local/bin/rustag
COPY --from=build /app/target/release/rustag-cloud /usr/local/bin/rustag-cloud
WORKDIR /data

# JSON-RPC, WebSocket, REST API.
EXPOSE 8899 8900 9000
# Control-plane HTTP (when running rustag-cloud).
EXPOSE 8080

# Bind the public REST API on all interfaces so a platform proxy can reach it
# (only the API port honors this; the JSON-RPC/WS servers stay on loopback). The
# API port also follows $PORT when the host injects one (Render/Heroku). JSON
# logs suit container log shippers.
ENV RUSTAG_BIND_HOST=0.0.0.0 \
    RUSTAG_LOG_FORMAT=json

# Default to a one-shot demo stagenet: create-if-needed on the (mounted) /data
# volume, preload Pyth/Raydium/token from mainnet, then serve. For a safe PUBLIC
# demo also set RUSTAG_DEMO_MODE=1 and RUSTAG_MAINNET_RPC=<your key> (see
# render.yaml). To run the control plane instead, override the entrypoint:
#   docker run --rm -p 8080:8080 --entrypoint rustag-cloud <image> serve
ENTRYPOINT ["rustag"]
CMD ["serve"]
