# Original User Request

## Initial Request — 2026-07-26T15:26:28Z

# Teamwork Project Prompt — Draft

> Status: Launched
> Goal: Craft prompt → get user approval → delegate to teamwork_preview

Transform the existing single-tenant Rust E-commerce backend into a multi-tenant, highly configurable SaaS platform (similar to Supabase/Firebase). The platform must feature strict tenant isolation at the database and event levels, scoped API keys for developers, tier-based usage metering, and robust webhook event streaming.

Working directory: c:\Users\USER\Documents\Previous\E-commerce\b2b-SaaS-application\rust-backend
Integrity mode: demo

## Requirements

### R1. Centralized Tenant & Auth Middleware
Implement scoped API keys and tier-based usage limits within a central middleware located in the API Gateway. This middleware must intercept incoming traffic, extract the tenant context (including tenant ID, tier limits, and feature flags), and inject it into the request scope for downstream services to inherit without handling auth themselves.

### R2. Hybrid Database Multi-Tenancy
Refactor the PostgreSQL schema to support a hybrid multi-tenancy model. Free/low-tier developers will use a shared database with Row-Level Security (RLS) enforcing tenant isolation. Enterprise tiers will route to dedicated database pools. All domain tables must be updated to include a `tenant_id` foreign key.

### R3. Tenant-Aware Event Mesh
Ensure the RabbitMQ / Redis Streams event mesh isolates payloads by tenant. All generated events must be enriched with the originating `tenant_id`, and consumer microservices must validate this tenant context before executing business logic to prevent cross-tenant data corruption.

## Acceptance Criteria

### API Gateway Authentication
- [ ] A test script can successfully hit the gateway with a valid scoped API key and receive a 200 OK with the injected tenant context.
- [ ] A test script hitting the gateway without an API key or with an invalid key is rejected with a 401 Unauthorized.
- [ ] Usage metering logic successfully rejects requests with a 402 Payment Required when a tenant exceeds their tier limit.

### Database Isolation
- [ ] A database integration test verifies that querying `orders` using RLS under Tenant A's context returns only Tenant A's data, completely hiding Tenant B's data.
- [ ] All table schemas successfully compile with `cargo sqlx prepare` reflecting the new `tenant_id` columns and indexes.

### Event Isolation
- [ ] An automated test proves that when `OrderCreatedEvent` is published, it contains the correct `tenant_id`.
- [ ] The consuming service rejects or ignores events if the `tenant_id` does not match the expected context.
