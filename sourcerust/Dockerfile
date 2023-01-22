FROM rust:slim-buster as builder
RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates
COPY . /app
WORKDIR /app
RUN cargo build --target x86_64-unknown-linux-musl --release

ENV MY_FRIDAYS=6

FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/itsfriday /itsfriday
ENTRYPOINT ["/itsfriday", "/app/certs/tls.crt", "/app/certs/tls.key"]
