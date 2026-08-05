FROM rust:latest as builder
WORKDIR /app
COPY . .
WORKDIR /app/rust-backend/notifications
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/rust-backend/target/release/notifications /usr/local/bin/notifications
EXPOSE 3009
CMD ["notifications"]
