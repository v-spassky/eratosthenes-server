FROM rust:1.76.0
WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
COPY src src
RUN cargo build --release
EXPOSE 3030
ENTRYPOINT ["./target/release/eratosthenes-server"]
