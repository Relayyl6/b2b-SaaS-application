FROM rust:1.80 as builder
WORKDIR /app
COPY . .
WORKDIR /app/rust-backend/supplier-management
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN useradd -m -s /bin/bash appuser
WORKDIR /app
COPY --from=builder /app/rust-backend/supplier-management/target/release/supplier-management /usr/local/bin/supplier-management
RUN chown -R appuser:appuser /app && chmod +x /usr/local/bin/supplier-management
USER appuser
EXPOSE 3011
CMD ["supplier-management"]
