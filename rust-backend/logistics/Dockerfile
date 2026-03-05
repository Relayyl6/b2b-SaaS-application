FROM rust:1.84 as builder
WORKDIR /app
COPY . .
WORKDIR /app/rust-backend/logistics
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/rust-backend/logistics/target/release/logistics /usr/local/bin/logistics
EXPOSE 3005
CMD ["logistics"]
