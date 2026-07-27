# Master Plan — Multi-Tenant SaaS Transformation

## Objective
Transform the single-tenant Rust E-commerce backend into a multi-tenant SaaS platform featuring strict tenant isolation at DB and event levels, scoped API keys, tier-based usage metering, and robust event streaming.

## Milestones & Decompositions

### Milestone R1: Centralized Tenant & Auth Middleware
- Scoped API keys (extract tenant ID, tier limits, feature flags).
- Tier-based usage limits in API Gateway (402 Payment Required on limit exceeded).
- Inject tenant context into request scope for downstream services.
- 401 Unauthorized for missing/invalid keys.

### Milestone R2: Hybrid Database Multi-Tenancy
- Refactor PostgreSQL schema for hybrid multi-tenancy.
- Shared DB with Row-Level Security (RLS) enforcing tenant isolation for free/low tiers.
- Dedicated DB connection pool routing for enterprise tiers.
- Add `tenant_id` FK to domain tables (e.g. `orders`, etc.).
- Update `cargo sqlx prepare` metadata for updated schemas and indexes.

### Milestone R3: Tenant-Aware Event Mesh
- Enrich all events (e.g. `OrderCreatedEvent`) with originating `tenant_id`.
- RabbitMQ / Redis Streams payload isolation by tenant.
- Consumer microservices validation of tenant context before executing business logic.

### Parallel Track: E2E Testing Suite
- Opaque-box requirement-driven test suite.
- Tiers 1-4: Feature coverage, Boundary/Corner cases, Cross-feature interactions, Real-world scenarios.
- Tier 5: Adversarial coverage hardening.
- Publishes `TEST_READY.md`.

## Workflow & Strategy
- For each milestone: Explorer -> Worker -> Reviewer -> Challenger -> Auditor cycle.
- Audit gating: Forensic Auditor MUST return CLEAN before milestone is marked DONE.
- Continuous heartbeat and state recovery via `.agents/orchestrator/progress.md`.
