# Context Summary

## Project Overview
The project is a Rust E-commerce backend (`c:\Users\USER\Documents\Previous\E-commerce\b2b-SaaS-application\rust-backend`).
Goal: Transform it into a multi-tenant SaaS platform (similar to Supabase/Firebase).

## Core Requirements
1. **R1: Centralized Tenant & Auth Middleware** in API Gateway. Scoped API keys, usage limits, context injection, 401/402 handling.
2. **R2: Hybrid Database Multi-Tenancy**. Shared DB with Postgres RLS for low/free tiers, dedicated connection pools for enterprise tiers, `tenant_id` FK on domain tables, updated `cargo sqlx prepare`.
3. **R3: Tenant-Aware Event Mesh**. Enriched payloads with `tenant_id`, consumer validation of tenant context.

## Acceptance Criteria Summary
- Gateway Auth: Valid key -> 200 OK + tenant context; No/invalid key -> 401 Unauthorized; Limit exceeded -> 402 Payment Required.
- DB Isolation: RLS query on `orders` under Tenant A returns only Tenant A data; `cargo sqlx prepare` compiles cleanly.
- Event Isolation: `OrderCreatedEvent` contains `tenant_id`; consumers reject mismatched tenant events.
