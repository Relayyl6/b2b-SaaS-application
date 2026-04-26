FROM rust:latest as builder
WORKDIR /app
COPY . .
WORKDIR /app/rust-backend/product-catalog
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/rust-backend/product-catalog/target/release/product-catalog /usr/local/bin/product-catalog
EXPOSE 3003
CMD ["product-catalog"]
