FROM rust:1.80-bullseye AS builder
WORKDIR /app

# Copy the entire workspace
COPY ./rust-backend /app

# Build all binaries in the workspace (release mode)
RUN cargo build --release

# Runtime stage: minimal debian image
FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy all compiled binaries from the builder stage
COPY --from=builder /app/target/release/analytics /usr/local/bin/
COPY --from=builder /app/target/release/inventory-service /usr/local/bin/
COPY --from=builder /app/target/release/logistics /usr/local/bin/
COPY --from=builder /app/target/release/notifications /usr/local/bin/
COPY --from=builder /app/target/release/order-service /usr/local/bin/
COPY --from=builder /app/target/release/payments /usr/local/bin/
COPY --from=builder /app/target/release/product-catalog /usr/local/bin/
COPY --from=builder /app/target/release/supplier-management /usr/local/bin/
COPY --from=builder /app/target/release/tenant-management /usr/local/bin/
COPY --from=builder /app/target/release/user-management /usr/local/bin/

# Make them executable
RUN chmod +x /usr/local/bin/*

# Default command (will be overridden by docker-compose)
CMD ["echo", "Please specify a service command in docker-compose.yml"]
