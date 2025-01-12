FROM rust:1.83.0

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
COPY src src
COPY locations.example.ndjson ./

RUN cargo build --release

EXPOSE 3030

ENTRYPOINT ["./target/release/eratosthenes-server"]
