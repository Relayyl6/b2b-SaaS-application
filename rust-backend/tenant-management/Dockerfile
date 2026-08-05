FROM rust:latest as builder
WORKDIR /app
COPY . .
WORKDIR /app/rust-backend/tenant-management
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/rust-backend/target/release/tenant-management /usr/local/bin/tenant-management
EXPOSE 3000
CMD ["tenant-management"]
