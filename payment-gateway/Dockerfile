FROM rust:1.67 as builder

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/payment_gateway /usr/local/bin/payment_gateway

EXPOSE 8080

CMD ["payment_gateway"]
