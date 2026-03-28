FROM rust:slim AS builder

RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*
RUN rustup target add $(uname -m)-unknown-linux-musl

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target $(uname -m)-unknown-linux-musl
RUN rm -rf src

COPY src ./src
RUN touch src/main.rs
RUN cargo build --release --target $(uname -m)-unknown-linux-musl && \
    cp target/$(uname -m)-unknown-linux-musl/release/nowplaying /app/nowplaying

FROM alpine:3

RUN apk add --no-cache ca-certificates \
    && adduser -D -H nowplaying

COPY --from=builder /app/nowplaying /usr/local/bin/

USER nowplaying
EXPOSE 3000

CMD ["nowplaying"]
