FROM rust:1.84 as builder
WORKDIR /app
COPY . .
WORKDIR /app/rust-backend/user-management
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/rust-backend/user-management/target/release/user-management /usr/local/bin/user-management
EXPOSE 3004
CMD ["user-management"]
