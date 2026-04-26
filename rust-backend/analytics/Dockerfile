FROM rust:latest as builder
WORKDIR /app
COPY . .
WORKDIR /app/rust-backend/analytics
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/rust-backend/analytics/target/release/analytics /usr/local/bin/analytics
EXPOSE 3007
CMD ["analytics"]
