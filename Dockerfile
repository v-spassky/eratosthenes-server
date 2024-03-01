FROM rust:1.76 as builder
WORKDIR /usr/src/app
RUN apt-get update && apt-get install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl
COPY Cargo.toml Cargo.lock ./
COPY src src
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM alpine:3.17
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/eratosthenes-server /usr/local/bin/eratosthenes-server
EXPOSE 3030
CMD ["eratosthenes-server"]
