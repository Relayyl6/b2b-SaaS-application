# E2E Testing Strategy & Architecture Report

## 1. Observation

Direct observations from examining the codebase at `c:\Users\USER\Documents\Previous\E-commerce\b2b-SaaS-application\rust-backend`:

### 1.1 Existing Test Infrastructure & Cargo Workspace
- **Workspace Cargo Configuration** (`Cargo.toml`, lines 1-15):
  ```toml
  [workspace]
  resolver = "2"
  members = [
      "analytics", "e2e-tests",
      "inventory-management",
      "logistics",
      "notifications",
      "order-service",
      "payments",
      "platform",
      "product-catalog",
      "supplier-management",
      "user-management",
      "e2e-tests", "gateway-tests",
  ]
  ```
- **Existing Integration Test Crates**:
  - `e2e-tests/tests/event_mesh_test.rs`: Tests async message publishing and consumption over Redis Streams (`platform::streams::StreamPublisher` and `consume_json`) and RabbitMQ (`lapin::Connection`).
    - Line 21: `async fn test_redis_stream_publish_and_consume()`
    - Line 89: `async fn test_rabbitmq_publish_and_consume()`
  - `gateway-tests/tests/security_tests.rs`: Tests API gateway security headers, rate limiting, JWT auth middleware, and maximum body size.
    - Line 4: `async fn test_security_headers()`
    - Line 33: `async fn test_rate_limiting()` (checks 10r/s + burst 20 returning 503/429)
    - Line 71: `async fn test_jwt_auth_middleware()` (verifies `/orders` returns `401 Unauthorized` without auth token)
    - Line 91: `async fn test_max_body_size()` (verifies 11MB payload returns `413 Payload Too Large`)
- **Existing Unit & Microservice Test Suites**:
  - `user-management` test runner output recorded in `um_test.txt` (lines 103-116):
    - 11 unit tests passing: `models::tests`, `auth::tests::test_create_jwt`, `protected::handlers::tests::test_admin_stats_handler`, `unprotected::handlers::tests::test_sign_up_password_validation_too_short`, `auth::tests::test_password_hashing_and_verification`.
  - `platform` test runner output recorded in `test_lib_out.txt` (lines 18-30):
    - 8 unit tests passing: `streams::tests::test_stream_for_event`, `streams::tests::test_streams_for_events`, `streams::tests::test_parse_stream_reply`, `observability::tests::test_init_observability`, `metrics::tests::test_init_and_handler`.

### 1.2 Running Environment & Gateway Setup
- **Nginx API Gateway Configuration** (`infra/nginx/nginx.conf`):
  - Line 9: Rate limit zone: `limit_req_zone $binary_remote_addr zone=api_limit:10m rate=10r/s;`
  - Lines 61-67: Internal auth validation endpoint:
    ```nginx
    location = /_auth {
        internal;
        proxy_pass http://users_backend/auth/validate;
        proxy_pass_request_body off;
        proxy_set_header Content-Length "";
        proxy_set_header Authorization $http_authorization;
    }
    ```
  - Line 81: Protected routes requiring authentication: `location /orders { auth_request /_auth; proxy_pass http://orders_backend; }`
- **Docker Orchestration Environment** (`docker-compose.yml`):
  - Services defined: Postgres (port 5432), TimescaleDB (port 5433), Redis (port 6379), RabbitMQ (ports 5672/15672), OpenTelemetry Collector (ports 4317/4318), Prometheus (port 9090), Nginx API Gateway (ports 80/443), and 9 domain microservices (`user-management`, `product-catalog`, `order-service`, `inventory-management`, `logistics`, `notifications`, `payments`, `supplier-management`, `analytics`).
- **Postgres Database Setup Script** (`infra/postgres/init-multiple-databases.sh`):
  - Initializes separate databases: `users`, `products`, `orders`, `inventory`, `logistics`, `notifications`, `payments`, `suppliers`, `analytics_service`.

### 1.3 Database & Schema Infrastructure
- `order-service/migrations/20231034_order_service.sql`:
  - Lines 12-24: Creates `orders` table with columns `id UUID PRIMARY KEY`, `user_id UUID`, `supplier_id UUID`, `product_id UUID`, `items JSONB`, `qty INT`, `status order_status`, `created_at`, `updated_at`, `expires_at`, `order_timestamp`.
- `saas_transformation_strategy.md`:
  - Lines 11-19: Specifies Row-Level Security (RLS) requirements for multi-tenancy:
    - Adding `tenant_id (UUID)` column to all tables.
    - Foreign key constraints `FOREIGN KEY (tenant_id) REFERENCES tenants(id)`.
    - PostgreSQL RLS policies enforcing `tenant_id` context scope per request.
  - Lines 56-60: Metering requirements:
    - Metering microservice backed by Redis checking tier limits (Free: 100 orders/mo, Growth: 10,000 orders/mo, Enterprise: Unlimited).
    - Returning `402 Payment Required` when quota is exceeded.

---

## 2. Logic Chain

1. **Test Infrastructure Alignment**:
   - Observation 1.1 demonstrates that the workspace already employs `e2e-tests` and `gateway-tests` as dedicated integration test targets in `Cargo.toml`.
   - Redis Streams (`platform::streams`) and Nginx `auth_request /_auth` are established integration patterns in the project.
   - Therefore, the 4-tier E2E testing framework should be structured inside `e2e-tests/tests/`, referencing `platform` and shared HTTP/DB utilities.

2. **Gateway Auth & Usage Metering Logic**:
   - Observation 1.2 shows Nginx delegating authentication to `/_auth` (`user-management/auth/validate`).
   - `saas_transformation_strategy.md` specifies that API keys/JWTs resolve into a `TenantContext` containing `tenant_id`, `tier`, and usage limits.
   - When requests exceed quota limits, the gateway/metering layer returns `402 Payment Required`.
   - Therefore, Gateway Auth tests must cover valid token forwarding (`200 OK` + `TenantContext` injection), missing/invalid token rejection (`401 Unauthorized`), and quota overflow (`402 Payment Required`).

3. **Database Isolation & RLS Logic**:
   - Observation 1.3 shows the current schema for `orders` and the planned RLS transformation (`tenant_id` column + PostgreSQL RLS policies).
   - SQL queries in `sqlx` require compile-time verification (`cargo sqlx prepare --check`) to guarantee column/type safety.
   - Therefore, Database Isolation tests must execute queries under explicit session contexts (`SET LOCAL app.current_tenant_id = '...'`) to verify Tenant A cannot read/mutate Tenant B rows, and verify `cargo sqlx prepare` check passes.

4. **Event Isolation & Payload Enrichment Logic**:
   - Observation 1.1 (`e2e-tests/tests/event_mesh_test.rs`) and Observation 1.3 show event publishing over Redis Streams and RabbitMQ.
   - For multi-tenant event security, domain events (e.g., `OrderCreatedEvent`) must be enriched with `tenant_id`. Subscribing workers must reject mismatched tenant events and route invalid messages to a Dead-Letter Queue (DLQ).
   - Therefore, Event Isolation tests must verify payload enrichment (`tenant_id` presence), matching consumer execution, mismatched consumer rejection, and DLQ routing.

5. **4-Tier Strategy Synthesis**:
   - Structuring tests into Tier 1 (Feature Coverage), Tier 2 (Boundary & Corner Cases), Tier 3 (Cross-Feature Pairwise Interactions), and Tier 4 (Real-World Application Scenarios) ensures complete test coverage from individual acceptance criteria up to complex, realistic multi-tenant traffic and attack scenarios.

---

## 3. Caveats

- **Read-Only Scope**: The current task is an architectural design and read-only investigation. No source files outside of `.agents/teamwork_preview_explorer_e2e_1` were modified.
- **Docker Dependency for Execution**: Full execution of Tier 1 through Tier 4 E2E tests requires running background infrastructure (PostgreSQL with RLS, Redis Streams, RabbitMQ, and Nginx Gateway) via `docker compose up`.
- **Database RLS Migration State**: The base migration `20231034_order_service.sql` defines the original `orders` table schema. The RLS migration adding `tenant_id` and PostgreSQL `CREATE POLICY` statements must be applied prior to executing live RLS DB integration tests.

---

## 4. Conclusion & 4-Tier Strategy Design

### 4.1 Feature Inventory & Acceptance Criteria

| Feature Area | User Requirements / Scenario | Acceptance Criteria |
|---|---|---|
| **Gateway Auth & Scoping** | Valid API Key / JWT authentication, header context injection, and quota enforcement. | 1. **Valid Auth**: Request with valid key/JWT returns `200 OK` and injects `TenantContext` (`X-Tenant-Id`, `X-Tenant-Tier`).<br>2. **Invalid/Missing Auth**: Request missing or supplying invalid/expired credentials returns `401 Unauthorized`.<br>3. **Quota Exceeded**: Request exceeding tier quota (e.g., Free Tier >100 orders/mo) returns `402 Payment Required`. |
| **Database Isolation (RLS)** | Multi-tenant row-level security isolation on PostgreSQL `orders` table and SQL compile-time checking. | 1. **RLS Query Isolation**: Query under `Tenant A` context (`app.current_tenant_id`) returns only Tenant A rows; Tenant B rows return 0 records.<br>2. **SQLx Prepare Check**: `cargo sqlx prepare --check` passes cleanly for all `tenant_id` parameterized queries. |
| **Event Isolation** | Domain event tenant enrichment and consumer validation/rejection. | 1. **Payload Enrichment**: `OrderCreatedEvent` payload contains valid `tenant_id` in payload and stream metadata.<br>2. **Consumer Rejection**: Subscribing consumer rejects/drops events with mismatched `tenant_id` and routes them to DLQ (`stream:dlq`). |

---

### 4.2 Comprehensive 4-Tier Test Suite Design

#### **Tier 1: Feature Coverage (≥5 tests per feature)**

##### Feature 1: Gateway Auth & Scoping (5 tests)
1. `test_gateway_auth_valid_api_key_returns_200_and_context`: Submits HTTP request with valid API key `sk_live_tenant_a_123`. Asserts HTTP `200 OK` and inspects response header/body confirming `X-Tenant-Id` context injection.
2. `test_gateway_auth_missing_header_returns_401`: Submits HTTP request to protected route `/orders` without `Authorization` or `X-API-Key` headers. Asserts HTTP `401 Unauthorized`.
3. `test_gateway_auth_invalid_api_key_returns_401`: Submits HTTP request with malformed key `sk_invalid_999`. Asserts HTTP `401 Unauthorized`.
4. `test_gateway_auth_expired_jwt_returns_401`: Submits HTTP request with expired JWT token (`exp` timestamp in past). Asserts HTTP `401 Unauthorized` with token expired message.
5. `test_gateway_auth_usage_limit_exceeded_returns_402`: Simulates 101st request on a Free Tier account (100 order limit). Asserts HTTP `402 Payment Required` with error code `USAGE_LIMIT_EXCEEDED`.

##### Feature 2: Database Isolation (RLS) (5 tests)
1. `test_db_rls_tenant_a_cannot_read_tenant_b_orders`: Executes `SELECT * FROM orders WHERE id = $1` with `app.current_tenant_id` set to Tenant A. Asserts 0 rows returned when `$1` is Tenant B's order ID.
2. `test_db_rls_tenant_a_cannot_update_tenant_b_orders`: Executes `UPDATE orders SET status = 'cancelled' WHERE id = $1` under Tenant A session context. Asserts 0 rows updated for Tenant B order.
3. `test_db_rls_tenant_a_cannot_delete_tenant_b_orders`: Executes `DELETE FROM orders WHERE id = $1` under Tenant A session context. Asserts 0 rows affected for Tenant B order.
4. `test_db_rls_insert_enforces_tenant_id_matching`: Attempts to `INSERT INTO orders (id, tenant_id, ...)` with `tenant_id = TenantB` while `app.current_tenant_id = TenantA`. Asserts RLS check constraint violation error.
5. `test_db_sqlx_prepare_check_schema_validity`: Runs SQL query static analyzer check ensuring all `orders` SQL queries with `tenant_id` compile against DB schema metadata without type mismatches.

##### Feature 3: Event Isolation (5 tests)
1. `test_event_order_created_contains_tenant_id_enrichment`: Emits `OrderCreatedEvent` via `StreamPublisher`. Deserializes stream payload and asserts `payload.tenant_id == expected_tenant_id`.
2. `test_event_consumer_processes_matching_tenant_event`: Consumer configured for Tenant A receives `OrderCreatedEvent` with `tenant_id = TenantA`. Asserts handler executes successfully.
3. `test_event_consumer_rejects_mismatched_tenant_event`: Consumer configured for Tenant A receives `OrderCreatedEvent` with `tenant_id = TenantB`. Asserts handler rejects event without processing order logic.
4. `test_event_envelope_metadata_carries_tenant_context`: Inspects raw Redis Stream envelope attributes (`XADD` args) and verifies `tenant_id` metadata key is present.
5. `test_event_mismatched_tenant_event_routed_to_dlq`: When consumer rejects mismatched event, asserts envelope is published to `stream:dlq` with field `error = tenant_mismatch`.

---

#### **Tier 2: Boundary & Corner Cases (≥5 tests per feature)**

##### Feature 1: Gateway Auth & Scoping (5 tests)
1. `test_gateway_auth_boundary_exact_usage_limit_returns_200`: Request #100 on a 100-request quota returns HTTP `200 OK`; request #101 immediately returns HTTP `402 Payment Required`.
2. `test_gateway_auth_usage_counter_reset_window`: Simulates time-window expiration for usage counter in Redis. Verifies quota resets to 0 and subsequent requests return `200 OK`.
3. `test_gateway_auth_malformed_auth_header_format`: Passes garbage string `Authorization: Bearer $$$invalid_token!` to gateway. Asserts `401 Unauthorized` without gateway process crash.
4. `test_gateway_auth_concurrent_rate_burst_handling`: Fires 30 concurrent requests under `limit_req_zone` (10r/s, burst=20). Asserts rate limiter handles burst cleanly, returning 429/503 for excess requests.
5. `test_gateway_auth_revoked_api_key_instant_invalidation`: Revokes API key in DB/Redis cache. Asserts next immediate request with revoked key changes from `200 OK` to `401 Unauthorized`.

##### Feature 2: Database Isolation (RLS) (5 tests)
1. `test_db_rls_null_or_uninitialized_tenant_context`: Executes queries on `orders` without executing `SET LOCAL app.current_tenant_id`. Asserts 0 rows returned (default-deny RLS policy).
2. `test_db_rls_sql_injection_in_tenant_id`: Passes `' OR '1'='1` string to `tenant_id` session parameter. Asserts query engine rejects malformed UUID without SQL execution bypass.
3. `test_db_rls_cross_tenant_fk_join_prevention`: Executes `SELECT * FROM orders JOIN products ON orders.product_id = products.id` where Tenant A order references Tenant B product. Asserts RLS blocks cross-tenant table join.
4. `test_db_rls_transaction_isolation_rollback`: Opens connection, sets `SET LOCAL app.current_tenant_id = TenantA`, and rolls back transaction. Asserts session state does not bleed into next connection check out.
5. `test_db_rls_bypass_attempt_via_raw_queries`: Attempts to override RLS using unauthorized `SET ROLE` or raw connection strings under standard app user role. Asserts permission denied.

##### Feature 3: Event Isolation (5 tests)
1. `test_event_null_tenant_id_payload_rejection`: Attempts to publish `OrderCreatedEvent` with `tenant_id: null`. Asserts serialization/validation failure at publisher boundary.
2. `test_event_cross_tenant_stream_poisoning`: Manually injects Tenant B payload into Tenant A's dedicated Redis stream key. Asserts consumer signature check detects stream poisoning and drops message.
3. `test_event_high_throughput_burst_multi_tenant`: Emits 1,000 mixed-tenant events rapidly. Asserts consumers process all events with 100% tenant isolation under queue pressure.
4. `test_event_consumer_reconnect_tenant_state_preservation`: Restarts consumer group worker mid-stream. Asserts reconnected consumer retains tenant filter criteria and resumes processing without context leak.
5. `test_event_malformed_json_payload_dlq`: Publishes corrupted JSON payload bearing valid `tenant_id` metadata. Asserts message is routed directly to `stream:dlq` without worker crash.

---

#### **Tier 3: Cross-Feature Interactions (Pairwise Combinations)**

##### Combination 1: Gateway Auth + Database Isolation (RLS)
1. `test_cross_auth_to_db_tenant_propagation`: Client authenticates at Gateway with Tenant A API key. Gateway validates token, injects `X-Tenant-Id: TenantA` header. `order-service` receives header and applies `SET LOCAL app.current_tenant_id = TenantA` to DB session. Verifies inserted/queried order rows are locked to Tenant A.
2. `test_cross_auth_forged_tenant_header_rejection`: Attacker sends HTTP request with valid Tenant A credentials but attaches header `X-Tenant-Id: TenantB`. Asserts API Gateway strips/overwrites forged header with authenticated Tenant A ID, preventing RLS spoofing.

##### Combination 2: Gateway Auth + Event Isolation
1. `test_cross_auth_to_event_enrichment_flow`: Client calls `POST /orders` with Tenant A auth. Service creates order and emits `OrderCreatedEvent`. Verifies published event payload automatically contains `tenant_id: TenantA` derived from authenticated context.
2. `test_cross_auth_rate_limited_event_suppression`: Tenant B exceeds order quota (HTTP `402 Payment Required`). Gateway blocks request at border. Verifies `order-service` is never invoked and no domain event is published.

##### Combination 3: Database Isolation (RLS) + Event Isolation
1. `test_cross_event_consumer_db_session_scoping`: Subscribing consumer in `inventory-management` receives `OrderCreatedEvent` with `tenant_id: TenantA`. Consumer extracts `tenant_id`, acquires DB connection, executes `SET LOCAL app.current_tenant_id = TenantA`, and reserves stock. Asserts DB write succeeds under RLS scope.
2. `test_cross_event_mismatch_prevents_db_write`: Consumer receives `OrderCreatedEvent` with mismatched `tenant_id`. Consumer drops event before database layer, asserting zero DB queries or connection lock acquisitions occur.

---

#### **Tier 4: Real-World Application Scenarios**

##### Scenario 1: Multi-Tenant E-Commerce Order Fulfillment & Quota Enforcement
- **Name**: `test_real_world_multi_tenant_fulfillment_and_quota_enforcement`
- **Description**: Simulates concurrent multi-tenant activity between Tenant A (Growth Tier) and Tenant B (Free Tier).
- **Execution Flow**:
  1. Tenant A (Growth Tier) places order via `POST /orders` with API key `sk_live_tenant_a`.
  2. API Gateway validates auth, verifies usage count < 10,000, injects `X-Tenant-Id: TenantA`, and forwards request.
  3. `order-service` sets `app.current_tenant_id = TenantA`, writes order to Postgres `orders` table, and emits `OrderCreatedEvent` with `tenant_id: TenantA`.
  4. `inventory-management` worker consumes event, validates `tenant_id == TenantA`, scopes DB session to Tenant A, reserves stock, and emits `InventoryReservedEvent`.
  5. Concurrently, Tenant B (Free Tier, 100 order limit) submits 101st order.
  6. Gateway intercepts Tenant B request, identifies usage quota overflow, and returns `402 Payment Required`.
  7. Asserts Tenant A order completes full fulfillment cycle while Tenant B order is rejected cleanly at boundary without database or event side-effects.

##### Scenario 2: Cross-Tenant Data Leakage Attack & Security Audit Stress Test
- **Name**: `test_real_world_cross_tenant_attack_resilience`
- **Description**: Simulates a malicious actor with Tenant A access attempting cross-tenant data extraction and event injection targeting Tenant B.
- **Execution Flow**:
  1. Attacker sends `GET /orders/<tenant_b_order_id>` using Tenant A JWT. Gateway validates token, injects Tenant A identity. `order-service` executes RLS query scoped to Tenant A. Database returns 0 rows; endpoint responds with `404 Not Found`.
  2. Attacker sends `POST /orders` with body payload `{ "tenant_id": "<tenant_b_uuid>", "items": [...] }`. Gateway/Service overwrites payload `tenant_id` with authenticated Tenant A identity from JWT before DB insert.
  3. Attacker bypasses HTTP API and injects forged `OrderCreatedEvent` containing Tenant B's ID directly into Redis Stream key.
  4. Consumer worker in `inventory-management` reads event envelope, performs cryptographic HMAC signature check against tenant secret, fails validation, drops message, logs security alert, and routes message to `stream:dlq`.
  5. Asserts Tenant B data remains 100% uncompromised across HTTP, Database, and Event Mesh layers.

---

### 4.3 Test File Layout & Directory Architecture

```
e2e-tests/
├── Cargo.toml
├── src/
│   ├── lib.rs                          # Shared test helpers, DB pool factories, HTTP client builders
│   └── test_context.rs                  # TenantContext mock fixtures & token generators
└── tests/
    ├── event_mesh_test.rs               # Existing Redis & RabbitMQ integration test
    ├── tier1_feature_coverage/
    │   ├── gateway_auth_tests.rs         # 5 tests for Gateway Auth
    │   ├── db_isolation_tests.rs        # 5 tests for DB RLS & sqlx prepare check
    │   └── event_isolation_tests.rs     # 5 tests for Event Enrichment & Filtering
    ├── tier2_boundary_cases/
    │   ├── gateway_auth_boundary_tests.rs# 5 boundary tests for Auth & Quotas
    │   ├── db_isolation_boundary_tests.rs# 5 boundary tests for DB RLS
    │   └── event_isolation_boundary_tests.rs # 5 boundary tests for Event Mesh
    ├── tier3_cross_feature/
    │   ├── auth_db_interaction_tests.rs # Pairwise Auth + DB RLS tests
    │   ├── auth_event_interaction_tests.rs # Pairwise Auth + Event tests
    │   └── db_event_interaction_tests.rs # Pairwise DB RLS + Event tests
    └── tier4_real_world/
        ├── multi_tenant_lifecycle_tests.rs # End-to-end multi-tenant order fulfillment
        └── security_audit_attack_tests.rs  # Cross-tenant attack resilience scenario
```

---

### 4.4 Test Runner Commands

- **Run Entire E2E Workspace Test Suite**:
  ```bash
  cargo test -p e2e-tests --test '*' -- --nocapture
  ```
- **Run Specific Tier Suites**:
  ```bash
  # Tier 1 Feature Coverage
  cargo test -p e2e-tests --test gateway_auth_tests -- --nocapture
  cargo test -p e2e-tests --test db_isolation_tests -- --nocapture
  cargo test -p e2e-tests --test event_isolation_tests -- --nocapture

  # Tier 2 Boundary Cases
  cargo test -p e2e-tests --test gateway_auth_boundary_tests -- --nocapture
  cargo test -p e2e-tests --test db_isolation_boundary_tests -- --nocapture
  cargo test -p e2e-tests --test event_isolation_boundary_tests -- --nocapture

  # Tier 3 Cross-Feature
  cargo test -p e2e-tests --test auth_db_interaction_tests -- --nocapture

  # Tier 4 Real-World Scenarios
  cargo test -p e2e-tests --test multi_tenant_lifecycle_tests -- --nocapture
  cargo test -p e2e-tests --test security_audit_attack_tests -- --nocapture
  ```
- **Execute SQLx Compile-Time Query Verification**:
  ```bash
  cargo sqlx prepare --check --workspace
  ```

---

### 4.5 Step-by-Step Test Creation Plan

1. **Step 1: Test Infrastructure Framework Setup (`e2e-tests/src/lib.rs`)**:
   - Define `TestHarness` struct providing database connection pools (`PgPool`), Redis clients (`deadpool_redis::Pool`), and HTTP clients (`reqwest::Client`).
   - Implement helper `setup_tenant_session(pool: &PgPool, tenant_id: Uuid)` to execute `SET LOCAL app.current_tenant_id`.
   - Implement helper `generate_test_token(tenant_id: Uuid, tier: Tier)` for API Key/JWT mock generation.

2. **Step 2: Implement Tier 1 Feature Coverage Tests**:
   - Create `e2e-tests/tests/tier1_feature_coverage/gateway_auth_tests.rs` with 5 auth tests (valid 200, missing 401, invalid 401, expired 401, quota exceeded 402).
   - Create `e2e-tests/tests/tier1_feature_coverage/db_isolation_tests.rs` with 5 DB isolation tests (read isolation, update isolation, delete isolation, insert RLS enforcement, and `cargo sqlx prepare` check).
   - Create `e2e-tests/tests/tier1_feature_coverage/event_isolation_tests.rs` with 5 event tests (enrichment, matching consumer, mismatched rejection, metadata check, DLQ routing).

3. **Step 3: Implement Tier 2 Boundary & Corner Case Tests**:
   - Create boundary test files in `e2e-tests/tests/tier2_boundary_cases/` for exact limit #100 vs #101, counter reset, SQL injection prevention, cross-tenant joins, null tenant payloads, and high-throughput multi-tenant event bursts.

4. **Step 4: Implement Tier 3 Cross-Feature Pairwise Interaction Tests**:
   - Create pairwise interaction test files in `e2e-tests/tests/tier3_cross_feature/` covering Gateway Auth + DB RLS, Gateway Auth + Event Mesh, and DB RLS + Event Mesh.

5. **Step 5: Implement Tier 4 Real-World Application Scenario Tests**:
   - Create `multi_tenant_lifecycle_tests.rs` and `security_audit_attack_tests.rs` in `e2e-tests/tests/tier4_real_world/`.

6. **Step 6: Test Suite Validation & CI Pipeline Wire-Up**:
   - Run `cargo test -p e2e-tests --test '*' -- --nocapture`.
   - Run `cargo sqlx prepare --check --workspace`.
   - Verify zero test failures and confirm full coverage across all 4 tiers.

---

## 5. Verification Method

To independently verify this strategy report:
1. **File Inspection**:
   - Inspect report path: `c:\Users\USER\Documents\Previous\E-commerce\b2b-SaaS-application\rust-backend\.agents\teamwork_preview_explorer_e2e_1\handoff.md`.
2. **Cargo Workspace Verification**:
   - Verify workspace membership of `e2e-tests` and `gateway-tests` in `c:\Users\USER\Documents\Previous\E-commerce\b2b-SaaS-application\rust-backend\Cargo.toml`.
3. **Existing Integration Tests Execution**:
   - Execute command (requires Docker stack running):
     ```bash
     cargo test -p e2e-tests --test event_mesh_test -- --nocapture
     cargo test -p gateway-tests --test security_tests -- --nocapture
     ```
4. **Invalidation Conditions**:
   - Report is invalidated if `Cargo.toml` structure changes, or if Gateway Auth, RLS database schema, or Event Stream structures deviate from the specifications outlined in `saas_transformation_strategy.md` and `ARCHITECTURE.md`.
