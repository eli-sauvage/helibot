FROM rust

WORKDIR /app/helibot/


COPY Cargo.toml Cargo.toml
COPY src/ src/
COPY views/ views/
COPY .env .env
COPY Rocket.toml Rocket.toml


RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/helibot/target \
    cargo install --path .

CMD ["helibot"]
