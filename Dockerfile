FROM rust:slim AS builder

ARG TARGETARCH

RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*

RUN case "$TARGETARCH" in \
      amd64) echo x86_64-unknown-linux-musl ;; \
      arm64) echo aarch64-unknown-linux-musl ;; \
      riscv64) echo riscv64gc-unknown-linux-musl ;; \
    esac > /rust-target && \
    rustup target add $(cat /rust-target)

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target $(cat /rust-target)
RUN rm -rf src

COPY src ./src
RUN touch src/main.rs
RUN cargo build --release --target $(cat /rust-target) && \
    cp target/$(cat /rust-target)/release/nowplaying /app/nowplaying

FROM alpine:3

RUN apk add --no-cache ca-certificates \
    && adduser -D -H nowplaying

COPY --from=builder /app/nowplaying /usr/local/bin/

USER nowplaying
EXPOSE 3000

CMD ["nowplaying"]
