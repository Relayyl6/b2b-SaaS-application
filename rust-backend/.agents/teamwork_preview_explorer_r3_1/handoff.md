# Milestone R3: Tenant-Aware Event Mesh — Investigation Handoff Report

## 1. Observation

A comprehensive analysis of the event messaging infrastructure across the workspace was conducted. Below are the verbatim observations, file paths, line numbers, and existing implementations.

### 1.1 Shared Platform Streaming Architecture (`platform/src/streams.rs`)
- **`StreamPublisher` (`platform/src/streams.rs:9-98`)**:
  - `publish` (lines 39-66) serializes message payload via `serde_json::to_string(message)` and executes Redis command:
    ```rust
    redis::cmd("XADD")
        .arg(stream)
        .arg("*")
        .arg("event_type")
        .arg(event_type)
        .arg("payload")
        .arg(payload)
        .query_async(&mut *conn)
        .await?;
    ```
  - `publish_async` (lines 68-98) spawns a background tokio task. If stream publication fails, it routes the message to Dead Letter Queue stream `stream:dlq` (lines 84-94).
- **`StreamEnvelope<T>` (`platform/src/streams.rs:14-20`)**:
  ```rust
  #[derive(Debug, Clone)]
  pub struct StreamEnvelope<T> {
      pub stream: String,
      pub id: String,
      pub event_type: String,
      pub payload: T,
  }
  ```
  *(Note: `StreamEnvelope` currently lacks a `tenant_id` field).*
- **`stream_for_event` (`platform/src/streams.rs:101-113`)**:
  - Maps event prefixes to stream keys: `product.` -> `"stream:products"`, `order.` -> `"stream:orders"`, `inventory.` -> `"stream:inventory"`, `logistics.` -> `"stream:logistics"`, `payment.` -> `"stream:payments"`, `user.` -> `"stream:users"`, `supplier.` | `tenant.` -> `"stream:suppliers"`, `notification.` -> `"stream:notifications"`, fallback -> `"stream:platform"`.
- **`consume_json` & `parse_stream_reply` (`platform/src/streams.rs:137-256`)**:
  - Consumes Redis Stream messages using `XREADGROUP` (lines 158-172).
  - `parse_stream_reply::<T>` (lines 194-256) extracts field values for `"event_type"` and `"payload"`, deserializes payload string into `T` via `serde_json::from_str::<T>(payload)`, and returns `Vec<StreamEnvelope<T>>`.
  - Automatically acknowledges messages (`XACK`) when handler returns `Ok(())` (lines 180-187).

### 1.2 Event Definitions Across Microservices
The workspace defines event structs across individual service crates. None of these structs currently possess a unified `tenant_id` field:

1. **`order-service`**:
   - `OrderEvent` (`order-service/src/models.rs:70-97`):
     ```rust
     pub struct OrderEvent {
         pub event_type: String,
         pub product_id: Uuid,
         pub supplier_id: Uuid,
         pub name: Option<String>,
         pub description: Option<String>,
         pub price: Option<f64>,
         pub category: Option<String>,
         pub low_stock_threshold: Option<i32>,
         pub unit: Option<String>,
         pub quantity_change: Option<i32>,
         pub available: Option<bool>,
         pub order_id: Option<Uuid>,
         pub quantity: Option<i32>,
         pub reservation_id: Option<Uuid>,
         pub timestamp: DateTime<Utc>,
         pub order_timestamp: Option<DateTime<Utc>>,
         pub expires_at: DateTime<Utc>,
         pub user_id: Option<Uuid>,
         pub status: Option<OrderStatus>,
         pub notification_channel: Option<String>,
         pub refund_amount: Option<f64>,
     }
     ```
2. **`inventory-management`**:
   - `ProductEvent` (`inventory-management/src/models.rs:65-85`): Contains product, inventory, and order details. Missing `tenant_id`.
   - `StockUpdateEvent` (`inventory-management/src/models.rs:37-50`): Stock level updates. Missing `tenant_id`.
3. **`product-catalog`**:
   - `ProductEvent` (`product-catalog/src/models.rs:113-133`): Product lifecycle events (`product.created`, `product.updated`, `product.deleted`). Missing `tenant_id`.
4. **`logistics`**:
   - `LogisticsEvent` (`logistics/src/models.rs:68-79`): Shipment lifecycle (`logistics.shipment_created`, `logistics.shipment_updated`, `logistics.shipment_cancelled`). Missing `tenant_id`.
   - `IncomingOrderEvent` (`logistics/src/models.rs:81-89`): Incoming order event consumer model. Missing `tenant_id`.
5. **`payments`**:
   - `PaymentEvent` (`payments/src/models.rs:60-74`): Payment lifecycle events (`payment.success`, `payment.failed`). Missing `tenant_id`.
   - `OrderContextEvent` (`payments/src/redis_sub.rs:11-19`): Consumer payload for order/inventory events. Missing `tenant_id`.
6. **`notifications`**:
   - `DomainEvent` (`notifications/src/models.rs:83-96`): Consumer model for domain notifications. Missing `tenant_id`.
7. **`analytics`**:
   - `AnalyticsEvent` (`analytics/src/models.rs:10-38`): Raw event struct used for timescale analytics ingestion. Missing `tenant_id`.
   - `Event` (`analytics/src/models.rs:43-49`): RabbitMQ topic event wrapper.
8. **`supplier-management`**:
   - `SupplierEvent` (`supplier-management/src/models.rs:59-67`): Supplier onboarding and status updates. Missing `tenant_id`.
9. **`user-management`**:
   - `UserCreatedEvent` (`user-management/src/unprotected/handlers.rs:34-40`): Inline struct for user signup events. Missing `tenant_id`.
   - `PasswordResetRequestedEvent` (`user-management/src/unprotected/handlers.rs:210-214`): Inline struct for password resets. Missing `tenant_id`.

### 1.3 Message Queue & Publisher / Consumer Implementations

#### Redis Streams Publishers
- **`user-management`**: `sign_up_user` (`user-management/src/unprotected/handlers.rs:48-57`) publishes `user.created` directly via `platform::streams::StreamPublisher`.
- **`inventory-management`**: `RedisPublisher` (`inventory-management/src/redis_pub.rs`) wraps `StreamPublisher`. Publishes `inventory.reserved`, `inventory.rejected`, `inventory.expired`, `inventory.released`, `inventory.finalized`, `inventory.updated`, `inventory.lowstock` (`inventory-management/src/redis_sub/events.rs`).
- **`logistics`**: `RedisPublisher` (`logistics/src/publisher.rs`). Publishes `logistics.shipment_created`, `logistics.shipment_cancelled` (`logistics/src/redis_sub.rs:100,124`).
- **`order-service`**: `RedisPublisher` (`order-service/src/redis_pub.rs`). Publishes `order.created`, `order.failed`, `order.confirmed`, `order.cancelled`, `order.shipped`, `order.delivered`, `order.pending`, `order.refunded`, `order.processing`, `order.review_requested`, `inventory.release_command`, `payment.refund_command`, `logistics.shipment_preparation_command` (`order-service/src/routes.rs:72-315`).

#### RabbitMQ Integration
- **`analytics/src/publisher.rs:17-52`**: `publish_example_event(ev: Event)` publishes `Event` to `analytics_events_topic` exchange (lapin Topic exchange).
- **`logistics/src/rabbit_pub.rs:20-73`**: `RabbitPublisher` publishes `LogisticsEvent` to `analytics_events_topic`.
- **`product-catalog/src/rabbit_pub.rs:28-106`**: `publish_example_event(ev: &ProductEvent)` publishes `ProductEvent` to `analytics_events_topic`.
- **`analytics/src/worker/consumer.rs:20-205`**: `RabbitConsumer` declares durable queue `analytics_queue` with DLQ routing `analytics_dlq` bound to topic exchange `analytics_events_topic` with routing key `#`. Consumes messages, inspects `x-retries` header, and inserts events into TimescaleDB.

#### Redis Streams Consumers
- **`analytics/src/worker/redis_consumer.rs:38-91`**: Listens to 26 event types on group `"analytics"` and inserts events into TimescaleDB.
- **`inventory-management/src/redis_sub.rs:32-91`**: Listens to 10 event types on group `"inventory-management"`.
- **`logistics/src/redis_sub.rs:14-56`**: Listens to 3 event types on group `"logistics"`.
- **`notifications/src/redis_sub.rs:30-122`**: Listens to 17 event types on group `"notifications"`.
- **`order-service/src/redis_sub.rs:31-65`**: Listens to 10 event types on group `"order-service"`.
- **`payments/src/redis_sub.rs:23-58`**: Listens to 5 event types on group `"payments"`.

---

## 2. Logic Chain

1. **Premise 1**: Multi-tenant B2B SaaS applications require strictly isolated event streaming so that microservices only process business events belonging to the active tenant, preventing cross-tenant data contamination or unauthorized actions.
2. **Observation Step**: Inspecting `platform/src/streams.rs`, all Redis Stream field additions (`XADD`) write only `event_type` and `payload`. Inspecting event structs across all 9 microservices (`OrderEvent`, `ProductEvent`, `LogisticsEvent`, `PaymentEvent`, `DomainEvent`, `AnalyticsEvent`, `SupplierEvent`), none contain a `tenant_id` field. Inspecting RabbitMQ implementations (`analytics_events_topic`), published messages contain no AMQP headers or payload attributes identifying `tenant_id`.
3. **Deduction 1**: Currently, events published across microservices lack originating `tenant_id` metadata in both message headers/envelopes and event payloads.
4. **Observation Step**: Inspecting all microservice consumer handlers (`inventory-management/src/redis_sub.rs`, `order-service/src/redis_sub.rs`, `logistics/src/redis_sub.rs`, `notifications/src/redis_sub.rs`, `payments/src/redis_sub.rs`, `analytics/src/worker/redis_consumer.rs`), consumers deserialize incoming payloads into generic event models without performing any tenant context validation.
5. **Deduction 2**: Currently, consumers execute state updates (e.g. reserving stock, updating order status, processing payments, creating shipments) blindly on any event received, regardless of tenant boundary.
6. **Conclusion**: To build a Tenant-Aware Event Mesh:
   - Shared platform envelope infrastructure (`platform/src/streams.rs`) and RabbitMQ headers must support `tenant_id`.
   - All domain event structs must be enriched with `pub tenant_id: Uuid` (or `Option<Uuid>`).
   - Event publishers must extract tenant context (from auth tokens/headers or entity state) and populate `tenant_id`.
   - Consumer validation logic must be introduced to verify `event.tenant_id` against the target service's tenant context before executing business logic, dropping or DLQ-routing mismatched events.

---

## 3. Caveats

- **Backward Compatibility**: Existing database tables or stored event logs may contain historic events published without a `tenant_id`. Migration logic must allow `Option<Uuid>` or set a fallback `tenant_id` (e.g. system default tenant) during transition.
- **Service Identity**: Some services like `analytics` or `notifications` consume events across all tenants (multi-tenant aggregator services). For these services, tenant validation should verify that `event.tenant_id` is present and valid, while tenant-isolated services (`inventory-management`, `order-service`, `logistics`, `payments`) must validate against their specific tenant scope.
- **Supplier vs Tenant Mapping**: In some service domain models, `supplier_id` represents the business entity acting as the tenant. The implementation plan must establish a clear mapping between `tenant_id` and `supplier_id` where applicable.

---

## 4. Conclusion & Step-by-Step Implementation Plan

### Final Assessment
The backend currently possesses a functional Redis Streams and RabbitMQ event-driven messaging architecture, but lacks tenant identity propagation and consumer validation. Achieving Milestone R3 requires enriching shared streaming envelopes, domain event structs, publishers, and consumer validation handlers.

### Step-by-Step Implementation Plan

#### Phase 1: Shared Platform Envelope & Redis Stream Infrastructure (`platform/src/streams.rs`)
1. **Extend `StreamEnvelope<T>`**:
   Update `StreamEnvelope<T>` in `platform/src/streams.rs`:
   ```rust
   #[derive(Debug, Clone)]
   pub struct StreamEnvelope<T> {
       pub stream: String,
       pub id: String,
       pub event_type: String,
       pub tenant_id: Option<Uuid>,
       pub payload: T,
   }
   ```
2. **Update `StreamPublisher::publish` and `publish_async`**:
   Accept `tenant_id: Option<Uuid>` (or `tenant_id: Uuid`) and write it into the Redis Stream fields:
   ```rust
   let tenant_str = tenant_id.map(|id| id.to_string()).unwrap_or_default();
   redis::cmd("XADD")
       .arg(stream)
       .arg("*")
       .arg("event_type").arg(event_type)
       .arg("tenant_id").arg(&tenant_str)
       .arg("payload").arg(payload)
       .query_async(&mut *conn)
       .await?;
   ```
3. **Update `parse_stream_reply`**:
   Extract `"tenant_id"` from the field map and parse it into `Option<Uuid>`, assigning it to `StreamEnvelope.tenant_id`.

#### Phase 2: Domain Event Struct Enrichment
Add `pub tenant_id: Option<Uuid>` (or `pub tenant_id: Uuid`) to all domain event structs across crates:
- `order-service/src/models.rs`: `OrderEvent`
- `inventory-management/src/models.rs`: `ProductEvent`, `StockUpdateEvent`
- `product-catalog/src/models.rs`: `ProductEvent`
- `logistics/src/models.rs`: `LogisticsEvent`, `IncomingOrderEvent`
- `payments/src/models.rs`: `PaymentEvent`, `OrderContextEvent`
- `notifications/src/models.rs`: `DomainEvent`
- `analytics/src/models.rs`: `AnalyticsEvent`
- `supplier-management/src/models.rs`: `SupplierEvent`
- `user-management/src/unprotected/handlers.rs`: `UserCreatedEvent`, `PasswordResetRequestedEvent`

#### Phase 3: RabbitMQ Header Propagation
In `analytics/src/publisher.rs`, `logistics/src/rabbit_pub.rs`, and `product-catalog/src/rabbit_pub.rs`:
- Add `x-tenant-id` header to `BasicProperties::default().with_headers(headers)`.
- In `analytics/src/worker/consumer.rs`, read `x-tenant-id` header from AMQP delivery properties.

#### Phase 4: Event Publisher Context Population
Update HTTP routes and workflow handlers where events are constructed:
- Extract tenant identity from `X-Tenant-Id` request header, JWT claims, or stored domain models (`order.supplier_id` / `order.user_id`).
- Populate `event.tenant_id` before invoking `publish` or `publish_async`.

#### Phase 5: Consumer Validation & Tenant Guarding
1. Implement tenant validation helper in `platform`:
   ```rust
   pub fn validate_tenant_event(
       envelope_tenant: Option<Uuid>,
       payload_tenant: Option<Uuid>,
       expected_tenant: Option<Uuid>,
   ) -> Result<Uuid, TenantValidationError> { ... }
   ```
2. Update consumer loop handlers (`inventory-management/src/redis_sub.rs`, `order-service/src/redis_sub.rs`, `logistics/src/redis_sub.rs`, `payments/src/redis_sub.rs`, `notifications/src/redis_sub.rs`, `analytics/src/worker/redis_consumer.rs`):
   - Check `envelope.tenant_id` / `event.tenant_id`.
   - If missing or mismatched with service tenant context:
     - Log warning: `tracing::warn!(%event_type, ?tenant_id, "Tenant mismatch or missing tenant ID — ignoring event");`
     - Increment metric: `metrics::inc_event(service_name, stream, event_type, "tenant_mismatch");`
     - Acknowledge message (`Ok(())`) or route to DLQ without executing DB modifications.
3. Database Queries: Ensure SQL queries inside consumer handlers enforce tenant scoping (`WHERE tenant_id = $x`).

---

## 5. Verification Method

### 5.1 Verification Commands
Once the implementation is applied by implementers, verify using:

1. **Workspace Compilation Check**:
   ```bash
   cargo check --workspace
   ```
   *Expected result*: Clean compilation without type errors or missing fields on event struct initializations.

2. **Workspace Unit & Integration Tests**:
   ```bash
   cargo test --workspace
   ```
   *Expected result*: All unit tests pass, including model serialization/deserialization tests.

3. **Event Mesh E2E Tests (`e2e-tests`)**:
   ```bash
   cargo test -p e2e-tests --test event_mesh_test
   ```
   *Expected result*: Tests verify that Redis Streams and RabbitMQ correctly serialize, transport, and validate `tenant_id` across events.

### 5.2 Verification Inspection Checklist
- Inspect `platform/src/streams.rs` to confirm `XADD` writes `"tenant_id"` and `StreamEnvelope` exposes `tenant_id`.
- Inspect `order-service/src/models.rs`, `inventory-management/src/models.rs`, `logistics/src/models.rs`, `payments/src/models.rs`, `notifications/src/models.rs`, `product-catalog/src/models.rs`, `supplier-management/src/models.rs`, `analytics/src/models.rs` to verify `tenant_id` field existence.
- Inspect consumer handlers in microservices to verify tenant mismatch rejection logic.

### 5.3 Invalidation Conditions
- Any domain event published without `tenant_id` field in Redis stream entry or RabbitMQ header.
- Consumer handlers executing business logic or DB writes when event `tenant_id` does not match expected tenant context.
