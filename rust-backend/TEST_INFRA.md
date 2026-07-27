# Multi-Tenant B2B SaaS E2E Test Infrastructure & Architecture (`TEST_INFRA.md`)

## 1. Executive Summary & Architecture Overview

This document specifies the end-to-end (E2E) testing framework for the B2B SaaS Multi-Tenant Platform. The E2E test infrastructure validates system security, tenant isolation, authentication, usage metering, database Row-Level Security (RLS), and asynchronous event distribution across all microservices.

```
                  +--------------------------------------------------+
                  |               API Gateway / Nginx                |
                  |  - Rate Limiting (10r/s) & Quota Enforcement     |
                  |  - Header Context Injection (X-Tenant-Id, Tier)  |
                  +-------------------------+------------------------+
                                            |
                                            v
                  +--------------------------------------------------+
                  |       Platform Auth & Tenant Middleware          |
                  |  - API Key Resolution & Redis Usage Metering     |
                  |  - JWT Claims Validation & Tenant Context        |
                  +-------------------------+------------------------+
                                            |
                         +------------------+------------------+
                         |                                     |
                         v                                     v
         +-------------------------------+     +-------------------------------+
         |     PostgreSQL Database       |     |     Event Mesh (Redis/RMQ)    |
         |  - RLS Policies on `orders`   |     |  - Tenant-Enriched Payload    |
         |  - app.current_tenant_id      |     |  - Consumer Mismatch Reject   |
         |  - Compile Check (sqlx)       |     |  - Dead Letter Queue (DLQ)    |
         +-------------------------------+     +-------------------------------+
```

---

## 2. Feature Inventory & Multi-Tenant Acceptance Criteria

| Feature Area | Architectural Component | Mandatory Acceptance Criteria |
|---|---|---|
| **Gateway Auth & Usage Metering** | `platform::middleware::TenantAuthMiddleware`, API Gateway | 1. **Valid Token/Key**: Returns `200 OK` and injects `TenantContext` (`X-Tenant-Id`, `X-Tenant-Tier`).<br>2. **Invalid/Missing Token**: Returns `401 Unauthorized`.<br>3. **Usage Exceeded**: Requests exceeding tier limits (e.g. Free Tier > 100 orders/mo) return `402 Payment Required`. |
| **Database Isolation (RLS)** | PostgreSQL RLS (`orders` table), `sqlx` | 1. **Query Scoping**: Queries under `Tenant A` session (`SET LOCAL app.current_tenant_id = '...'`) return only Tenant A rows; Tenant B rows return 0 records.<br>2. **SQLx Verification**: All parameterized queries pass static schema check (`cargo sqlx prepare --check`). |
| **Event Isolation & Mesh** | `platform::streams::StreamPublisher`, Redis / RabbitMQ | 1. **Payload Enrichment**: Published domain events contain verified `tenant_id` and HMAC envelope signature.<br>2. **Consumer Rejection & DLQ**: Subscribing consumers reject events with mismatched `tenant_id` and route them to `stream:dlq`. |

---

## 3. 4-Tier Test Suite Architecture & Methodology

The E2E test suite (`e2e-tests/`) is structured into four distinct test tiers to ensure full lifecycle verification:

```
e2e-tests/
├── src/
│   ├── lib.rs                          # TestHarness, DB connection pools, HTTP fixtures
│   └── test_context.rs                  # TenantContext fixtures, Mock JWT/API key generators, HMAC validators
└── tests/
    ├── event_mesh_test.rs               # Stream & AMQP basic connectivity test
    ├── tier1_feature_coverage/          # Tier 1: Single Feature Acceptance Tests (>= 5 per feature)
    │   ├── gateway_auth_tests.rs
    │   ├── db_isolation_tests.rs
    │   └── event_isolation_tests.rs
    ├── tier2_boundary_cases/            # Tier 2: Edge Cases & Boundary Thresholds (>= 5 per feature)
    │   ├── gateway_auth_boundary_tests.rs
    │   ├── db_isolation_boundary_tests.rs
    │   └── event_isolation_boundary_tests.rs
    ├── tier3_cross_feature/             # Tier 3: Pairwise Cross-Feature Interactions
    │   ├── auth_db_interaction_tests.rs
    │   ├── auth_event_interaction_tests.rs
    │   └── db_event_interaction_tests.rs
    └── tier4_real_world/                # Tier 4: End-to-End Application Scenarios
        ├── multi_tenant_lifecycle_tests.rs
        └── security_audit_attack_tests.rs
```

### Tier Descriptions & Coverage Requirements

1. **Tier 1: Feature Coverage**
   - **Focus**: Verifies base acceptance criteria for each core feature independently.
   - **Threshold**: Minimum 5 dedicated test cases per feature area (Total: 15 tests).
2. **Tier 2: Boundary & Corner Cases**
   - **Focus**: Stress-tests limits, edge conditions, invalid inputs, SQL injection attempts, null tenant payloads, exact quota limits (#100 vs #101), and high-throughput bursts.
   - **Threshold**: Minimum 5 boundary test cases per feature area (Total: 15 tests).
3. **Tier 3: Cross-Feature Interactions**
   - **Focus**: Pairwise validation between security domains: Auth + DB RLS, Auth + Event Mesh, DB RLS + Event Mesh.
   - **Threshold**: Minimum 2 multi-domain interaction tests per pairwise combination (Total: 6 tests).
4. **Tier 4: Real-World Application Scenarios**
   - **Focus**: Comprehensive multi-step scenarios simulating realistic user traffic, concurrent multi-tenant activity, quota enforcement, and multi-vector security attacks.
   - **Threshold**: 2 full end-to-end integration scenario suites.

---

## 4. Test Harness & Helper Architecture

The test harness in `e2e-tests/src/` provides reusable fixtures:

- **`TestHarness` (`lib.rs`)**:
  - Manages `PgPool`, Redis client connection pools, and `reqwest::Client`.
  - Configures target service base URLs and JWT secret context.
  - Exposes `set_tenant_session(conn, tenant_id)` to execute Postgres `SET LOCAL app.current_tenant_id = '...'`.
- **`TestContext` & Security Helpers (`test_context.rs`)**:
  - `generate_mock_jwt(user_id, tenant_id, tier, secret)`: Generates valid signed JWT tokens.
  - `generate_expired_jwt(...)`: Generates expired JWT tokens for 401 tests.
  - `generate_mock_api_key(tenant_id, prefix)`: Creates deterministic tenant API keys (`sk_live_...`).
  - `create_enriched_event(...)`: Enriches event payloads with `tenant_id`, timestamp, and HMAC-SHA256 signature.
  - `validate_event_tenant_enrichment(...)`: Validates event payload tenant ID matching and HMAC integrity.

---

## 5. Quality Gates & Coverage Thresholds

| Metric | Target / Gate | Enforcement Mechanism |
|---|---|---|
| **Compilation** | 100% Clean | `cargo check -p e2e-tests` |
| **Test Compilation** | 100% Clean | `cargo test -p e2e-tests --no-run` |
| **SQL Schema Validity** | Pass | `cargo sqlx prepare --check` |
| **Tier 1 Feature Tests** | >= 15 passing tests | `cargo test -p e2e-tests --test '*_tests'` |
| **Tier 2 Boundary Tests** | >= 15 passing tests | `cargo test -p e2e-tests --test '*_boundary_tests'` |
| **Tier 3 Interaction Tests** | >= 6 passing tests | `cargo test -p e2e-tests --test '*_interaction_tests'` |
| **Tier 4 Scenario Tests** | 2 full scenario suites | `cargo test -p e2e-tests --test 'multi_tenant_*' --test 'security_*'` |

---

## 6. Execution Instructions

### Run Complete E2E Suite
```bash
cargo test -p e2e-tests -- --nocapture
```

### Run Specific Test Tiers
```bash
# Tier 1 Feature Coverage
cargo test -p e2e-tests --test gateway_auth_tests --test db_isolation_tests --test event_isolation_tests

# Tier 2 Boundary Cases
cargo test -p e2e-tests --test gateway_auth_boundary_tests --test db_isolation_boundary_tests --test event_isolation_boundary_tests

# Tier 3 Cross-Feature Interactions
cargo test -p e2e-tests --test auth_db_interaction_tests --test auth_event_interaction_tests --test db_event_interaction_tests

# Tier 4 Real-World Scenarios
cargo test -p e2e-tests --test multi_tenant_lifecycle_tests --test security_audit_attack_tests
```
