FROM rust:latest as builder
WORKDIR /app
COPY . .
WORKDIR /app/rust-backend/order-service
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/rust-backend/order-service/target/release/order-service /usr/local/bin/order-service
EXPOSE 3005
CMD ["order-service"]
