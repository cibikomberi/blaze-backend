FROM clux/muslrust:stable AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/blaze blaze
EXPOSE 8080
CMD ["./blaze"]
