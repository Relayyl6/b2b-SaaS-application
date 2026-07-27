# Handoff Report — Milestone R1: Centralized Tenant & Auth Middleware

**Explorer ID**: Explorer 1  
**Working Directory**: `c:\Users\USER\Documents\Previous\E-commerce\b2b-SaaS-application\rust-backend\.agents\teamwork_preview_explorer_r1_1`  
**Target Project Root**: `c:\Users\USER\Documents\Previous\E-commerce\b2b-SaaS-application\rust-backend`  

---

## 1. Observation

### 1.1 Project Structure & Workspace Setup
- **Root `Cargo.toml`**: Configures a Cargo workspace (`resolver = "2"`) containing 11 member crates:
  - Microservices: `analytics`, `inventory-management`, `logistics`, `notifications`, `order-service`, `payments`, `product-catalog`, `supplier-management`, `user-management`.
  - Shared Internal Crate: `platform` (`platform = { path = "platform" }`).
  - Test Harnesses: `e2e-tests`, `gateway-tests`.
- **Framework & Runtime**: All services use `actix-web = "4"`, `tokio = "1"`, `sqlx = "0.7"`, `redis = "0.25"`, `jsonwebtoken = "10.2.0"`.

### 1.2 Dual-Layer API Gateway Architecture
- **Layer 1 (Edge Proxy - Nginx)**: `infra/nginx/nginx.conf`
  - Runs on port `:80` / `:443`.
  - Routes traffic to individual microservices (`user-management:3004`, `product-catalog:3003`, `order-service:3005`, `inventory-management:3006`, `logistics:3008`, `notifications:3009`, `payments:3010`, `supplier-management:3011`, `analytics:3007`).
  - Implements Nginx subrequest authentication via `auth_request /_auth;` (lines 61–67, 76–90).
  - Proxy route `/_auth` forwards to `http://users_backend/auth/validate`.
  - **Critical Observation**: `infra/nginx/nginx.conf` does **NOT** propagate returned response headers (`X-User-Id`, `X-User-Role`) from `/_auth` to upstream services. It lacks `auth_request_set` and `proxy_set_header` directives inside location blocks.
- **Layer 2 (Service-Level Middleware)**: `user-management/src/middleware/authmiddleware.rs`
  - Defines Actix `Transform` / `Service` middleware (`AuthMiddleware`).
  - Parses `Authorization: Bearer <JWT_TOKEN>` header (lines 80–86).
  - Decodes token into `Claims` struct (`user-management/src/models.rs` lines 56–60):
    ```rust
    pub struct Claims {
        pub sub: Uuid,
        pub role: UserRole,
        pub exp: usize,
    }
    ```
  - Checks token revocation against Redis (`revoked_token:<token>`) or PostgreSQL (`revoked_tokens` table).
  - Fetches full `Users` object from DB and inserts into request extensions:
    `req.extensions_mut().insert(user);` (line 125).

### 1.3 Token Validation Endpoint (`validate_token`)
- Defined in `user-management/src/unprotected/handlers.rs` (lines 137–186):
  - Extracts JWT token from `Authorization` header.
  - Decodes `Claims`.
  - Checks Redis/DB revocation.
  - Returns `204 No Content` with headers:
    - `X-User-Id: <user_id>`
    - `X-User-Role: <role>`
  - Returns `401 Unauthorized` on missing/invalid token or revoked token.

### 1.4 Gaps Identified in Existing System
1. **No Tenant Context**: Neither `Claims` nor database tables (`users`, `orders`, `products`, `suppliers`) currently carry a `tenant_id`, subscription `tier`, or `feature_flags`.
2. **No API Key Support**: Authentication currently only supports Bearer JWT tokens. Scoped API keys (`sk_live_...` or `pk_live_...`) are not parsed or validated.
3. **No Usage Metering / Billing Limits**: There is no check for tenant request limits or return of `402 Payment Required`.
4. **Header Propagation Gap**: Nginx gateway does not pass `X-User-Id`, `X-User-Role`, or tenant context headers to downstream services.
5. **Service Gaps**: Downstream services (e.g., `order-service/src/routes.rs` lines 11–42) do not extract tenant or user context from request extensions or incoming HTTP headers.

---

## 2. Logic Chain

1. **Centralized Architectural Layer**:
   - Because `platform` is already imported by all workspace microservices (`platform = { path = "platform" }`), moving tenant authentication data structures, Actix-web middleware, and extractors into `platform` ensures zero code duplication across services.
   - For edge enforcement, Nginx must be configured to forward authentication headers, OR `user-management` must act as a dedicated Tenant & Auth Gateway service.

2. **Tenant Context Data Model**:
   - In a SaaS model, requests originate from either a user (Dashboard JWT) or a developer (API Key).
   - Both authentication methods must resolve to a unified `TenantContext`:
     - `tenant_id: Uuid`
     - `user_id: Option<Uuid>`
     - `tier: PricingTier` (Free, Growth, Enterprise)
     - `permissions: Vec<String>` (e.g., `["orders:create", "products:read"]`)
     - `feature_flags: HashMap<String, bool>`
     - `auth_method: AuthMethod` (Jwt vs ApiKey)

3. **API Key & JWT Auth Parsing**:
   - If header is `Authorization: Bearer sk_live_...` or `X-API-Key: sk_live_...`:
     - Extract raw key string, hash with SHA-256 (or Argon2).
     - Fast lookup in Redis cache (`api_key:<hash>`). Fallback to `api_keys` PostgreSQL table.
     - Look up associated tenant, active tier, and permissions.
   - If header is `Authorization: Bearer <jwt_token>`:
     - Decode JWT claims (updated to include `tenant_id` and `tier`).
     - Check token revocation in Redis/DB.
     - Build `TenantContext`.

4. **Tier-Based Usage Metering (402 Payment Required)**:
   - Atomic Redis counter per tenant: `usage:<tenant_id>:<YYYY-MM>`.
   - Tier limits:
     - `Free`: 100 requests/month
     - `Growth`: 10,000 requests/month
     - `Enterprise`: Unlimited (`u64::MAX`)
   - Pre-handler execution: `INCR` Redis key. If returned count exceeds tier limit:
     - Immediately reject request with `402 Payment Required` and structured JSON response.

5. **Context Propagation to Downstream Services**:
   - **At Nginx Gateway**: `auth_request_set` directives copy `X-Tenant-Id`, `X-Tenant-Tier`, `X-User-Id`, and `X-Permissions` from `/auth/validate` to proxy headers.
   - **At Microservice Level**: `TenantAuthMiddleware` in `platform` extracts incoming `X-Tenant-*` headers or validates tokens directly, inserting `TenantContext` into Actix `req.extensions_mut()`.
   - **Extractor Pattern**: Provide `web::ReqData<TenantContext>` or an Actix `FromRequest` impl (`TenantContextExtractor`) so handlers can declare `tenant: TenantContext` as a handler parameter.

---

## 3. Caveats

- **Nginx Local Environment**: In non-Docker development environments running individual `cargo run` services directly without Nginx, microservices need fallback internal Actix middleware to validate tokens/headers.
- **Database Migrations Required**: Database schema updates (adding `tenants`, `api_keys`, and `tenant_id` foreign keys) are owned by data migration scripts; middleware must support default test tenants during transition.
- **Redis Availability**: If Redis connection fails, the middleware should fail open/closed gracefully based on fallback database queries.

---

## 4. Conclusion & Implementation Plan

### 4.1 Data Structures to Create

#### Path: `platform/src/tenant.rs`
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PricingTier {
    Free,
    Growth,
    Enterprise,
}

impl PricingTier {
    pub fn monthly_limit(&self) -> u64 {
        match self {
            PricingTier::Free => 100,
            PricingTier::Growth => 10_000,
            PricingTier::Enterprise => u64::MAX,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthMethod {
    Jwt,
    ApiKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub tier: PricingTier,
    pub permissions: Vec<String>,
    pub feature_flags: HashMap<String, bool>,
    pub auth_method: AuthMethod,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKeyRecord {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub key_prefix: String,
    pub key_hash: String,
    pub permissions: Vec<String>,
    pub rate_limit_override: Option<u64>,
    pub is_active: bool,
}
```

### 4.2 Middleware Pattern & Implementation Steps

#### Step 1: Update `platform/src/lib.rs`
Export `tenant` module and Actix middleware utilities:
```rust
pub mod metrics;
pub mod observability;
pub mod streams;
pub mod tenant;
pub mod middleware;
```

#### Step 2: Implement Shared Actix Middleware (`platform/src/middleware/tenant_middleware.rs`)
- Parses `Authorization: Bearer ...` or `X-API-Key: ...` or incoming `X-Tenant-Id` headers.
- Performs Redis usage metering count (`usage:<tenant_id>:<period>`).
- If limit exceeded, returns `402 Payment Required`:
  ```json
  {
    "error": "Payment Required",
    "message": "Usage limit exceeded for current pricing tier",
    "tier": "Free",
    "limit": 100,
    "current_usage": 101
  }
  ```
- If missing/invalid key/token, returns `401 Unauthorized`.
- On success, injects `TenantContext` into `req.extensions_mut()`.

#### Step 3: Update `user-management` Auth & Validation Handler
- Update `user-management/src/models.rs`: Add `tenant_id` to `Claims`.
- Update `validate_token` in `user-management/src/unprotected/handlers.rs`:
  - Support both JWT tokens and API key validation.
  - Return headers:
    - `X-Tenant-Id: <tenant_id>`
    - `X-Tenant-Tier: <tier>`
    - `X-User-Id: <user_id>`
    - `X-Tenant-Permissions: <json_or_csv>`

#### Step 4: Refactor `infra/nginx/nginx.conf` Header Forwarding
Update Nginx location blocks to capture subrequest headers and set proxy headers for downstream services:
```nginx
location /orders {
    auth_request /_auth;
    auth_request_set $tenant_id $upstream_http_x_tenant_id;
    auth_request_set $tenant_tier $upstream_http_x_tenant_tier;
    auth_request_set $user_id $upstream_http_x_user_id;

    proxy_set_header X-Tenant-Id $tenant_id;
    proxy_set_header X-Tenant-Tier $tenant_tier;
    proxy_set_header X-User-Id $user_id;
    proxy_pass http://orders_backend;
}
```

---

## 5. Verification Method

1. **Unit & Middleware Tests**:
   - Run workspace unit tests: `cargo test --workspace`
   - Specific platform middleware test: `cargo test -p platform`
2. **Gateway Security Tests**:
   - Run integration tests in `gateway-tests`: `cargo test -p gateway-tests`
3. **cURL Invalidation Verification**:
   - Missing Key / Token -> `401 Unauthorized`:
     `curl -i https://localhost/orders`
   - Valid API Key / JWT -> `200 OK` / `201 Created`:
     `curl -i https://localhost/orders -H "Authorization: Bearer sk_live_test123"`
   - Exceeded Usage Limit -> `402 Payment Required`:
     Fire 101 requests for a Free tier tenant and verify `HTTP/1.1 402 Payment Required`.
