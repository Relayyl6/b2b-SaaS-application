FROM rust:latest as builder
WORKDIR /app
COPY . .
WORKDIR /app/rust-backend/inventory-management
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/rust-backend/inventory-management/target/release/inventory-management /usr/local/bin/inventory-management
EXPOSE 3006
CMD ["inventory-management"]
