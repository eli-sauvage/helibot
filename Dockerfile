FROM rust:1-bookworm AS builder

ENV SQLX_OFFLINE=true

WORKDIR /app/helibot/

COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY migrations migrations

#/!\ please make sure to run `cargo sqlx prepare` before
COPY .sqlx .sqlx

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/helibot/target \
    cargo install --path . --locked

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/helibot /usr/local/bin/helibot

CMD ["helibot"]
