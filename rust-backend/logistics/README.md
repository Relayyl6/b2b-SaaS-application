# Logistics Service

The logistics service owns shipment orchestration for the B2B backend. It is not just a shipment table API: it reacts to inventory/order events, creates shipments when stock is reserved, enforces shipment state transitions, and emits logistics events that order, analytics, and notifications can consume.

## Responsibilities

- Create shipments manually or from `inventory.reserved` events.
- Prevent duplicate shipments per order.
- Cancel shipments when an order is cancelled.
- Track shipment status from `pending` to `intransit` to `delivered`.
- Publish shipment events to Redis and RabbitMQ for downstream consumers.
- Provide supplier-focused shipment listing with status filtering and pagination.

## Runtime

```env
DATABASE_URL=postgres://postgres:postgres@postgres:5432/logistics
REDIS_URL=redis://redis:6379
RABBITMQ_URL=amqp://guest:guest@rabbitmq:5672/%2f
SERVICE_PORT=3008
```

## HTTP API

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/health` | Health check |
| `POST` | `/shipments` | Create shipment |
| `GET` | `/shipments/{shipment_id}` | Fetch one shipment |
| `GET` | `/shipments/supplier/{supplier_id}` | List supplier shipments |
| `PUT` | `/shipments/{shipment_id}/status` | Move shipment through workflow |
| `PUT` | `/shipments/order/{order_id}/cancel` | Cancel active shipment for an order |

### Create Shipment

```json
{
  "order_id": "11111111-1111-1111-1111-111111111111",
  "user_id": "22222222-2222-2222-2222-222222222222",
  "supplier_id": "33333333-3333-3333-3333-333333333333",
  "product_id": "44444444-4444-4444-4444-444444444444",
  "notes": "Fragile package"
}
```

### Update Shipment Status

```json
{
  "status": "intransit",
  "notes": "Picked up by carrier"
}
```

Allowed transitions:

| From | To |
| --- | --- |
| `pending` | `intransit`, `cancelled` |
| `intransit` | `delivered`, `cancelled` |
| `delivered` | terminal |
| `cancelled` | terminal |

## Event Subscriptions

The service consumes:

- `inventory.reserved`: creates a shipment if one does not already exist for the order.
- `order.cancelled`: cancels the shipment for the order if it has not been delivered.

## Events Published

- `logistics.shipment_created`
- `logistics.shipment_updated`
- `logistics.shipment_cancelled`

Events are sent to Redis for operational workflows and RabbitMQ for analytics/event ingestion.

## Scaling Notes

The HTTP API can scale horizontally. The Redis listener should run as one replica while this repo uses Redis Pub/Sub, because each subscriber receives every message. To scale consumers horizontally without duplicate shipment creation, move the workflow listener to Redis Streams or Kafka consumer groups and add idempotency keys per event.

The service is CPU-light and database-bound, so horizontal scaling usually helps more than vertical scaling. Increase Postgres connection limits and tune pool sizes before adding many replicas.
