# B2B SaaS Rust Backend

This repository is a Rust microservice backend for a B2B e-commerce/SaaS platform. It now has a root workspace and a shared local runtime so the services can be developed as one backend while remaining independently deployable.

## Services

- `user-management`: identity, auth, protected user actions.
- `product-catalog`: product CRUD, search, bulk creation, product assets.
- `order-service`: order lifecycle and order state.
- `inventory-management`: stock, reservation, release, finalization.
- `logistics`: shipment creation, tracking, cancellation, event orchestration.
- `notifications`: durable notification outbox, event-to-message mapping, delivery worker.
- `analytics`: event ingestion and analytical rollups.
- `payments`: idempotent payment intents and webhook/status handling.
- `supplier-management`: supplier/tenant onboarding and status lifecycle.

Read [ARCHITECTURE.md](./ARCHITECTURE.md) for the platform model, workflows, and scaling notes.

## Local Stack

Copy the environment example and start the stack:

```bash
cp .env.example .env
docker compose up --build
```

The API gateway listens on:

```text
http://localhost:8080
```

Core infrastructure:

- Postgres: `localhost:5432`
- TimescaleDB: `localhost:5433`
- Redis: `localhost:6379`
- RabbitMQ: `localhost:5672`
- RabbitMQ UI: `http://localhost:15672`

## Workspace Commands

From the repository root:

```bash
cargo check --workspace
cargo test --workspace
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
```

You can also run a single service:

```bash
cd logistics
cargo run
```

## Scaling Locally

Stateless HTTP services can be scaled with Compose:

```bash
docker compose up --build --scale product-catalog=3 --scale order-service=2
```

The workflow consumers use Redis Streams consumer groups through the shared `platform` crate. Producers still call local `RedisPublisher` wrappers, but those wrappers append to streams with `XADD` rather than using Pub/Sub. Scale consumer-backed services by keeping replicas in the same consumer group and giving each replica a unique `CONSUMER_NAME`.

Payments and supplier onboarding are part of the event flow:

- `payments` emits `payment.*` events. Inventory consumes successful, failed, and cancelled payments; notifications creates user/supplier messages from them.
- `supplier-management` emits `supplier.*` events. Notifications consumes them for onboarding and status updates.
- `logistics` creates shipments from `inventory.finalized`, so shipment creation happens after payment success rather than immediately after stock reservation.
