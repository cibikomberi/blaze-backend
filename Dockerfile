FROM rust:1.89 AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y \
    libpq-dev pkg-config musl-tools

COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y libpq5 ca-certificates
COPY --from=builder /app/target/release/blaze .
EXPOSE 8080
CMD ["./blaze"]
