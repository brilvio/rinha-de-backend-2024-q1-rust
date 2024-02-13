FROM rust:1.76-buster as builder

WORKDIR /app

ENV SQLX_OFFLINE=true

COPY . .
COPY .sqlx ./.sqlx

# RUN apk update
# RUN apk add --no-cache pkgconfig openssl openssl-dev musl-dev

# RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release

FROM debian:buster-slim

WORKDIR /usr/local/bin

COPY --from=builder /app/target/release/rinha-de-backend-2024-q1 .

RUN apt-get update && apt install -y openssl

CMD ["./rinha-de-backend-2024-q1"]