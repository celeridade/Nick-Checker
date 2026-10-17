# l96a1-api — build estático Rust + runtime mínimo
FROM rust:1.99-slim-trixie AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/l96a1-core ./crates/l96a1-core
COPY crates/l96a1-proxy ./crates/l96a1-proxy
COPY crates/l96a1-dataset ./crates/l96a1-dataset
COPY crates/l96a1-checkers ./crates/l96a1-checkers
COPY crates/l96a1-api ./crates/l96a1-api
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates \
  && rm -rf /var/lib/apt/lists \
  && cargo build --release -p l96a1-api

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
  && rm -rf /var/lib/apt/lists
COPY --from=build /app/target/release/l96a1-api /app/l96a1-api
# healthcheck binário: curl leve em /health
RUN printf '#!/bin/sh\nexec curl -fsS http://127.0.0.1:8080/health >/dev/null\n' > /app/l96a1-api-healthcheck \
  && chmod +x /app/l96a1-api-healthcheck
EXPOSE 8080
CMD ["/app/l96a1-api"]
