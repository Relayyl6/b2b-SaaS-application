# Project: Rust Multi-Tenant SaaS Backend

## Architecture Overview
The platform transforms the single-tenant Rust backend into a multi-tenant SaaS application.
Components:
- **API Gateway / Middleware**: Centralized authentication, scoped API key validation, tier-based usage metering, tenant context extraction & downstream header/extension injection.
- **Database Layer**: PostgreSQL hybrid multi-tenancy. Shared database with Row-Level Security (RLS) for free/low tiers; dedicated DB connection pools for enterprise tiers. Domain tables have `tenant_id` foreign keys and proper index structures.
- **Event Mesh Layer**: RabbitMQ / Redis Streams. Event payloads enriched with originating `tenant_id`. Consumer microservices validate context before processing.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | Centralized Tenant & Auth Middleware (R1) | API Gateway middleware, API key scoping, usage limits, context injection | None | IN_PROGRESS |
| M2 | Hybrid Database Multi-Tenancy (R2) | Postgres RLS, Enterprise DB pools, tenant_id FKs, sqlx prepare | M1 | PLANNED |
| M3 | Tenant-Aware Event Mesh (R3) | Event payload enrichment, consumer tenant validation | M1 | PLANNED |
| M4 | E2E Testing Track | Requirement-driven test suite (Tiers 1-4, Tier 5 adversarial) | None | IN_PROGRESS |

## Interface Contracts

### Gateway -> Downstream Context Injection
- Extracted Headers / Extensions:
  - `X-Tenant-ID`: UUID string representing tenant ID
  - `X-Tenant-Tier`: String (`free`, `standard`, `enterprise`)
  - `X-Feature-Flags`: Comma-separated or JSON list of enabled flags

### Auth Middleware Response Statuses
- Valid API Key & Within Usage Limit: Proceed (200 OK downstream)
- Missing or Invalid API Key: `401 Unauthorized`
- Tier Usage Limit Exceeded: `402 Payment Required`

### Database Multi-Tenancy Interface
- Shared DB: Execute `SET LOCAL app.current_tenant_id = '...'` before query execution for RLS policy enforcement.
- Enterprise Tier: Route query / pool connection to tenant-dedicated connection pool based on tenant tier context.

### Event Mesh Payload Interface
- Payload standard header/wrapper:
  ```json
  {
    "event_id": "...",
    "tenant_id": "...",
    "event_type": "...",
    "timestamp": "...",
    "data": { ... }
  }
  ```
- Consumer validation: verify `event.tenant_id == expected_context_tenant_id`.

## Code Layout
- Existing codebase structure to be explored by subagents.
