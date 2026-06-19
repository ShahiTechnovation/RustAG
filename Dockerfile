# Multi-stage build for the RustAG binaries: the `rustag` stagenet runtime and
# the `rustag-cloud` control plane. TLS is rustls (no OpenSSL), and SQLite is
# bundled, so the runtime image needs only ca-certificates.

FROM rust:1.85-bookworm AS build
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

# Default to the stagenet CLI. Override the entrypoint to run the control plane:
#   docker run --rm -p 8080:8080 rustag rustag-cloud serve
ENTRYPOINT ["rustag"]
CMD ["--help"]
