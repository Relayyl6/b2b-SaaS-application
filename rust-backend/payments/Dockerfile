FROM rust:latest as builder
WORKDIR /app
COPY . .
WORKDIR /app/rust-backend/payments
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN useradd -m -s /bin/bash appuser
WORKDIR /app
COPY --from=builder /app/rust-backend/target/release/payments /usr/local/bin/payments
RUN chown -R appuser:appuser /app && chmod +x /usr/local/bin/payments
USER appuser
EXPOSE 3010
CMD ["payments"]
