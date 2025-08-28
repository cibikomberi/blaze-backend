FROM rust:1.89 AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y musl-tools && rustup target add x86_64-unknown-linux-musl
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/blaze blaze
EXPOSE 8080
CMD ["./blaze"]
