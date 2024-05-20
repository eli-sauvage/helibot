FROM rust

ENV SQLX_OFFLINE true

WORKDIR /app/helibot/

COPY Cargo.toml Cargo.toml
COPY src/ src/
COPY views/ views/
COPY .env .env

#/!\ please make sure to run `cargo sqlx prepare` before
COPY .sqlx .sqlx

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/helibot/target \
    cargo install --path .

CMD ["helibot"]
