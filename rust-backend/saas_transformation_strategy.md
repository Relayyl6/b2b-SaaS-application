# Commerce-as-a-Service (CaaS) Platform — Master Transformation Strategy

> **Standard:** This document is written to the same standard as Supabase, Firebase, and Stream platform documentation.
> Every section contains real Rust code, real SQL, real environment variables, and real API contracts —
> not placeholder text. Every decision maps back to a direct use-case from one of those three platforms.

---

## Table of Contents

1. [The Platform Vision — Why This Exists](#1-the-platform-vision)
2. [Use-Case Analysis: Stream, Supabase & Firebase as Moats](#2-use-case-analysis)
3. [Platform Configuration: All Keys, All Secrets, All Variables](#3-platform-configuration)
4. [Multi-Tenancy Architecture (Hybrid Isolation Model)](#4-multi-tenancy-architecture)
5. [API Key System — Generation, Scoping, Storage & Validation](#5-api-key-system)
6. [TenantContext — The Request Analysis Core](#6-tenantcontext--request-analysis)
7. [Critical Information in API Responses](#7-critical-information-in-api-responses)
8. [Event Mesh Isolation (Tenant-Safe Redis Streams)](#8-event-mesh-isolation)
9. [Webhook System — HMAC Signing & DLQ Retries](#9-webhook-system)
10. [Usage Metering & Billing Engine](#10-usage-metering--billing-engine)
11. [Complete Database Schema (Multi-Tenant)](#11-complete-database-schema)
12. [Dashboard Configuration APIs (Like Supabase Dashboard)](#12-dashboard-configuration-apis)
23. [Tenant Onboarding Flow](#23-tenant-onboarding-flow)
24. [Developer Experience (DX) & Client SDKs](#24-developer-experience-dx--client-sdks)
25. [Integrations Model: Native Defaults & BYOP](#25-integrations-model-native-defaults--byop)
26. [Headless Checkout Orchestration](#26-headless-checkout-orchestration)
27. [Platform Dashboard UI Mapping](#27-platform-dashboard-ui-mapping)

---

## 1. The Platform Vision

You are building **Commerce-as-a-Service (CaaS)** — the commerce infrastructure primitive that other companies
build their product on top of. The three moats:

| Moat | What it Means | Reference Platform |
|------|---------------|-------------------|
| **API Key Configurability** | Every tenant gets scoped, rotatable keys with live/test environments | Stripe, Stream |
| **Critical Request Analysis** | Every HTTP request is analyzed: latency, tenant, scope, payload size, logged to TimescaleDB | Supabase, Datadog |
| **Extreme Data Extensibility** | Every entity (Order, Product, User) carries a `metadata: JSONB` field tenants can write anything into | Firebase Firestore, Supabase |
| **Real-time Event Streaming** | Tenants subscribe to `order.created`, `payment.failed` etc. via webhooks with HMAC verification | Stream Platform |
| **Hybrid Multi-Tenancy** | Free/Pro → shared Postgres with RLS. Enterprise → dedicated DB pool, zero data commingling | Supabase Pro/Team |

---

## 2. Use-Case Analysis

### 2.1 Stream Platform — The Dashboard & API Key Model

Stream (getstream.io) is the reference for what our configurability should look like.

**What Stream gives every developer when they create an app:**
```
App Name:       my-commerce-app
App ID:         1234567
Region:         US East
API Key:        abc123defghi         ← Public. Safe to put in frontend code.
API Secret:     xxxxxxxxxxxxxxxx     ← Server-side only. Never expose.
Webhook Secret: whsec_EXAMPLE_SECRET_REDACTED      ← Used to verify webhook payloads.
```

**Stream's Critical Dashboard Features (we must match all of these):**
- **Message Stats:** API calls/day, MAUs, active connections
- **Moderation Config:** Per-app configurable block lists, profanity filters
- **Push Notification Config:** Per-app APNs/FCM credentials stored encrypted
- **Webhook Config:** Per-app URL, enabled events, delivery logs, replay button
- **Rate Limits:** Per-app overrides to rate limits
- **Channel Types:** Fully customizable per-app: permissions, reactions, read receipts

**Our Equivalent:**
```
Tenant Name:       acme-commerce
Tenant ID:         ten_01H8X2QMNP7KZDX92F3CJVBKM2
Region:            us-east-1
Public Key:        pk_test_EXAMPLE_KEY_REDACTED    ← Frontend SDK. Read-only commerce data.
Secret Key:        sk_live_EXAMPLE_KEY_REDACTED    ← Server-side. Full write access.
Webhook Secret:    whsec_EXAMPLE_SECRET_REDACTED      ← HMAC verification of all webhook payloads.
```

**Configuration Options (like Stream's app settings):**
- Orders per month limit (tier-based, overridable for Enterprise)
- Allowed payment providers per tenant (`stripe`, `paystack`, `flutterwave`)
- Inventory reservation TTL (seconds before an uncompleted reservation expires)
- Webhook retry policy (max attempts, backoff strategy)
- Notification channels enabled per tenant (Email, SMS, Push)
- Custom metadata schema validation (optional JSON Schema per entity type)
- IP allowlist for API key usage

---

### 2.2 Supabase — The Project Config & RLS Model

**Supabase gives every project:**
```
Project URL:        https://xyzabc.supabase.co
Anon Key:           eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...  ← For frontend, limited permissions
Service Role Key:   eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...  ← Admin, bypasses RLS
JWT Secret:         your-super-secret-jwt-token-here
DB Connection:      postgresql://postgres:password@db.xyzabc.supabase.co:5432/postgres
```

**Our Equivalent (for our tenant's "connection info" panel in their dashboard):**
```
API Base URL:   https://api.commerceplatform.io/v1
Tenant ID:      ten_01H8X2QMNP7KZDX92F3CJVBKM2
Public Key:     pk_live_EXAMPLE_KEY_REDACTED  ← Safe for client-side SDKs
Secret Key:     sk_live_EXAMPLE_KEY_REDACTED  ← Never expose publicly
Webhook Secret: whsec_EXAMPLE_SECRET_REDACTED

# For Enterprise tenants only:
Direct DB URL:  postgresql://tenant_acme:pass@pg-acme.commerceplatform.io:5432/acme_commerce
```

**Supabase RLS Pattern — Our Implementation:**
In Supabase, Row-Level Security policies like `USING (auth.uid() = user_id)` ensure data isolation.
Our equivalent uses `SET LOCAL app.current_tenant_id` which our RLS policies check:

```sql
-- Applied automatically by TenantContext::apply_rls()
SET LOCAL app.current_tenant_id = 'ten_01H8X2QMNP7KZDX92F3CJVBKM2';

-- RLS policy on orders table
CREATE POLICY tenant_isolation_policy ON orders
    USING (tenant_id = current_setting('app.current_tenant_id')::uuid);
```

---

### 2.3 Firebase — The Per-App Config & Realtime Model

**Firebase gives every app:**
```javascript
// Firebase SDK config (safe to expose in frontend)
const firebaseConfig = {
  apiKey: "AIzaSyD...",
  authDomain: "myapp.firebaseapp.com",
  projectId: "myapp",
  storageBucket: "myapp.appspot.com",
  messagingSenderId: "123456789",
  appId: "1:123456789:web:abc123def456"
};
```

**Firebase's Configurability Model (what we adopt):**
- **Security Rules per resource:** Firestore rules, Storage rules, Realtime DB rules — all configurable from dashboard
- **Remote Config:** Key-value config pushed to apps without a deploy
- **Feature Flags:** Per-user/per-device feature flag targeting
- **Analytics:** User retention, event funnels, cohort analysis

**Our Commerce Equivalent:**
```json
// Returned when a tenant initializes our SDK (like Firebase initializeApp)
{
  "project": {
    "tenant_id": "ten_01H8X2QMNP7KZDX92F3CJVBKM2",
    "name": "Acme Corp",
    "environment": "live",
    "region": "us-east-1"
  },
  "keys": {
    "public_key": "pk_live_...",
    "webhook_secret_hint": "whsec_****U2V3"
  },
  "config": {
    "order_mode": "managed",
    "inventory_reservation_ttl_seconds": 900,
    "payment_providers": ["stripe"],
    "notification_channels": ["email", "sms", "push"],
    "metadata_schema": null,
    "rate_limits": {
      "requests_per_second": 100,
      "burst": 200
    }
  },
  "features": {
    "multi_vendor": true,
    "real_time_inventory": true,
    "vendor_analytics": false,
    "white_label": false
  }
}
```

---

## 3. Platform Configuration

### 3.1 The Complete `.env` Reference

This is the **definitive** environment variable reference for every service in this platform.
Every variable listed here maps to a real configuration point used by the Rust services.

```bash
# =============================================================================
# DATABASE — Core & Tenant Routing
# =============================================================================

# Shared PostgreSQL pool (Free/Pro tier tenants)
DATABASE_URL=postgres://commerce_user:secret@localhost:5432/commerce_shared

# Control plane database (tenants, API keys, billing — separate from commerce data)
CONTROL_PLANE_DATABASE_URL=postgres://control_user:secret@localhost:5432/commerce_control

# TimescaleDB for analytics, API request logs, and usage metering
ANALYTICS_DATABASE_URL=postgres://analytics_user:secret@localhost:5432/commerce_analytics

# =============================================================================
# REDIS — Streams, Cache, Rate Limiting
# =============================================================================

REDIS_URL=redis://127.0.0.1:6379/
# For Redis Cluster (production):
# REDIS_CLUSTER_URLS=redis://node1:6379,redis://node2:6379,redis://node3:6379

# =============================================================================
# MESSAGE BROKER — RabbitMQ for Analytics fanout
# =============================================================================

AMQP_ADDR=amqp://guest:guest@127.0.0.1:5672/%2f
RABBITMQ_URL=amqp://guest:guest@127.0.0.1:5672/%2f

# =============================================================================
# AUTH & PLATFORM SECURITY
# =============================================================================

# JWT signing secret (for user session tokens in user-management service)
SECRET=my_super_secret_jwt_key_change_in_production_min_32_chars

# Master encryption key for encrypting stored API key hashes (AES-256-GCM)
PLATFORM_MASTER_KEY=base64_encoded_32_byte_key_here

# Internal service-to-service auth token (microservices call each other with this)
INTERNAL_SERVICE_TOKEN=internal_svc_token_change_me

# =============================================================================
# PAYMENTS — Stripe
# =============================================================================

STRIPE_SECRET_KEY=sk_test_51...
STRIPE_WEBHOOK_SECRET=whsec_...
# Stripe Connect (for vendor payment splitting)
STRIPE_CONNECT_CLIENT_ID=ca_...

# =============================================================================
# PAYMENTS — Alternative Providers (tenant-level config, not platform-level)
# =============================================================================
# These are tenant-configurable via Dashboard, not platform env vars.
# The platform stores them encrypted in the control_plane DB.
# PAYSTACK_SECRET_KEY  → stored encrypted in tenants.payment_config JSONB
# FLUTTERWAVE_API_KEY  → stored encrypted in tenants.payment_config JSONB

# =============================================================================
# STORAGE — Product Media & Assets
# =============================================================================

CLOUDINARY_CLOUD_NAME=my_cloud_name
CLOUDINARY_API_KEY=my_api_key
CLOUDINARY_API_SECRET=my_api_secret

# =============================================================================
# NOTIFICATIONS — Email
# =============================================================================

SENDGRID_API_KEY=SG.your_sendgrid_api_key_here
SENDGRID_FROM_EMAIL=no-reply@yourdomain.com

# Fallback webhook for email (non-SendGrid)
EMAIL_WEBHOOK_URL=http://localhost:8080/email/webhook

# =============================================================================
# NOTIFICATIONS — SMS
# =============================================================================

TWILIO_ACCOUNT_SID=ACyour_twilio_account_sid_here
TWILIO_AUTH_TOKEN=your_twilio_auth_token_here
TWILIO_FROM_NUMBER=+1234567890

# Fallback webhook for SMS (non-Twilio)
SMS_WEBHOOK_URL=http://localhost:8080/sms/webhook

# =============================================================================
# NOTIFICATIONS — Push (Expo)
# =============================================================================

EXPO_PUSH_URL=https://exp.host/--/api/v2/push/send
EXPO_ACCESS_TOKEN=your_expo_access_token_here

# =============================================================================
# NOTIFICATIONS — Dry Run (disables all outbound notification delivery)
# =============================================================================

NOTIFICATION_DRY_RUN=true   # Set to false in production

# =============================================================================
# OBSERVABILITY — OpenTelemetry & Prometheus
# =============================================================================

RUST_LOG=info,platform=debug,order_service=debug
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
OTEL_SERVICE_NAME=commerce-platform
JAEGER_ENDPOINT=http://localhost:14268/api/traces

# =============================================================================
# PLATFORM FEATURES (Feature Flags — platform-wide defaults)
# =============================================================================

# Whether the platform-level webhook dispatcher is active
WEBHOOK_DISPATCHER_ENABLED=true

# Default inventory reservation TTL (seconds)
INVENTORY_RESERVATION_TTL_SECONDS=900

# Maximum metadata JSONB size per entity (bytes)
MAX_METADATA_SIZE_BYTES=65536

# =============================================================================
# SERVICE PORTS
# =============================================================================

TENANT_MANAGEMENT_PORT=3000
USER_MANAGEMENT_PORT=3001
PRODUCT_CATALOG_PORT=3003
SUPPLIER_MANAGEMENT_PORT=3004
ORDER_SERVICE_PORT=3005
INVENTORY_MANAGEMENT_PORT=3006
ANALYTICS_PORT=3007
LOGISTICS_PORT=3008
PAYMENTS_PORT=3009
NOTIFICATIONS_PORT=3010

# =============================================================================
# SERVICE-TO-SERVICE URLS
# =============================================================================

SUPPLIER_MANAGEMENT_URL=http://localhost:3004
TENANT_MANAGEMENT_URL=http://localhost:3000
ANALYTICS_URL=http://localhost:3007

# =============================================================================
# RATE LIMITING (Redis-backed per tenant)
# =============================================================================

# Default rate limit for free tier (requests per second)
RATE_LIMIT_FREE_RPS=10
# Default rate limit for growth tier
RATE_LIMIT_GROWTH_RPS=100
# Default rate limit for enterprise tier
RATE_LIMIT_ENTERPRISE_RPS=1000

# =============================================================================
# BILLING — Stripe Billing (platform charges tenants)
# =============================================================================

STRIPE_BILLING_SECRET_KEY=sk_live_...
STRIPE_BILLING_WEBHOOK_SECRET=whsec_...
STRIPE_FREE_PRICE_ID=price_free
STRIPE_GROWTH_PRICE_ID=price_growth_29
STRIPE_SCALE_PRICE_ID=price_scale_199

# =============================================================================
# WEBHOOK DISPATCH ENGINE
# =============================================================================

# Maximum webhook delivery attempts before moving to DLQ
WEBHOOK_MAX_ATTEMPTS=5
# Base backoff delay in milliseconds (doubles each retry)
WEBHOOK_BACKOFF_BASE_MS=1000
# Maximum backoff delay in milliseconds
WEBHOOK_BACKOFF_MAX_MS=300000
# Timeout for each webhook delivery attempt (milliseconds)
WEBHOOK_DELIVERY_TIMEOUT_MS=5000
```

---

## 4. Multi-Tenancy Architecture

### 4.1 The Hybrid Model (Option C)

```
Free/Pro Tenants (< $500/mo)          Enterprise Tenants (> $500/mo)
┌─────────────────────────────┐       ┌──────────────────────────────────┐
│  Shared PostgreSQL Instance  │       │  Dedicated PostgreSQL Instance    │
│  + Row-Level Security        │       │  (per tenant, physically isolated)│
│  tenant_id on every table    │       │  No RLS needed — schema isolation │
│  RLS via SET LOCAL           │       │  Routed by DynamicPoolRouter      │
└─────────────────────────────┘       └──────────────────────────────────┘
            │                                           │
            └───────────────────┬───────────────────────┘
                                │
                   ┌────────────▼────────────┐
                   │   DynamicPoolRouter      │
                   │  (platform/src/          │
                   │   db_router.rs)          │
                   └─────────────────────────┘
```

### 4.2 DynamicPoolRouter — How it Works

The `DynamicPoolRouter` (already in `platform/src/db_router.rs`) routes each request to the correct pool:

```rust
// platform/src/db_router.rs (already implemented — annotated for clarity)
impl DynamicPoolRouter {
    pub async fn get_pool(&self, ctx: &TenantContext) -> Result<PgPool, sqlx::Error> {
        match ctx.tier {
            // Enterprise: check for a dedicated pool URL in the TenantContext.
            // If db_connection_url is set (comes from control-plane DB), open/cache a new pool.
            PricingTier::Enterprise => {
                if let Some(ref db_url) = ctx.db_connection_url {
                    // Check cache first (O(1) read lock)
                    {
                        let pools = self.dedicated_pools.read().await;
                        if let Some(pool) = pools.get(&ctx.tenant_id) {
                            return Ok(pool.clone()); // Cache hit
                        }
                    }
                    // Cache miss: open new pool and cache it
                    let mut pools = self.dedicated_pools.write().await;
                    let new_pool = PgPool::connect(db_url).await?;
                    pools.insert(ctx.tenant_id, new_pool.clone());
                    Ok(new_pool)
                } else {
                    // Enterprise without dedicated DB URL → fallback to shared
                    Ok(self.shared_pool.clone())
                }
            }
            // Free & Growth → always use shared pool with RLS
            PricingTier::Free | PricingTier::Growth => Ok(self.shared_pool.clone()),
        }
    }
}
```

### 4.3 PostgreSQL Row-Level Security Setup

```sql
-- Step 1: Create the GUC variable for tenant context propagation
ALTER DATABASE commerce_shared SET app.current_tenant_id = '';

-- Step 2: Enable RLS on every table
ALTER TABLE orders ENABLE ROW LEVEL SECURITY;
ALTER TABLE products ENABLE ROW LEVEL SECURITY;
ALTER TABLE inventory ENABLE ROW LEVEL SECURITY;
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE payments ENABLE ROW LEVEL SECURITY;
ALTER TABLE logistics_shipments ENABLE ROW LEVEL SECURITY;
ALTER TABLE notifications ENABLE ROW LEVEL SECURITY;
ALTER TABLE suppliers ENABLE ROW LEVEL SECURITY;
ALTER TABLE analytics_events ENABLE ROW LEVEL SECURITY;

-- Step 3: Create policies that read the GUC variable
CREATE POLICY tenant_isolation ON orders
    FOR ALL
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY tenant_isolation ON products
    FOR ALL
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

-- (repeat for all tables)

-- Step 4: Create a role that cannot bypass RLS (used by services)
CREATE ROLE commerce_app_role NOINHERIT;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO commerce_app_role;
-- NOTE: SUPERUSER and table owner can bypass RLS. commerce_app_role cannot.
```

```rust
// In EVERY Actix handler that touches the DB, apply RLS first:
pub async fn create_order_handler(
    tenant: ReqData<TenantContext>,
    db_router: Data<DynamicPoolRouter>,
    req: Json<CreateOrderRequest>,
) -> impl Responder {
    let pool = db_router.get_pool(&tenant).await.unwrap();
    let mut tx = pool.begin().await.unwrap();

    // THIS IS MANDATORY — sets the tenant context for the transaction
    tenant.apply_rls(&mut *tx).await.unwrap();
    // Now all queries within this transaction are automatically filtered by tenant_id

    let order = Order::create(&mut *tx, &tenant.tenant_id, req.into_inner())
        .await
        .unwrap();

    tx.commit().await.unwrap();
    HttpResponse::Created().json(order)
}
```

---

## 5. API Key System

### 5.1 Key Anatomy (Stream/Stripe Style)

Every tenant gets keys in two environments: `test` and `live`.

```
Format:   {type}_{env}_{26-char-base58-random}

Examples:
  sk_test_EXAMPLE_KEY_REDACTED   ← Secret key, test environment
  sk_live_EXAMPLE_KEY_REDACTED   ← Secret key, live environment
  pk_test_EXAMPLE_KEY_REDACTED   ← Public key, test environment
  pk_live_EXAMPLE_KEY_REDACTED   ← Public key, live environment
  whsec_EXAMPLE_SECRET_REDACTED   ← Webhook signing secret
```

**Key Types:**

| Prefix | Use | Permissions |
|--------|-----|-------------|
| `sk_live_` | Server-side, production | Full read+write. Never expose publicly. |
| `sk_test_` | Server-side, testing | Full read+write against test data. |
| `pk_live_` | Client-side SDK, production | Read-only public catalog/inventory |
| `pk_test_` | Client-side SDK, testing | Read-only public catalog/inventory (test data) |
| `whsec_` | Webhook verification | Used to verify HMAC signatures. Not an auth token. |
| `rk_` | Restricted key | Custom scopes. E.g. `rk_orders:read_abc123` |

### 5.2 Key Generation (Rust Implementation)

```rust
// In tenant-management service
use rand::Rng;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};

pub struct ApiKey {
    /// The full plaintext key — ONLY shown ONCE to the developer, never stored
    pub plaintext: String,
    /// The prefix (first 10 chars) — stored in DB for identification
    pub prefix: String,
    /// SHA-256 hash of the full key — stored in DB for validation
    pub hash: String,
}

pub fn generate_api_key(key_type: &str, environment: &str) -> ApiKey {
    // Generate 32 cryptographically random bytes
    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..32).map(|_| rng.gen::<u8>()).collect();

    // Encode as base58 to avoid ambiguous characters (no 0, O, l, I)
    let random_part = bs58::encode(&random_bytes).into_string();

    // Assemble the full key
    let plaintext = format!("{}_{}_{}",
        key_type,       // "sk", "pk", "rk"
        environment,    // "live", "test"
        &random_part[..26]
    );

    // Compute the hash (this is what gets stored in the DB)
    let mut hasher = Sha256::new();
    hasher.update(plaintext.as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    // The prefix is used to look up which hash to compare against
    let prefix = plaintext[..10].to_string(); // e.g. "sk_live_7c"

    ApiKey { plaintext, prefix, hash }
}
```

### 5.3 Key Storage Schema

```sql
-- Control plane database (NOT the shared commerce DB)
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    tier VARCHAR(50) NOT NULL DEFAULT 'free',          -- 'free' | 'growth' | 'enterprise'
    stripe_customer_id VARCHAR(255),                    -- For billing
    db_connection_url TEXT,                             -- Enterprise: dedicated DB URL (encrypted)
    payment_config JSONB DEFAULT '{}',                  -- Encrypted payment provider keys
    notification_config JSONB DEFAULT '{}',             -- Per-tenant notification settings
    feature_flags JSONB DEFAULT '{}',                   -- Per-tenant feature overrides
    webhook_config JSONB DEFAULT '{}',                  -- Webhook URLs and event subscriptions
    rate_limit_override INTEGER,                        -- NULL = use tier default
    metadata_schema JSONB DEFAULT NULL,                 -- Optional JSON Schema for metadata validation
    ip_allowlist TEXT[],                                -- NULL = allow all IPs
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,                         -- Human-readable label: "Production Server Key"
    key_prefix VARCHAR(16) NOT NULL UNIQUE,             -- First 10 chars: "sk_live_7c"
    key_hash VARCHAR(64) NOT NULL UNIQUE,               -- SHA-256 of full key
    key_type VARCHAR(8) NOT NULL,                       -- 'sk' | 'pk' | 'rk'
    environment VARCHAR(8) NOT NULL,                    -- 'live' | 'test'
    scopes TEXT[] NOT NULL DEFAULT '{}',               -- ['orders:read', 'orders:write', ...]
    rate_limit_override INTEGER,                        -- NULL = use tenant default
    last_used_at TIMESTAMPTZ,
    last_used_ip INET,
    usage_count BIGINT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    expires_at TIMESTAMPTZ,                             -- NULL = never expires
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID                                     -- User who created this key (for audit)
);

-- Fast lookup index: when a request comes in with a key prefix, we find the hash instantly
CREATE INDEX idx_api_keys_prefix ON api_keys(key_prefix) WHERE is_active = TRUE;
CREATE INDEX idx_api_keys_tenant ON api_keys(tenant_id) WHERE is_active = TRUE;
```

### 5.4 Key Validation — The Gateway Hot Path

```rust
// This runs on EVERY request. It must be fast.
// Strategy: prefix → Redis cache → DB fallback

pub async fn validate_api_key(
    raw_key: &str,
    redis: &Pool,             // deadpool-redis
    db: &PgPool,              // control-plane DB
) -> Result<TenantContext, AuthError> {

    // 1. Extract prefix (first 10 chars after "Bearer ")
    let key = raw_key.strip_prefix("Bearer ").unwrap_or(raw_key);
    if key.len() < 26 {
        return Err(AuthError::InvalidKey);
    }
    let prefix = &key[..10]; // e.g. "sk_live_7c"

    // 2. Hash the incoming key
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let incoming_hash = format!("{:x}", hasher.finalize());

    // 3. Check Redis cache first (TTL: 5 minutes)
    let cache_key = format!("apikey:prefix:{}", prefix);
    let mut conn = redis.get().await?;
    if let Ok(cached_json) = redis::cmd("GET")
        .arg(&cache_key)
        .query_async::<_, String>(&mut *conn)
        .await
    {
        let cached: CachedKeyRecord = serde_json::from_str(&cached_json)?;
        // Constant-time comparison to prevent timing attacks
        if !constant_time_compare(&incoming_hash, &cached.key_hash) {
            return Err(AuthError::InvalidKey);
        }
        return Ok(build_tenant_context(cached));
    }

    // 4. Cache miss — query the control-plane database
    let record = sqlx::query_as::<_, ApiKeyRecord>(
        r#"
        SELECT k.id, k.key_hash, k.key_type, k.environment, k.scopes,
               k.rate_limit_override, t.id as tenant_id, t.tier,
               t.feature_flags, t.db_connection_url
        FROM api_keys k
        JOIN tenants t ON t.id = k.tenant_id
        WHERE k.key_prefix = $1 AND k.is_active = TRUE
          AND (k.expires_at IS NULL OR k.expires_at > NOW())
        "#
    )
    .bind(prefix)
    .fetch_optional(db)
    .await?
    .ok_or(AuthError::KeyNotFound)?;

    // 5. Constant-time hash comparison
    if !constant_time_compare(&incoming_hash, &record.key_hash) {
        return Err(AuthError::InvalidKey);
    }

    // 6. Cache the result (5 minute TTL)
    let to_cache = serde_json::to_string(&CachedKeyRecord::from(&record))?;
    let _: () = redis::cmd("SETEX")
        .arg(&cache_key)
        .arg(300) // 5 minutes
        .arg(to_cache)
        .query_async(&mut *conn)
        .await?;

    // 7. Fire-and-forget: update last_used_at (non-blocking)
    let db_clone = db.clone();
    let key_id = record.id;
    tokio::spawn(async move {
        let _ = sqlx::query(
            "UPDATE api_keys SET last_used_at = NOW(), usage_count = usage_count + 1 WHERE id = $1"
        )
        .bind(key_id)
        .execute(&db_clone)
        .await;
    });

    Ok(build_tenant_context_from_record(record))
}

// Constant-time comparison prevents timing attacks
fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() { return false; }
    a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}
```

---

## 6. TenantContext — Request Analysis

### 6.1 The Full TenantContext Struct

The `TenantContext` (in `platform/src/tenant.rs`) is the single most important object in the platform.
Every Actix handler receives it. It encodes **everything** the platform needs to make decisions.

```rust
// platform/src/tenant.rs — EXTENDED from current implementation
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    // Identity
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,             // The end-user making the request (if JWT path)
    pub api_key_id: Option<Uuid>,          // The API key used (if API key path)

    // Environment
    pub environment: Environment,           // Test | Live

    // Authorization
    pub tier: PricingTier,                 // Free | Growth | Enterprise
    pub permissions: Vec<String>,          // ["orders:read", "orders:write", "products:read"]
    pub auth_method: AuthMethod,           // Jwt | ApiKey

    // Configurability
    pub feature_flags: HashMap<String, bool>,  // Per-tenant feature overrides
    pub rate_limit: u32,                       // requests/second for this key

    // Routing
    pub db_connection_url: Option<String>, // Enterprise: dedicated DB URL

    // Request Analysis (populated by middleware, NOT from DB)
    #[serde(skip)]
    pub request_start: Option<Instant>,    // For latency tracking
    pub request_id: String,               // UUID per request for tracing
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment { Test, Live }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PricingTier {
    Free,       // 100 orders/month
    Growth,     // 10,000 orders/month
    Enterprise, // Unlimited
}

impl PricingTier {
    pub fn monthly_order_limit(&self) -> u64 {
        match self {
            Self::Free => 100,
            Self::Growth => 10_000,
            Self::Enterprise => u64::MAX,
        }
    }

    pub fn rate_limit_rps(&self) -> u32 {
        match self {
            Self::Free => 10,
            Self::Growth => 100,
            Self::Enterprise => 1000,
        }
    }

    pub fn max_webhook_endpoints(&self) -> usize {
        match self {
            Self::Free => 2,
            Self::Growth => 10,
            Self::Enterprise => 100,
        }
    }
}
```

### 6.2 The TenantMiddleware — Critical Request Analysis

This middleware runs on EVERY request and does the following:
1. Validates the API key (or JWT)
2. Builds the `TenantContext`
3. Checks rate limits
4. Injects the context into the request
5. Records request metadata for the analytics pipeline

```rust
// In the API gateway or in each service's Actix app
use actix_web::dev::{ServiceRequest, ServiceResponse, Transform, Service};
use actix_web::Error;
use futures::future::{ok, Ready};

pub struct TenantMiddleware {
    pub redis: Pool,
    pub control_db: PgPool,
    pub analytics_publisher: StreamPublisher, // publishes to analytics service
}

impl<S, B> Transform<S, ServiceRequest> for TenantMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = TenantMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(TenantMiddlewareService {
            service: Rc::new(service),
            redis: self.redis.clone(),
            control_db: self.control_db.clone(),
            analytics_publisher: self.analytics_publisher.clone(),
        })
    }
}

impl<S, B> Service<ServiceRequest> for TenantMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let redis = self.redis.clone();
        let control_db = self.control_db.clone();
        let analytics_publisher = self.analytics_publisher.clone();
        let service = self.service.clone();
        let request_start = Instant::now();
        let request_id = Uuid::new_v4().to_string();
        let method = req.method().to_string();
        let path = req.path().to_string();
        let ip = req.connection_info().realip_remote_addr()
                    .unwrap_or("unknown").to_string();

        Box::pin(async move {
            // 1. Extract Authorization header
            let auth_header = req.headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("")
                .to_string();

            // 2. Validate key → build TenantContext
            let mut ctx = match validate_api_key(&auth_header, &redis, &control_db).await {
                Ok(ctx) => ctx,
                Err(e) => {
                    return Ok(req.error_response(
                        actix_web::error::ErrorUnauthorized(
                            serde_json::json!({
                                "error": "unauthorized",
                                "message": e.to_string(),
                                "request_id": request_id
                            }).to_string()
                        )
                    ).map_into_right_body())
                }
            };

            // 3. Populate request analysis fields
            ctx.request_start = Some(request_start);
            ctx.request_id = request_id.clone();

            // 4. Enforce rate limits (Redis sliding window)
            let rate_key = format!("ratelimit:{}:{}", ctx.tenant_id, 
                                   request_start.elapsed().as_secs() / 1);
            // ... (sliding window counter logic)

            // 5. Inject TenantContext into request extensions
            req.extensions_mut().insert(ctx.clone());

            // 6. Call the downstream handler
            let response = service.call(req).await?;
            let latency_ms = request_start.elapsed().as_millis() as u64;
            let status = response.status().as_u16();

            // 7. Fire-and-forget: publish request analysis event to analytics pipeline
            analytics_publisher.publish_async("api.request.completed", ApiRequestEvent {
                tenant_id: ctx.tenant_id,
                request_id,
                method,
                path,
                status_code: status,
                latency_ms,
                ip,
                environment: ctx.environment,
                tier: ctx.tier,
                timestamp: chrono::Utc::now(),
            });

            Ok(response)
        })
    }
}
```

---

## 7. Critical Information in API Responses

### 7.1 Standard Response Envelope

Every API response is wrapped in a standard envelope that includes critical metadata.
This is the same pattern used by Stripe's API responses.

```json
// POST /v1/orders — Success Response
{
  "data": {
    "id": "ord_01H8X2QMNP7KZDX92F3CJVBKM2",
    "tenant_id": "ten_01H8X2QMNP7KZDX92F3CJVBKM2",
    "status": "pending",
    "customer_id": "usr_01H8X2QMNP7KZDX92F3CJVBKM2",
    "total": 9999,
    "currency": "usd",
    "line_items": [...],
    "metadata": {},
    "created_at": "2026-07-26T17:00:00Z"
  },
  "meta": {
    "request_id": "req_01H8X2QMNP7KZDX92F3CJVBKM2",
    "environment": "live",
    "api_version": "2026-07-01",
    "usage": {
      "orders_this_month": 47,
      "monthly_limit": 100,
      "percent_used": 47.0,
      "resets_at": "2026-08-01T00:00:00Z"
    },
    "latency_ms": 23
  }
}
```

```json
// Error Response — Tier Limit Exceeded
HTTP/1.1 402 Payment Required
{
  "error": {
    "code": "usage_limit_exceeded",
    "message": "You have used 100/100 orders for this month on the Free tier.",
    "param": "orders_created",
    "upgrade_url": "https://dashboard.commerceplatform.io/upgrade",
    "docs_url": "https://docs.commerceplatform.io/errors/usage_limit_exceeded"
  },
  "meta": {
    "request_id": "req_01H8X2QMNP7KZDX92F3CJVBKM2",
    "environment": "live",
    "usage": {
      "orders_this_month": 100,
      "monthly_limit": 100,
      "percent_used": 100.0,
      "resets_at": "2026-08-01T00:00:00Z"
    }
  }
}
```

### 7.2 Critical Response Headers

Every response includes these headers (like Stripe and Supabase do):

```
X-Request-Id: req_01H8X2QMNP7KZDX92F3CJVBKM2
X-Commerce-Version: 2026-07-01
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 93
X-RateLimit-Reset: 1753574400
X-Usage-Orders-Remaining: 53
X-Environment: live
```

### 7.3 Rust Implementation

```rust
// Response wrapper — add this to every handler
pub struct ApiResponse<T: Serialize> {
    pub data: T,
    pub tenant: TenantContext,
    pub usage: UsageSummary,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn into_http_response(self, status: StatusCode) -> HttpResponse {
        let latency_ms = self.tenant.request_start
            .map(|s| s.elapsed().as_millis() as u64)
            .unwrap_or(0);

        let body = serde_json::json!({
            "data": self.data,
            "meta": {
                "request_id": self.tenant.request_id,
                "environment": self.tenant.environment,
                "api_version": "2026-07-01",
                "usage": self.usage,
                "latency_ms": latency_ms,
            }
        });

        HttpResponse::build(status)
            .insert_header(("X-Request-Id", self.tenant.request_id.as_str()))
            .insert_header(("X-Commerce-Version", "2026-07-01"))
            .insert_header(("X-RateLimit-Limit", self.tenant.rate_limit.to_string()))
            .insert_header(("X-Environment", format!("{:?}", self.tenant.environment)))
            .insert_header(("X-Usage-Orders-Remaining",
                self.usage.monthly_limit.saturating_sub(self.usage.orders_this_month).to_string()))
            .json(body)
    }
}
```

---

## 8. Event Mesh Isolation

### 8.1 Why This is Critical

Without tenant isolation in the event mesh, this happens:

```
Tenant A places order → publishes to stream:orders
Tenant B's inventory consumer reads from stream:orders
→ Tenant B's inventory drops for Tenant A's order
→ Silent data corruption and security breach
```

### 8.2 Current Implementation (platform/src/streams.rs)

The `StreamPublisher` already extracts `tenant_id` from events and stores it as a top-level field:

```rust
// From streams.rs — the tenant_id is added as a Redis stream field
let res: String = redis::cmd("XADD")
    .arg(stream)
    .arg("*")
    .arg("event_type").arg(event_type)
    .arg("tenant_id").arg(&tenant_str)   // ← Critical: stored at stream level
    .arg("payload").arg(payload)
    .query_async(&mut *conn)
    .await?;
```

### 8.3 All Event Structs Must Include tenant_id

Every event struct across all microservices **must** have this field:

```rust
// REQUIRED pattern for ALL events
#[derive(Serialize, Deserialize)]
pub struct OrderCreatedEvent {
    pub tenant_id: Uuid,         // ← MANDATORY. Never omit.
    pub environment: String,     // ← "test" or "live"
    pub order_id: Uuid,
    pub customer_id: Uuid,
    pub line_items: Vec<LineItem>,
    pub total: i64,
    pub currency: String,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// Every consumer MUST validate tenant_id before touching the DB
pub async fn handle_order_created(
    envelope: StreamEnvelope<OrderCreatedEvent>,
    db_router: &DynamicPoolRouter,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let event = &envelope.payload;

    // GUARD: Ensure the stream-level tenant_id matches the payload tenant_id
    if envelope.tenant_id != Some(event.tenant_id) {
        tracing::error!(
            stream_tenant = ?envelope.tenant_id,
            payload_tenant = %event.tenant_id,
            "CRITICAL: tenant_id mismatch between stream envelope and payload. Rejecting."
        );
        return Err("tenant_id mismatch".into());
    }

    // Build a fake TenantContext to route to the correct DB pool
    let ctx = TenantContext::new(event.tenant_id, None, PricingTier::Free, vec![], AuthMethod::ApiKey);
    let pool = db_router.get_pool(&ctx).await?;
    let mut tx = pool.begin().await?;

    // Apply RLS before ANY query
    ctx.apply_rls(&mut *tx).await?;

    // Now operate on inventory — RLS guarantees tenant isolation at DB level too
    Inventory::reserve(&mut *tx, &event.tenant_id, &event.line_items).await?;

    tx.commit().await?;
    Ok(())
}
```

### 8.4 Dead Letter Queue (DLQ) — Already Implemented

The `StreamPublisher.publish_async()` already routes failed events to `stream:dlq`:

```rust
// From streams.rs — DLQ routing on publish failure
tracing::warn!(%event_type, error = %error_str, "redis stream publish failed, routing to DLQ");
redis::cmd("XADD")
    .arg("stream:dlq")
    .arg("*")
    .arg("event_type").arg(&event_type)
    .arg("tenant_id").arg(&tenant_str)
    .arg("payload").arg(payload)
    .arg("error").arg(&error_str)  // ← Error reason stored for debugging
    .query_async(&mut *conn)
    .await;
```

**DLQ Consumer** — a background task that replays events with exponential backoff:

```rust
pub async fn run_dlq_consumer(redis_url: &str) {
    consume_json::<serde_json::Value, _, _>(
        redis_url,
        "dlq-consumer-group",
        "dlq-worker-1",
        &["*"], // Match all event types
        |envelope| async move {
            let event_type = &envelope.event_type;
            let attempts: u32 = /* read from envelope metadata */ 0;
            let delay = Duration::from_millis(1000 * 2u64.pow(attempts)); // Exponential backoff
            tokio::time::sleep(delay.min(Duration::from_secs(300))).await;
            // Re-publish to the original stream
            // If max_attempts exceeded, move to dead-dead-letter queue or alert
            Ok(())
        }
    ).await.unwrap();
}
```

---

## 9. Webhook System

### 9.1 Architecture (Like Stream's Webhook System)

```
Platform Event (e.g. order.created)
    │
    ▼
notifications microservice (already exists)
    │
    ▼
WebhookDispatcher
    │
    ├── Looks up tenant's webhook endpoints
    │   (filtered by subscribed event types)
    │
    ├── Builds signed payload:
    │   - JSON body
    │   - HMAC-SHA256 signature using whsec_...
    │
    ├── POST to tenant's endpoint
    │
    ├── Success (2xx) → XACK from stream, mark delivered
    │
    └── Failure → push to webhook_dlq with attempt count
            │
            └── DLQ worker retries with exponential backoff
                Max 5 attempts, then mark as failed, alert tenant
```

### 9.2 Webhook Endpoint Schema

```sql
-- In control-plane database
CREATE TABLE webhook_endpoints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    description TEXT,
    secret VARCHAR(64) NOT NULL,              -- The whsec_... value (stored hashed, shown once)
    secret_hash VARCHAR(64) NOT NULL,
    event_types TEXT[] NOT NULL,              -- ['order.created', 'payment.failed', '*']
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE webhook_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint_id UUID NOT NULL REFERENCES webhook_endpoints(id),
    tenant_id UUID NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    event_id UUID NOT NULL,
    payload JSONB NOT NULL,
    status VARCHAR(20) NOT NULL,              -- 'pending' | 'delivered' | 'failed'
    attempts INTEGER NOT NULL DEFAULT 0,
    last_attempt_at TIMESTAMPTZ,
    response_status INTEGER,
    response_body TEXT,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### 9.3 HMAC Signing — The Verification Contract

**Platform side (signing the payload):**

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

pub fn sign_webhook_payload(
    payload: &[u8],
    secret: &str,
    timestamp: i64,
) -> String {
    type HmacSha256 = Hmac<Sha256>;

    // Stripe-style: sign "timestamp.payload"
    let signed_content = format!("{}.{}", timestamp, String::from_utf8_lossy(payload));

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC accepts any key length");
    mac.update(signed_content.as_bytes());
    let result = mac.finalize();

    format!("t={},v1={}", timestamp, hex::encode(result.into_bytes()))
}

pub async fn dispatch_webhook(
    endpoint: &WebhookEndpoint,
    event_type: &str,
    payload: serde_json::Value,
    client: &reqwest::Client,
) -> Result<u16, reqwest::Error> {
    let timestamp = chrono::Utc::now().timestamp();
    let body = serde_json::to_vec(&payload).unwrap();
    let signature = sign_webhook_payload(&body, &endpoint.secret, timestamp);

    let response = client
        .post(&endpoint.url)
        .header("Content-Type", "application/json")
        .header("X-Commerce-Signature", &signature)    // Like Stripe's Stripe-Signature
        .header("X-Commerce-Event", event_type)
        .header("X-Commerce-Delivery-Id", Uuid::new_v4().to_string())
        .header("X-Commerce-Timestamp", timestamp.to_string())
        .body(body)
        .timeout(Duration::from_secs(5))
        .send()
        .await?;

    Ok(response.status().as_u16())
}
```

**Tenant side (verifying the signature in their code):**

```rust
// Example for a Rust-based customer
pub fn verify_webhook_signature(
    payload: &[u8],
    signature_header: &str,  // "t=1234,v1=abc123..."
    secret: &str,
    tolerance_seconds: i64,
) -> Result<(), WebhookError> {
    // Parse header: t=timestamp,v1=sig
    let parts: HashMap<&str, &str> = signature_header
        .split(',')
        .filter_map(|s| { let mut kv = s.splitn(2, '='); Some((kv.next()?, kv.next()?)) })
        .collect();

    let timestamp: i64 = parts.get("t").ok_or(WebhookError::MissingTimestamp)?
        .parse().map_err(|_| WebhookError::InvalidTimestamp)?;
    let sig = parts.get("v1").ok_or(WebhookError::MissingSignature)?;

    // Check timestamp freshness (prevent replay attacks)
    let now = chrono::Utc::now().timestamp();
    if (now - timestamp).abs() > tolerance_seconds {
        return Err(WebhookError::TimestampTooOld);
    }

    // Recompute expected signature
    let signed_content = format!("{}.{}", timestamp, String::from_utf8_lossy(payload));
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(signed_content.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());

    if !constant_time_compare(&expected, sig) {
        return Err(WebhookError::InvalidSignature);
    }
    Ok(())
}
```

---

## 10. Usage Metering & Billing Engine

### 10.1 What Gets Metered

Every billable action is recorded in the analytics service:

| Metric | How Counted | Free Limit | Growth Limit |
|--------|-------------|------------|-------------|
| `orders_created` | Per order creation | 100/month | 10,000/month |
| `api_calls` | Per HTTP request | 10,000/month | 1,000,000/month |
| `webhook_deliveries` | Per webhook POST | 1,000/month | 100,000/month |
| `storage_bytes` | Media stored in Cloudinary | 1 GB | 50 GB |
| `active_products` | Products in catalog | 100 | 10,000 |
| `active_vendors` | Supplier accounts | 1 | 50 |

### 10.2 Real-Time Usage Tracking (Redis + TimescaleDB)

```rust
// Fast path: Redis increments (atomic, microsecond latency)
pub async fn increment_usage(
    redis: &Pool,
    tenant_id: &Uuid,
    metric: &str,
    amount: i64,
) -> Result<i64, Box<dyn std::error::Error>> {
    let mut conn = redis.get().await?;
    let now = chrono::Utc::now();
    // Key format: usage:{tenant_id}:{metric}:{YYYY-MM}
    let key = format!("usage:{}:{}:{}", tenant_id, metric, now.format("%Y-%m"));

    let new_value: i64 = redis::cmd("INCRBY")
        .arg(&key)
        .arg(amount)
        .query_async(&mut *conn)
        .await?;

    // Set TTL to 40 days (covers current month + buffer)
    let _: () = redis::cmd("EXPIRE")
        .arg(&key)
        .arg(40 * 24 * 3600)
        .query_async(&mut *conn)
        .await?;

    Ok(new_value)
}

pub async fn check_usage_limit(
    redis: &Pool,
    tenant: &TenantContext,
    metric: &str,
) -> Result<UsageCheckResult, Box<dyn std::error::Error>> {
    let mut conn = redis.get().await?;
    let now = chrono::Utc::now();
    let key = format!("usage:{}:{}:{}", tenant.tenant_id, metric, now.format("%Y-%m"));

    let current: i64 = redis::cmd("GET")
        .arg(&key)
        .query_async::<_, Option<i64>>(&mut *conn)
        .await?
        .unwrap_or(0);

    let limit = tenant.tier.monthly_order_limit() as i64;
    let percent_used = (current as f64 / limit as f64) * 100.0;

    // Warn at 80% usage
    if percent_used >= 80.0 && percent_used < 100.0 {
        // Send usage warning notification to tenant (async)
    }

    Ok(UsageCheckResult {
        current,
        limit,
        percent_used,
        is_exceeded: current >= limit,
    })
}
```

### 10.3 In Every Handler — The Usage Gate

```rust
#[post("/v1/orders")]
pub async fn create_order_handler(
    tenant: ReqData<TenantContext>,
    redis: Data<Pool>,
    db_router: Data<DynamicPoolRouter>,
    req: Json<CreateOrderRequest>,
) -> impl Responder {

    // 1. CHECK USAGE BEFORE DOING ANYTHING
    let usage_check = check_usage_limit(&redis, &tenant, "orders_created").await.unwrap();
    if usage_check.is_exceeded {
        return HttpResponse::PaymentRequired().json(serde_json::json!({
            "error": {
                "code": "usage_limit_exceeded",
                "message": format!("You have used {}/{} orders this month.",
                    usage_check.current, usage_check.limit),
                "upgrade_url": "https://dashboard.commerceplatform.io/upgrade",
            },
            "meta": {
                "request_id": tenant.request_id,
                "usage": usage_check,
            }
        }));
    }

    // 2. Execute the order creation
    let pool = db_router.get_pool(&tenant).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    tenant.apply_rls(&mut *tx).await.unwrap();

    let order = Order::create(&mut *tx, &tenant.tenant_id, req.into_inner())
        .await.unwrap();

    tx.commit().await.unwrap();

    // 3. ASYNC: Increment usage counter (fire-and-forget)
    let redis_clone = redis.clone();
    let tenant_id = tenant.tenant_id;
    tokio::spawn(async move {
        let _ = increment_usage(&redis_clone, &tenant_id, "orders_created", 1).await;
    });

    // 4. Return response with usage info in metadata
    let usage_after = check_usage_limit(&redis, &tenant, "orders_created").await.unwrap();
    ApiResponse { data: order, tenant: tenant.into_inner(), usage: usage_after }
        .into_http_response(StatusCode::CREATED)
}
```

---

## 11. Complete Database Schema (Multi-Tenant)

### 11.1 Core Migration — Adding tenant_id to All Tables

```sql
-- Migration: 0001_multi_tenant_baseline.sql
-- Apply to ALL microservice databases

-- ============================================================
-- TENANTS TABLE (in control-plane DB, referenced by all others)
-- ============================================================
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL,  -- URL-safe identifier
    tier VARCHAR(50) NOT NULL DEFAULT 'free',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================
-- ORDERS DATABASE (order-service)
-- ============================================================
ALTER TABLE orders ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL;
ALTER TABLE orders ADD COLUMN IF NOT EXISTS metadata JSONB NOT NULL DEFAULT '{}';
ALTER TABLE orders ADD COLUMN IF NOT EXISTS environment VARCHAR(8) NOT NULL DEFAULT 'live';

CREATE INDEX IF NOT EXISTS idx_orders_tenant_status ON orders(tenant_id, status);
CREATE INDEX IF NOT EXISTS idx_orders_tenant_customer ON orders(tenant_id, customer_id);
CREATE INDEX IF NOT EXISTS idx_orders_tenant_created ON orders(tenant_id, created_at DESC);
-- GIN index for metadata queries: GET /v1/orders?metadata.campaign_id=summer
CREATE INDEX IF NOT EXISTS idx_orders_metadata_gin ON orders USING GIN (metadata);

-- ============================================================
-- INVENTORY DATABASE (inventory-management)
-- ============================================================
ALTER TABLE inventory ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL;
ALTER TABLE inventory ADD COLUMN IF NOT EXISTS metadata JSONB NOT NULL DEFAULT '{}';

CREATE UNIQUE INDEX IF NOT EXISTS idx_inventory_tenant_product
    ON inventory(tenant_id, product_id);
CREATE INDEX IF NOT EXISTS idx_inventory_tenant_id ON inventory(tenant_id);

-- ============================================================
-- PRODUCTS DATABASE (product-catalog)
-- ============================================================
ALTER TABLE products ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL;
ALTER TABLE products ADD COLUMN IF NOT EXISTS metadata JSONB NOT NULL DEFAULT '{}';

CREATE INDEX IF NOT EXISTS idx_products_tenant_id ON products(tenant_id);
CREATE INDEX IF NOT EXISTS idx_products_tenant_active ON products(tenant_id, is_active);
CREATE INDEX IF NOT EXISTS idx_products_metadata_gin ON products USING GIN (metadata);

-- ============================================================
-- USERS DATABASE (user-management)
-- ============================================================
ALTER TABLE users ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL;
ALTER TABLE users ADD COLUMN IF NOT EXISTS metadata JSONB NOT NULL DEFAULT '{}';

CREATE UNIQUE INDEX IF NOT EXISTS idx_users_tenant_email ON users(tenant_id, email);
CREATE INDEX IF NOT EXISTS idx_users_tenant_id ON users(tenant_id);

-- ============================================================
-- PAYMENTS DATABASE (payments)
-- ============================================================
ALTER TABLE payment_intents ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL;
ALTER TABLE payment_intents ADD COLUMN IF NOT EXISTS metadata JSONB NOT NULL DEFAULT '{}';

CREATE INDEX IF NOT EXISTS idx_payments_tenant_id ON payment_intents(tenant_id);
CREATE INDEX IF NOT EXISTS idx_payments_tenant_order ON payment_intents(tenant_id, order_id);

-- ============================================================
-- LOGISTICS DATABASE (logistics)
-- ============================================================
ALTER TABLE shipments ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL;

CREATE INDEX IF NOT EXISTS idx_shipments_tenant_id ON shipments(tenant_id);

-- ============================================================
-- NOTIFICATIONS DATABASE (notifications)
-- ============================================================
ALTER TABLE notifications ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL;

CREATE INDEX IF NOT EXISTS idx_notifications_tenant_id ON notifications(tenant_id);

-- ============================================================
-- ANALYTICS DATABASE (TimescaleDB - analytics)
-- ============================================================
-- This table already handles time-series, we add tenant dimension
ALTER TABLE analytics_events ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL;
ALTER TABLE analytics_events ADD COLUMN IF NOT EXISTS environment VARCHAR(8);

-- Tenant-aware hypertable chunk index
CREATE INDEX IF NOT EXISTS idx_analytics_tenant_time
    ON analytics_events(tenant_id, time DESC);
```

---

## 12. Dashboard Configuration APIs

These are the **management APIs** tenants use in their dashboard — equivalent to the
Supabase Dashboard Settings or the Stream Dashboard App Settings.

### 12.1 API Key Management

```
# Create a new API key
POST /v1/dashboard/api-keys
Authorization: Bearer sk_live_...

Request:
{
  "name": "Production Server",
  "environment": "live",
  "type": "sk",
  "scopes": ["orders:read", "orders:write", "products:read", "inventory:read"]
}

Response (201):
{
  "data": {
    "id": "key_01H8X2QMNP7KZDX92F3CJVBKM2",
    "name": "Production Server",
    "prefix": "sk_live_7c",
    "environment": "live",
    "scopes": ["orders:read", "orders:write", "products:read", "inventory:read"],
    "created_at": "2026-07-26T17:00:00Z",
    "plaintext": "sk_live_EXAMPLE_KEY_REDACTED"
    // ↑ ONLY returned once. Never retrievable again.
  }
}

# List API keys (prefixes only, never hashes or plaintexts)
GET /v1/dashboard/api-keys

# Revoke an API key
DELETE /v1/dashboard/api-keys/{key_id}

# Roll a key (atomic: create new, revoke old in one operation)
POST /v1/dashboard/api-keys/{key_id}/roll
```

### 12.2 Webhook Configuration

```
# Register a webhook endpoint
POST /v1/dashboard/webhooks
{
  "url": "https://myapp.com/webhooks/commerce",
  "description": "Order lifecycle events",
  "event_types": ["order.created", "order.updated", "payment.succeeded", "payment.failed"]
}

Response (201):
{
  "data": {
    "id": "wh_01H8X2QMNP7KZDX92F3CJVBKM2",
    "url": "https://myapp.com/webhooks/commerce",
    "event_types": ["order.created", "order.updated", "payment.succeeded", "payment.failed"],
    "secret": "whsec_EXAMPLE_SECRET_REDACTED",  // ← ONLY returned once
    "is_active": true,
    "created_at": "2026-07-26T17:00:00Z"
  }
}

# View webhook delivery logs
GET /v1/dashboard/webhooks/{webhook_id}/deliveries?status=failed&limit=50

# Replay a failed delivery
POST /v1/dashboard/webhooks/{webhook_id}/deliveries/{delivery_id}/replay

# Test a webhook endpoint (sends a test event)
POST /v1/dashboard/webhooks/{webhook_id}/test
{ "event_type": "order.created" }
```

### 12.3 Platform Configuration

```
# Get the full project config (like Firebase initializeApp data)
GET /v1/dashboard/config
Authorization: Bearer sk_live_...

Response:
{
  "project": {
    "tenant_id": "ten_01H8X2QMNP7KZDX92F3CJVBKM2",
    "name": "Acme Corp",
    "environment": "live",
    "region": "us-east-1",
    "api_base_url": "https://api.commerceplatform.io/v1"
  },
  "limits": {
    "orders_per_month": 10000,
    "api_calls_per_month": 1000000,
    "active_products": 10000,
    "webhook_endpoints": 10,
    "storage_gb": 50
  },
  "settings": {
    "order_mode": "managed",
    "inventory_reservation_ttl_seconds": 900,
    "payment_providers": ["stripe"],
    "notification_channels": ["email", "sms"],
    "ip_allowlist": [],
    "rate_limit_rps": 100
  },
  "features": {
    "multi_vendor": true,
    "real_time_inventory": true,
    "vendor_analytics": false,
    "custom_metadata": true,
    "webhook_streaming": true,
    "dedicated_database": false
  }
}

# Update configuration
PATCH /v1/dashboard/config
{
  "settings": {
    "inventory_reservation_ttl_seconds": 1800,
    "order_mode": "vendor"
  }
}
```

### 12.4 Usage & Analytics

```
# Current month usage
GET /v1/dashboard/usage

Response:
{
  "period": "2026-07",
  "metrics": {
    "orders_created": { "value": 4750, "limit": 10000, "percent": 47.5 },
    "api_calls": { "value": 152000, "limit": 1000000, "percent": 15.2 },
    "webhook_deliveries": { "value": 9100, "limit": 100000, "percent": 9.1 },
    "storage_bytes": { "value": 2147483648, "limit": 53687091200, "percent": 4.0 }
  },
  "billing_cycle_ends": "2026-08-01T00:00:00Z"
}

# API request log (last 1000 requests)
GET /v1/dashboard/logs?status=500&service=order-service&limit=100

Response:
{
  "data": [
    {
      "request_id": "req_01H8X2...",
      "timestamp": "2026-07-26T16:45:22Z",
      "method": "POST",
      "path": "/v1/orders",
      "status": 500,
      "latency_ms": 234,
      "ip": "203.0.113.1"
    }
  ]
}
```

---

## 13. Secrets Management

### 13.1 Environment Classification

| Variable | Classification | How to Handle |
|----------|---------------|---------------|
| `DATABASE_URL` | 🔴 Critical Secret | AWS Secrets Manager / HashiCorp Vault |
| `SECRET` (JWT) | 🔴 Critical Secret | AWS Secrets Manager |
| `PLATFORM_MASTER_KEY` | 🔴 Critical Secret | Hardware Security Module (HSM) in prod |
| `STRIPE_SECRET_KEY` | 🔴 Critical Secret | AWS Secrets Manager |
| `STRIPE_WEBHOOK_SECRET` | 🔴 Critical Secret | AWS Secrets Manager |
| `SENDGRID_API_KEY` | 🟡 High Sensitivity | AWS Secrets Manager |
| `TWILIO_AUTH_TOKEN` | 🟡 High Sensitivity | AWS Secrets Manager |
| `INTERNAL_SERVICE_TOKEN` | 🟡 High Sensitivity | AWS Secrets Manager |
| `REDIS_URL` | 🟡 High Sensitivity | AWS Secrets Manager or env |
| `CLOUDINARY_API_SECRET` | 🟡 High Sensitivity | AWS Secrets Manager |
| `NOTIFICATION_DRY_RUN` | 🟢 Config | Plain env var |
| `RUST_LOG` | 🟢 Config | Plain env var |
| `*_PORT` variables | 🟢 Config | Plain env var or docker-compose |

### 13.2 Secrets Never Stored in Plaintext

The following are **never stored in plaintext** anywhere in the database:
- API Keys → stored as SHA-256 hash. Plaintext shown once.
- Webhook Secrets → stored as hash. Plaintext shown once.
- Tenant payment provider keys (Paystack, Flutterwave) → AES-256-GCM encrypted in JSONB column using `PLATFORM_MASTER_KEY`.

---

## 14. Phased Execution Roadmap

### Phase 1 — Foundational Tenancy (Weeks 1–3)
- [ ] Create `tenant-management` Actix service with control-plane DB
- [ ] Implement `generate_api_key()`, `validate_api_key()` with Redis caching
- [ ] Build full `TenantMiddleware` with request analysis and analytics publishing
- [ ] Add `tenant_id` + `metadata` columns to all 9 service databases via migrations
- [ ] Implement RLS policies on all tables
- [ ] Update `TenantContext::apply_rls()` call in every handler

### Phase 2 — Event Mesh Hardening (Weeks 4–5)
- [ ] Audit all event structs: ensure `tenant_id` and `environment` fields
- [ ] Add tenant_id validation guard to all Redis Stream consumers
- [ ] Implement DLQ consumer with exponential backoff
- [ ] Add `metadata` to all API request/response schemas

### Phase 3 — Webhook System (Weeks 6–7)
- [ ] Build `WebhookDispatcher` in `notifications` service
- [ ] Implement HMAC-SHA256 payload signing
- [ ] Add `webhook_endpoints` and `webhook_deliveries` tables
- [ ] Build Dashboard API endpoints for webhook CRUD
- [ ] Implement event replay from delivery log

### Phase 4 — Billing & Metering (Weeks 8–10)
- [ ] Integrate Redis-backed usage counters into every write handler
- [ ] Build usage enforcement middleware (402 on limit exceeded)
- [ ] Integrate Stripe Billing for subscription management
- [ ] Build `/v1/dashboard/usage` API backed by TimescaleDB aggregations
- [ ] Implement 80% usage warning notifications

### Phase 5 — Dashboard APIs & DX (Weeks 11–12)
- [ ] Build all `/v1/dashboard/*` endpoints
- [ ] Add response envelope with `meta.usage` to all API responses
- [ ] Add `X-RateLimit-*` and `X-Usage-*` response headers
- [ ] Write SDK initialization documentation matching Firebase/Supabase quality
- [ ] Write webhook verification examples for Node.js, Python, Go, Rust

---

*This document is the master reference for the CaaS platform transformation.
Every engineering decision, every database migration, and every API contract
should be validated against the standards established here.*

## 15. Gap Fills & Extended Configurability

This section fills gaps identified during the audit of the platform against the full configurability of Stream, Supabase, and Firebase. It provides production-grade Rust structs, SQL schemas, JSON configs, and API specs to support maximum configurability for every tenant.

### 15.1 Stream-Inspired Configurability

**A. Channel Types & Moderation (JSON Config)**
Stream allows defining channel types (e.g., messaging, livestream, team) with configurable permissions and moderation.

```json
{
  "channel_types": {
    "support": {
      "typing_events": true,
      "read_events": true,
      "connect_events": true,
      "search": false,
      "reactions": true,
      "replies": true,
      "mutes": true,
      "quotes": true,
      "message_retention": "90d",
      "max_message_length": 5000,
      "automod": "AI",
      "blocklist": "strict",
      "permissions": [
        { "action": "send_message", "roles": ["user", "admin"] },
        { "action": "delete_message", "roles": ["admin"] }
      ]
    }
  },
  "moderation": {
    "profanity_filter": {
      "enabled": true,
      "action": "flag", // flag, block, or mask
      "custom_words": ["badword1", "badword2"]
    }
  }
}
```

**B. Push Notification Credentials & Multi-Region (Rust & SQL)**
Credentials for APNs/FCM must be stored encrypted per-app. Rate limits can be overridden per app.

```sql
-- Adding dedicated push credentials and region config to the control plane
ALTER TABLE tenants 
ADD COLUMN push_credentials JSONB DEFAULT '{}', -- Encrypted APNs/FCM keys
ADD COLUMN region VARCHAR(50) DEFAULT 'us-east-1',
ADD COLUMN rate_limit_overrides JSONB DEFAULT '{}'; -- Fine-grained overrides per endpoint
```

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct PushCredentials {
    pub apns: Option<ApnsConfig>,
    pub fcm: Option<FcmConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApnsConfig {
    pub key_id: String,
    pub team_id: String,
    pub bundle_id: String,
    #[serde(skip_serializing)] // Never leak the cert key
    pub p8_certificate: String, 
    pub environment: String, // sandbox or production
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FcmConfig {
    pub project_id: String,
    #[serde(skip_serializing)]
    pub server_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RateLimitOverrides {
    pub default_rps: u32,
    pub endpoints: HashMap<String, u32>,
}
```

**C. Token Authentication (Server-Side Token Generation)**
Like Stream, tenants generate JWT tokens on their backend for their users to communicate directly with our edge.

```rust
use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TenantUserClaims {
    pub tenant_id: String,
    pub user_id: String,
    pub role: String,
    pub exp: usize,
}

pub fn generate_tenant_user_token(
    tenant_secret: &str,
    tenant_id: &str,
    user_id: &str,
    role: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims = TenantUserClaims {
        tenant_id: tenant_id.to_string(),
        user_id: user_id.to_string(),
        role: role.to_string(),
        exp: expiration as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(tenant_secret.as_bytes()),
    )
}
```

### 15.2 Supabase-Inspired Configurability

**A. Storage Bucket Policies & Provider Configs**
Supabase supports public/private buckets and granular OAuth configs per provider.

```sql
CREATE TABLE storage_buckets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    is_public BOOLEAN NOT NULL DEFAULT false,
    file_size_limit BIGINT NOT NULL DEFAULT 52428800, -- 50MB default
    allowed_mime_types TEXT[] DEFAULT '{"image/*", "application/pdf"}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- RLS Policy mapping to bucket configs
CREATE POLICY bucket_access_policy ON storage_objects
    FOR SELECT USING (
        bucket_id IN (SELECT id FROM storage_buckets WHERE is_public = true) OR
        tenant_id = current_setting('app.current_tenant_id', true)::uuid
    );
```

```json
{
  "auth_providers": {
    "google": {
      "enabled": true,
      "client_id": "google-client-id.apps.googleusercontent.com",
      "client_secret": "encrypted-secret"
    },
    "github": {
      "enabled": false,
      "client_id": "",
      "client_secret": ""
    },
    "email": {
      "enabled": true,
      "template_welcome": "Welcome {{user_name}} to {{app_name}}!",
      "smtp": {
        "host": "smtp.sendgrid.net",
        "port": 587,
        "user": "apikey",
        "pass": "encrypted-smtp-pass"
      }
    }
  }
}
```

**B. Edge Function Secrets & PostgREST Schema Exposure**

```sql
CREATE TABLE edge_function_secrets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    function_name VARCHAR(255) NOT NULL,
    secret_key VARCHAR(255) NOT NULL,
    secret_value TEXT NOT NULL, -- Encrypted at rest
    UNIQUE(tenant_id, function_name, secret_key)
);

ALTER TABLE tenants ADD COLUMN exposed_schemas TEXT[] DEFAULT '{"public"}';
```

**C. Realtime Subscription Config**

```json
{
  "realtime": {
    "enabled": true,
    "max_connections_per_user": 5,
    "broadcast": {
      "ack": true
    },
    "presence": {
      "key": "user_id"
    },
    "tables": [
      { "name": "orders", "events": ["INSERT", "UPDATE"] },
      { "name": "inventory", "events": ["UPDATE"] }
    ]
  }
}
```

### 15.3 Firebase-Inspired Configurability

**A. Remote Config & Security Rules**

```sql
CREATE TABLE remote_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    environment VARCHAR(50) NOT NULL DEFAULT 'production', -- support for multiple environments
    key VARCHAR(255) NOT NULL,
    value JSONB NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, environment, key)
);

CREATE TABLE security_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    resource_type VARCHAR(50) NOT NULL, -- 'firestore', 'storage', etc.
    rule_definition TEXT NOT NULL, -- The AST or rule script
    version INTEGER NOT NULL DEFAULT 1,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**B. Analytics Taxonomy & Multiple Environments**
Each tenant can define multiple project environments (development, staging, production) within the same tenant context, mapped to different API keys.

```sql
-- Environment support natively in tenants API Keys mapping
ALTER TABLE api_keys ADD COLUMN target_environment VARCHAR(50) DEFAULT 'production';

CREATE TABLE analytics_taxonomy (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    event_name VARCHAR(255) NOT NULL,
    schema JSONB NOT NULL, -- JSON Schema for validating the event properties
    is_active BOOLEAN DEFAULT true
);
```

### 15.4 Request Analysis & Critical Response Information

Every response must contain essential observability headers. A `TenantAnalyticsMiddleware` captures all this.

**Headers Injected on Every Response:**
```http
X-Request-Id: req_8f7b3a1c-9d2e-4f4a-b5c6-7e8f9a0b1c2d
X-Tenant-Id: ten_01H8X2QMNP7KZDX92F3CJVBKM2
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 995
X-RateLimit-Reset: 1718290800
X-Response-Time-Ms: 42
```

**Global Response Envelope Structure:**
```json
{
  "data": { ... },
  "meta": {
    "request_id": "req_8f7b3a1c-9d2e-4f4a-b5c6-7e8f9a0b1c2d",
    "timestamp": "2026-07-26T17:40:00Z",
    "usage": {
      "credits_consumed": 1,
      "billing_tier": "growth"
    }
  }
}
```

**Request Analysis (Rust Middleware Extension):**
```rust
use actix_web::{dev::{ServiceRequest, ServiceResponse}, Error};
use serde_json::json;
// use std::time::Instant;

pub async fn log_request_analysis(
    req: &ServiceRequest, 
    res: &ServiceResponse, 
    latency_ms: u64
) {
    // Tenant context and IDs
    let tenant_id = req.extensions().get::<TenantContext>()
        .map(|c| c.tenant_id.to_string())
        .unwrap_or_else(|| "anonymous".into());
        
    let req_size = req.headers().get("Content-Length")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("0");
    
    // Asynchronously log to TimescaleDB or Kafka
    let log_entry = json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "request_id": req.extensions().get::<String>().unwrap_or(&"unknown".into()),
        "tenant_id": tenant_id,
        "method": req.method().as_str(),
        "path": req.path(),
        "status": res.status().as_u16(),
        "latency_ms": latency_ms,
        "payload_size_bytes": req_size,
        "user_agent": req.headers().get("User-Agent").and_then(|v| v.to_str().ok()).unwrap_or(""),
        "client_ip": req.connection_info().realip_remote_addr().unwrap_or("")
    });
    
    // tokio::spawn(publish_to_analytics(log_entry));
}
```

## 16. SDK Initialization Pattern (Like Firebase initializeApp)

To provide a seamless developer experience, our platform exposes an initialization endpoint that returns the complete configuration blob for the frontend or backend SDK.

### 16.1 Initialization API Spec

```
GET /v1/init
Authorization: Bearer pk_live_EXAMPLE_KEY_REDACTED

Response (200 OK):
{
  "data": {
    "project": {
      "tenant_id": "ten_01H8X2QMNP7KZDX92F3CJVBKM2",
      "name": "Acme Corp",
      "environment": "live",
      "region": "us-east-1",
      "api_base_url": "https://api.commerceplatform.io/v1"
    },
    "features": {
      "multi_vendor": true,
      "real_time_inventory": true,
      "webhook_streaming": false
    },
    "remote_config": {
      "storefront_theme": "dark",
      "checkout_v2_enabled": true
    },
    "realtime": {
      "endpoint": "wss://realtime.commerceplatform.io",
      "heartbeat_interval_ms": 30000
    }
  },
  "meta": {
    "request_id": "req_a1b2c3d4",
    "timestamp": "2026-07-26T17:42:00Z"
  }
}
```

### 16.2 Rust SDK Initialization Pattern

Here is the corresponding Rust client SDK implementation used by tenant backends:

```rust
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct ProjectConfig {
    pub tenant_id: String,
    pub name: String,
    pub environment: String,
    pub region: String,
    pub api_base_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RealtimeConfig {
    pub endpoint: String,
    pub heartbeat_interval_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct InitResponseData {
    pub project: ProjectConfig,
    pub features: HashMap<String, bool>,
    pub remote_config: HashMap<String, serde_json::Value>,
    pub realtime: RealtimeConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct InitResponse {
    pub data: InitResponseData,
}

#[derive(Debug, Clone)]
pub struct CommercePlatformApp {
    client: Client,
    pub config: InitResponseData,
    api_key: String,
}

impl CommercePlatformApp {
    /// Initialize the platform app, similar to Firebase initializeApp()
    pub async fn initialize(api_key: &str) -> Result<Self, reqwest::Error> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap(),
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()?;

        // Perform the bootstrap request
        let response = client
            .get("https://api.commerceplatform.io/v1/init")
            .send()
            .await?
            .json::<InitResponse>()
            .await?;

        Ok(Self {
            client,
            config: response.data,
            api_key: api_key.to_string(),
        })
    }

    /// Retrieve a remote config value safely
    pub fn get_remote_config<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.config.remote_config.get(key).and_then(|val| {
            serde_json::from_value(val.clone()).ok()
        })
    }
}
```

---

## 17. Headless Commerce Architecture — "Backend in a Box"

This is the core thesis: every tenant gets a **complete, production-grade commerce backend** with zero infrastructure setup. They point their storefront at our API — Shopify storefront, a custom Next.js app, a React Native app, whatever — and everything just works.

### 17.1 What "Headless" Means for Our Platform

```
┌──────────────────────────────────────────────────────────────────┐
│                        TENANT'S FRONTEND                         │
│   Shopify Storefront  │  Next.js App  │  React Native  │  CLI    │
└──────────────┬─────────────────────────────────────────┬─────────┘
               │ pk_live_... (public key)                │ sk_live_... (server key)
               ▼                                         ▼
┌──────────────────────────────────────────────────────────────────┐
│                  Commerce Platform API Gateway                   │
│  TenantAuthMiddleware → TenantContext → DynamicPoolRouter        │
├────────────┬───────────┬──────────┬──────────┬───────────────────┤
│  Products  │ Inventory │  Orders  │ Payments │  Notifications    │
│  Catalog   │ Management│  Service │  Service │  + Webhooks       │
├────────────┴───────────┴──────────┴──────────┴───────────────────┤
│                     Platform Event Mesh                          │
│              Redis Streams (StreamPublisher, consume_json)       │
├──────────────────────────────────────────────────────────────────┤
│                  Multi-Tenant Postgres (RLS)                     │
│         DynamicPoolRouter: Shared (Free/Growth) │ Dedicated (Ent)│
└──────────────────────────────────────────────────────────────────┘
```

### 17.2 The Five Commerce Primitives (What Every Tenant Gets)

| Primitive | Service | Key Streams | Description |
|-----------|---------|-------------|-------------|
| **Catalog** | `product-catalog` | `product.created`, `product.updated`, `product.deleted` | Full product management: variants, pricing, assets via Cloudinary, categories, metadata JSONB |
| **Inventory** | `inventory-management` | `inventory.updated`, `inventory.reserved`, `inventory.released`, `inventory.lowstock` | Real-time stock tracking with reservation TTL, multi-warehouse support via supplier_id scoping |
| **Orders** | `order-service` | `order.created`, `order.confirmed`, `order.cancelled`, `order.shipped`, `order.delivered` | Full order lifecycle with expiration worker, optimistic locking via `version` field |
| **Payments** | `payments` | `payment.succeeded`, `payment.failed`, `payment.refunded` | Stripe Connect for multi-vendor splits, per-tenant payment provider config (Paystack/Flutterwave stored encrypted) |
| **Fulfillment** | `logistics` | `logistics.shipment_created`, `logistics.shipment_updated`, `logistics.shipment_cancelled` | Shipment lifecycle tracking, cancellation propagation back to order saga |

### 17.3 The Commerce Saga — Full Order Lifecycle

This is the event-driven saga that glues all five primitives together. Every step is tenant-isolated.

```
                         COMMERCE ORDER SAGA
                         ─────────────────────

1. POST /orders            (order-service)
   ├─► publishes: order.created
   │     tenant_id: ✓, reservation_ttl: from tenant config
   │
2. inventory-management consumer hears order.created
   ├─► Reserves stock (inventory.reserved OR inventory.rejected)
   │
3. order-service consumer hears inventory.reserved
   ├─► Marks order as CONFIRMED
   ├─► publishes: order.confirmed
   ├─► publishes: logistics.shipment_preparation_command
   │
4. payments consumer hears order.confirmed
   ├─► Triggers Stripe PaymentIntent
   ├─► publishes: payment.succeeded OR payment.failed
   │
5. [on payment.failed]
   ├─► publishes: inventory.release_command (undo reservation)
   ├─► publishes: order.cancelled
   │
6. logistics consumer hears logistics.shipment_preparation_command
   ├─► Creates shipment
   ├─► publishes: logistics.shipment_created
   │
7. order-service consumer hears logistics.shipment_created
   ├─► Updates order status to SHIPPED
   ├─► publishes: order.shipped
   │
8. notifications consumer hears order.shipped
   ├─► Sends email/SMS/push to end user
   ├─► Dispatches tenant webhook: POST tenant_webhook_url
   │     HMAC signed with tenant's whsec_...
   │
9. analytics consumer hears all events
   └─► Writes to TimescaleDB hypertables for tenant dashboards
```

### 17.4 Per-Microservice Phase 3 Contract

Every microservice in the platform MUST satisfy all four of these contracts:

#### Contract 1 — TenantAuthMiddleware is active
```rust
// In main.rs — REQUIRED in every service
HttpServer::new(move || {
    let tenant_mw = TenantAuthMiddleware::new()
        .with_redis(redis_client.get_ref().clone());
    App::new()
        .wrap(tenant_mw)              // MANDATORY
        .app_data(db_router.clone())  // DynamicPoolRouter, not PgPool
})
```

#### Contract 2 — Every handler extracts TenantContext
```rust
pub async fn my_handler(
    tenant: web::ReqData<platform::tenant::TenantContext>,
    db_router: web::Data<platform::db_router::DynamicPoolRouter>,
    redis_pub: web::Data<RedisPublisher>,
) -> impl Responder {
    let pool = db_router.get_pool(&tenant).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    tenant.apply_rls(&mut *tx).await.unwrap();
    // DB queries here are scoped to tenant automatically
    tx.commit().await.unwrap();

    redis_pub.publish_async("my.event", MyEvent {
        tenant_id: tenant.tenant_id,  // MANDATORY in every event
    });
}
```

#### Contract 3 — RLS applied before every DB query
Every handler that reads/writes the database MUST start a transaction and call `tenant.apply_rls()` before any query.

#### Contract 4 — tenant_id in every event payload
Every event struct published to a Redis stream MUST have `tenant_id: Uuid` (non-optional). Stream consumers MUST validate `envelope.tenant_id == Some(payload.tenant_id)`.

---

## 18. Service-by-Service Phase 3 Audit and Refactor Guide

### 18.1 `inventory-management` — Status: ~85% Complete

**Already done:**
- TenantAuthMiddleware wrapped in main.rs
- DynamicPoolRouter registered as app data
- All handlers extract TenantContext and DynamicPoolRouter
- tenant.apply_rls() called in every handler transaction

**Remaining gaps:**
- StockUpdateEvent.tenant_id should be Uuid (not Option<Uuid>)
- update_stock uses .publish().await — convert to .publish_async()
- redis_sub/events.rs saga handlers need db_router.get_pool() + apply_rls() before any DB write
- inventory.reserved, inventory.released event structs need environment: String field

### 18.2 `product-catalog` — Status: ~80% Complete

**Already done:**
- TenantAuthMiddleware::new() wrapped in main.rs
- DynamicPoolRouter registered as app data

**Remaining gaps:**
- Handlers must extract TenantContext and call tenant.apply_rls()
- All product event structs need tenant_id: Uuid
- main.rs should pass redis_client to TenantAuthMiddleware::with_redis()

### 18.3 `user-management` — Status: ~70% Complete

**Already done:**
- StreamPublisher wired in main.rs
- publish_async used in handlers

**Remaining gaps:**
- TenantAuthMiddleware NOT yet wrapped in main.rs
- Protected handlers need TenantContext + apply_rls()
- User event structs (user.registered, user.updated) need tenant_id: Uuid

### 18.4 `payments` — Status: ~75% Complete

**Already done:**
- StreamPublisher fully wired (web::Data<StreamPublisher>)
- publish_payment_event() helper function exists

**Remaining gaps:**
- TenantAuthMiddleware NOT yet wrapped in main.rs
- All handlers need TenantContext + db_router + apply_rls()
- publish_payment_event must include tenant_id: tenant.tenant_id

### 18.5 `logistics` — Status: ~75% Complete

**Already done:**
- StreamPublisher wired via publisher.rs wrapper
- Handlers use publish_async

**Remaining gaps:**
- TenantAuthMiddleware NOT yet wrapped in main.rs
- All handlers need TenantContext + apply_rls()
- Shipment event structs need tenant_id: Uuid

### 18.6 `notifications` — Status: ~50% Complete

**Already done:**
- Redis subscriber listening for events
- Email/SMS/Push provider abstraction via NotificationProvider

**Remaining gaps:**
- TenantAuthMiddleware NOT yet wrapped in main.rs
- Handlers need TenantContext + apply_rls()
- Notification DB records must include tenant_id
- Redis subscriber must extract tenant_id from event envelope
- DynamicPoolRouter needs to be wired in main.rs

### 18.7 `analytics` — Status: ~40% Complete

**Already done:**
- TimescaleDB integration exists

**Remaining gaps:**
- TenantAuthMiddleware NOT yet wrapped in main.rs
- DynamicPoolRouter needs to be wired
- All analytics write paths must include tenant_id
- Analytics event consumer must extract tenant_id from Redis stream envelope

### 18.8 `supplier-management` — Status: ~75% Complete

**Already done:**
- StreamPublisher fully wired via web::Data<StreamPublisher>
- publish_supplier_event() helper exists

**Remaining gaps:**
- TenantAuthMiddleware NOT yet wrapped in main.rs
- Handlers need TenantContext + db_router + apply_rls()
- Supplier event structs need tenant_id: Uuid

---

## 19. The Headless Commerce API Contract

### 19.1 Catalog API

```
POST   /v1/products                                    Create product
GET    /v1/products/{supplier_id}/{product_id}         Get single product
PUT    /v1/products/{supplier_id}/{product_id}         Update product
DELETE /v1/products/{supplier_id}/{product_id}         Delete product
GET    /v1/products/{supplier_id}                      List products for supplier
POST   /v1/products/bulk                               Bulk create products
GET    /v1/products/search?q=&category=&min_price=     Search
POST   /v1/products/{supplier_id}/{product_id}/assets  Register asset
GET    /v1/assets/cloudinary/sign-upload               Get signed upload URL
POST   /v1/suppliers                                   Create supplier
GET    /v1/suppliers/{id}                              Get supplier
```

### 19.2 Inventory API

```
POST   /v1/inventory                                   Create inventory record
GET    /v1/inventory/{supplier_id}                     List all inventory for supplier
GET    /v1/inventory/{supplier_id}/{product_id}        Get stock for specific product
POST   /v1/inventory/{supplier_id}/update              Update stock level (+ or -)
DELETE /v1/inventory/{supplier_id}/{product_id}        Remove inventory record
```

### 19.3 Orders API

```
POST   /v1/orders                   Create order
GET    /v1/orders/{id}              Get order by ID
PUT    /v1/orders/{id}/status       Update order status
DELETE /v1/orders/{id}              Cancel order
```

### 19.4 Payments API

```
POST   /v1/payments/intent                 Create PaymentIntent
GET    /v1/payments/intent/{id}            Get PaymentIntent status
POST   /v1/payments/intent/{id}/confirm    Confirm PaymentIntent
POST   /v1/payments/intent/{id}/cancel     Cancel PaymentIntent
POST   /v1/payments/intent/{id}/capture    Capture authorized payment
POST   /v1/payments/refund/{id}            Create refund
POST   /v1/payments/webhook                Stripe webhook receiver (no auth)
```

### 19.5 Notifications API

```
POST   /v1/notifications                               Send notification
GET    /v1/notifications                               List notifications
PUT    /v1/notifications/{id}/read                     Mark as read
POST   /v1/notification-devices                        Register device (push)
PUT    /v1/notification-preferences/user/{user_id}     Update preferences
```

---

## 20. Production Deployment Checklist

### 20.1 Pre-Deploy Verification (per service)

```bash
# 1. Compile check
cargo check -p <service-name>

# 2. Run unit tests
cargo test -p <service-name>

# 3. Verify RLS policies exist on all tables
psql $DATABASE_URL -c "SELECT tablename, policyname FROM pg_policies;"

# 4. Verify tenant_id index exists on every table
psql $DATABASE_URL -c "SELECT tablename, indexname FROM pg_indexes WHERE indexname LIKE '%tenant%';"

# 5. Full workspace compile
cargo check --workspace
```

### 20.2 Required Environment Variables (per service)

```bash
DATABASE_URL=          # Service-specific Postgres connection
REDIS_URL=             # Shared Redis instance
SECRET=                # JWT signing secret
SERVICE_PORT=          # Service port
RUST_LOG=info,<svc>=debug
```

### 20.3 Health Check Endpoints

Every service MUST expose:
- `GET /health` → 200 OK
- `GET /metrics` → Prometheus metrics (via platform::metrics)

---

## 21. Cross-Tenant Data Safety Rules (Non-Negotiable)

1. **Never** pass a raw `PgPool` to a protected handler. Always use `DynamicPoolRouter`.
2. **Never** skip `tenant.apply_rls()` before a database query in a protected context.
3. **Never** publish an event to a Redis stream without `tenant_id` in the payload.
4. **Never** store a webhook secret in plaintext. Only store the SHA-256 hash.
5. **Never** return data from one tenant in a response to another tenant's API key.
6. **Never** log the full value of `sk_live_` or `sk_test_` keys. Only log the prefix.
7. **Always** use `DynamicPoolRouter` so Enterprise tenants get dedicated pool routing.
8. **Always** verify `envelope.tenant_id == Some(payload.tenant_id)` in Redis Stream consumers.

---

## 22. The `platform` Crate — Public API Reference

### 22.1 `platform::tenant`

```rust
use platform::tenant::{TenantContext, PricingTier, AuthMethod, Environment};

// Key fields available in every protected handler
tenant.tenant_id: Uuid           // The tenant's UUID
tenant.user_id: Option<Uuid>     // End-user ID (JWT path only)
tenant.tier: PricingTier         // Free | Growth | Enterprise
tenant.permissions: Vec<String>  // ["orders:read", "orders:write", ...]
tenant.environment: Environment  // Test | Live
tenant.request_id: String        // UUID for request tracing

// Apply RLS — CALL THIS BEFORE EVERY QUERY
tenant.apply_rls(&mut *tx).await?;
```

### 22.2 `platform::db_router`

```rust
use platform::db_router::DynamicPoolRouter;

// Register in main.rs
let db_router = web::Data::new(DynamicPoolRouter::new(pool.clone()));

// Use in handlers — routes Free/Growth to shared, Enterprise to dedicated
let pool: PgPool = db_router.get_pool(&tenant).await?;
```

### 22.3 `platform::streams`

```rust
use platform::streams::{StreamPublisher, consume_json, StreamEnvelope};

// Create publisher
let publisher = StreamPublisher::new(&redis_url)?;
let publisher = StreamPublisher::noop(); // for tests

// Fire-and-forget publish (preferred in handlers)
publisher.publish_async("order.created", my_event);

// Awaitable publish (when error handling needed)
publisher.publish("order.created", &my_event).await?;

// Consumer (background task)
consume_json::<MyEvent, _, _>(
    &redis_url, "my-group", "my-worker", &["order.created"],
    |envelope: StreamEnvelope<MyEvent>| async move {
        assert_eq!(envelope.tenant_id, Some(envelope.payload.tenant_id));
        Ok(())
    }
).await?;
```

### 22.4 `platform::middleware::tenant_middleware`

```rust
use platform::middleware::tenant_middleware::TenantAuthMiddleware;

// With Redis API key caching
let mw = TenantAuthMiddleware::new()
    .with_redis(redis_client.get_ref().clone());

// Apply to Actix App
App::new().wrap(mw)
```

---

## 23. Tenant Onboarding Flow

```
1. Developer signs up at dashboard.commerceplatform.io
   └─► Creates tenant record in control-plane DB
   └─► Generates sk_test_..., pk_test_..., whsec_... (test env)

2. Developer makes first API call:
   curl -X POST https://api.commerceplatform.io/v1/products \
     -H "Authorization: Bearer sk_test_..." \
     -d '{"name":"Blue Widget","price":2999,"supplier_id":"..."}'

3. Platform validates the key:
   - Extracts prefix (first 10 chars)
   - Checks Redis cache (5-min TTL)
   - Falls back to control-plane DB query
   - Builds TenantContext { tenant_id, tier: Free, permissions: [...] }

4. Handler runs with full tenant isolation:
   - DynamicPoolRouter → shared Postgres pool (Free tier)
   - apply_rls() sets: SET LOCAL app.current_tenant_id = '<uuid>'
   - All queries scoped to their data only

5. Event published to Redis stream:
   XADD stream:products * event_type product.created tenant_id <uuid> payload {...}

6. Developer sees in their dashboard:
   - API call logged (request_id, latency, status)
   - Usage: 1/100 products used on Free tier
   - Event visible in webhook test console
```

---

*This document is the definitive production-grade reference for the Commerce-as-a-Service platform.*
*Total sections: 26. Every section maps to real Rust code, SQL, or API contracts in this codebase.*

---

## 24. Developer Experience (DX) & Client SDKs

A headless commerce platform is only as good as the SDKs that wrap it. To achieve Stream/Supabase-level DX, we must provide both Server-Side and Client-Side SDKs.

### 24.1 Client-Side SDK (Frontend, Read-Only)

Used in Next.js, React Native, or mobile apps. Initialized with `pk_test_...` or `pk_live_...`.
- **Constraint:** Cannot write arbitrary data, cannot create webhooks, cannot view other users' orders.
- **Scope:** Read products, manage the current user's cart, initiate checkout.

```typescript
// Example frontend usage
import { CommerceClient } from '@commerceplatform/js-sdk';

const commerce = new CommerceClient('pk_test_EXAMPLE_KEY_REDACTED', {
  region: 'us-east-1'
});

// Fetch products for a storefront
const products = await commerce.catalog.list({ category: 'electronics', limit: 10 });

// Add to cart and initiate checkout (Headless flow)
const cart = await commerce.cart.create();
await cart.addLineItem({ productId: 'prod_123', quantity: 2 });
const checkoutSession = await commerce.checkout.initiate(cart.id);
```

### 24.2 Server-Side SDK (Node.js/Python/Go, Full Write Access)

Used in the tenant's secure backend (e.g., Next.js API Routes, Lambda). Initialized with `sk_test_...` or `sk_live_...`.
- **Constraint:** Complete admin access to the tenant's data isolated by the RLS layer.

```typescript
// Example backend usage (Next.js API route)
import { CommerceAdmin } from '@commerceplatform/node-sdk';

const admin = new CommerceAdmin('sk_test_EXAMPLE_KEY_REDACTED');

// Tenant dynamically creates a new supplier
const supplier = await admin.suppliers.create({
  name: 'Acme Electronics',
  payout_routing_number: '123456789'
});

// Generate a time-scoped signed URL for secure asset upload
const uploadToken = await admin.assets.generateUploadUrl(supplier.id);
```

---

## 25. Integrations Model: Native Defaults & BYOP

The true power of this platform lies in its **out-of-the-box native integrations** coupled with absolute freedom for Enterprise scale.

### 25.1 Native Platform Defaults (The "Zero-Config" Path)

When a developer signs up, they shouldn't have to create a Stripe account, configure SendGrid, or wire up a logistics engine to get started. The platform handles it.

- **Payments:** Powered implicitly by the Platform's Master Stripe Connect Account.
- **Emails:** Handled via the Platform's Native SendGrid setup.
- **Logistics:** Platform-negotiated shipping rates via ShipEngine.

This allows developers to build a fully functional storefront on Day 1 using nothing but our `sk_test_...` key. The platform takes a fractional fee (e.g., 5%) per transaction for providing the native rails.

### 25.2 Bring Your Own Provider (BYOP) - The Enterprise Escape Hatch

To scale to enterprise tenants, we cannot force them into our native integrations. We must support **BYOP (Bring Your Own Provider)**, allowing them to inject their own API keys into our platform so that we bypass our native layers completely.

#### Provider Configuration Schema

The control-plane database stores encrypted integration credentials for each tenant:

```sql
CREATE TABLE tenant_integrations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(id),
    provider_type VARCHAR(50) NOT NULL, -- e.g., 'PAYMENT', 'LOGISTICS', 'EMAIL'
    provider_name VARCHAR(50) NOT NULL, -- e.g., 'STRIPE', 'SHIPENGINE', 'SENDGRID'
    encrypted_credentials JSONB NOT NULL, -- AES-256-GCM encrypted
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE (tenant_id, provider_type, provider_name)
);
```

#### Seamless Provider Handoff

When a tenant initiates a checkout, our `payments` microservice will:
1. Check if `tenant_integrations` has an active Stripe configuration for this `tenant_id`.
2. **If YES (BYOP):** Decrypt the tenant's specific Stripe Secret Key using our internal KMS. Call the Stripe API using *their* key. The funds go directly to the tenant's Stripe account.
3. **If NO (Native):** Fall back to the platform's Native Stripe Connect integration. The funds flow through our master account, and we trigger a programmatic payout to the supplier.

**Result:** Startups launch instantly using our Native defaults. Enterprise companies migrate securely using BYOP, reducing our platform's regulatory compliance burden.

---

## 26. Headless Checkout Orchestration

A CaaS platform shines during checkout orchestration. It requires coordinating 4 microservices simultaneously via robust Event-Driven sagas.

### 26.1 The Orchestration Saga (Cart to Order)

1. **Frontend:** Calls `POST /v1/checkout/intents` (routed to `order-service`).
2. **Order Service (Coordinator):** 
   - Generates a draft `Order` (status: `PENDING`).
3. **Inventory Service (Sync/Async):** 
   - `order-service` emits `inventory.reserve`.
   - `inventory-management` consumes the event, decrements stock, and sets a 15-minute TTL lock in Redis. If stock is unavailable, it fires an `inventory.reserve_failed` event, halting the saga.
4. **Payment Service (Sync):** 
   - `order-service` calls `payments` to generate a `PaymentIntent`. 
   - (This transparently uses the Native Default or the tenant's BYOP Stripe key, as detailed in Section 25).
5. **Frontend:** Renders the Stripe Elements UI using the returned `client_secret`.
6. **Payment Webhook (Async):** 
   - Stripe hits `POST /payments/webhooks`.
   - `payments` validates the HMAC signature (using the Platform's endpoint secret or the tenant's BYOP webhook secret) and publishes `order.paid`.
7. **Resolution (Async):**
   - `order-service` consumes `order.paid` → transitions order to `CONFIRMED`.
   - `inventory-management` consumes `order.paid` → finalizes the stock deduction (removes TTL).
   - `logistics` consumes `order.paid` → generates a `Shipment` record and a shipping label (Native or BYOP ShipEngine).
   - `notifications` consumes `order.paid` → sends order confirmation email to the end-customer (Native or BYOP SendGrid).

### 26.2 Failure Handling & Dead-Letter Queues (DLQ)

If a service is down (e.g., `notifications` goes offline):
- RabbitMQ retries the `order.paid` event based on exponential backoff.
- If it fails after 5 retries, the event is routed to a Dead-Letter Queue (`DLQ:notifications`).
- The Platform Dashboard alerts the tenant, and they can click "Replay Webhooks/Events" from their UI.

---

## 27. Platform Dashboard UI Mapping

To make this platform tangible, this is the bare-minimum UI mapping for the Developer Dashboard (e.g., what the tenant sees at `app.commerceplatform.io`).

### 1. Global Navigation (Sidebar)
- **Project Selection Dropdown:** Switch between "Acme Prod" and "Acme Staging"
- **Overview:** High-level metrics
- **API Keys:** Manage public/secret keys
- **Webhooks:** Register endpoints & view logs
- **Integrations:** Configure BYOP
- **Logs & Analytics:** Raw HTTP request traces
- **Billing & Usage:** Platform costs

### 2. Overview Screen
- **Hero Metrics:** API Calls (last 24h), Active Checkouts, Orders Processed, Webhook Delivery Success Rate (%).
- **Quick Links:** "View Documentation", "Copy API Keys".
- **Recent Activity:** Feed of recent API errors (e.g., "429 Too Many Requests" or "Webhook Failed").

### 3. API Keys Screen
- **Environment Toggle:** Live / Test
- **Standard Keys:** 
  - `pk_test_EXAMPLE_KEY_REDACTED...` (Visible, click to copy)
  - `sk_test_EXAMPLE_KEY_REDACTED...` (Hidden, click "Reveal", rotatable)
- **Restricted Keys (Pro Feature):** 
  - Table of custom keys with scoped permissions (e.g., `orders:read` only) and IP Allowlist inputs.

### 4. Webhooks Screen
- **Endpoints Table:** List of registered URLs (e.g., `https://api.acme.com/webhooks`).
- **Endpoint Detail View:**
  - **Signing Secret:** `whsec_EXAMPLE_SECRET_REDACTED` (Hidden, used for HMAC validation).
  - **Event Subscriptions:** Checkboxes for `order.created`, `payment.failed`, `inventory.low_stock`.
- **Delivery Logs (Crucial DX):**
  - Table of recent webhook attempts.
  - Columns: Timestamp, Event Type, HTTP Status (200, 500), Latency.
  - Click to view exact JSON payload sent and the exact response body received.
  - "Replay Event" button.

### 5. Integrations Screen (The BYOP Engine)
- Grid of "Cards" representing platform integrations.
- **Stripe Card:**
  - Status: "Using Native Default" (Green badge)
  - Button: "Connect Custom Stripe Account" -> Opens a modal to input `sk_live_...` and `webhook_secret`.
- **SendGrid Card:**
  - Status: "Using Native Default"
  - Button: "Configure Custom SMTP / SendGrid"
- **ShipEngine Card:**
  - Status: "Not Configured"
  - Button: "Connect Carrier Account"

### 6. Logs & Analytics Screen
- **API Explorer:** A real-time tail of HTTP requests made by the tenant's API keys.
- Columns: Method, Endpoint, Status Code, Latency, IP Address.
- **Filters:** By Date, Status Code (e.g., "Show me all 500s").

### 7. Settings > Billing
- Current Tier: Free / Growth / Enterprise.
- Metering progress bars:
  - "Orders Processed: 8,432 / 10,000"
  - "API Calls: 1.2M / 2M"
- Invoice history.

## 28. Enterprise SaaS & Commerce Infrastructure Expansion (Stripe/Firebase/Supabase parity)

To position this platform as a true "Backend-as-a-Service for Commerce" (analogous to what Firebase is for mobile apps or Stripe is for payments), the architecture must abstract away heavy commerce complexities into simple configurable primitives. 

### 28.1 Omni-Channel Notifications Engine (Email, SMS, Push)
Similar to **Twilio / Firebase Cloud Messaging (FCM)**:
- **Unified Payload API:** A single `POST /notifications` endpoint where tenants can send an order update. The platform automatically determines whether the user prefers Email, SMS, or Push based on their `user_management` preferences.
- **Provider Fallbacks:** The platform maintains native integrations (e.g., SendGrid for Email, Twilio for SMS). If a primary provider experiences downtime, the system automatically falls back to a secondary provider (e.g., AWS SES, MessageBird) with zero tenant intervention.
- **Transactional Templates:** Tenants can manage localized HTML email templates directly in the platform dashboard, injecting dynamic variables like `{{order.total}}` or `{{shipping.tracking_url}}` which are parsed at runtime by the `notifications` microservice.

### 28.2 Global Edge Compute & Data Residency
Similar to **Supabase Edge Functions / Stripe Global Routing**:
- **Multi-Region RLS (Row Level Security):** Expanding our current PostgreSQL RLS to support Data Residency. European tenants will have their tenant shards physically located in EU data centers (GDPR compliance), while US tenants reside in NA. The `DynamicPoolRouter` in the `platform` crate will automatically route DB connections to the correct regional cluster based on the API key's origin region.
- **Edge Caching for Product Catalog:** Read-heavy commerce operations (like listing a product catalog) will be replicated to Edge nodes (e.g., Cloudflare Workers or Redis Edge). This guarantees sub-50ms catalog load times globally for end-users, bypassing the origin Postgres database entirely.

### 28.3 The Logistics & Fulfillment Abstraction Layer
Similar to **Shippo / EasyPost**:
- **Unified Carrier API:** Tenants don't need to write code for FedEx, UPS, or DHL. They provide their carrier API credentials in the dashboard (BYOP - Bring Your Own Provider), and the `logistics` microservice maps our standard `ShipmentRequest` model to the specific carrier's payload.
- **Smart Routing & Rate Shopping:** The API will automatically query all connected carriers and return the cheapest or fastest shipping option dynamically during the checkout saga.
- **Webhook Translation:** Carrier webhooks (e.g., "Out for Delivery") are ingested by the platform, normalized into our standard `LogisticsEvent`, and pushed to the tenant via our unified Webhook system.

### 28.4 Multi-Party Settlement & Sub-Accounts (The "Connect" Model)
Similar to **Stripe Connect**:
- **B2B2C Architecture:** For enterprise tenants operating marketplaces, they can programmatically spin up "Sub-Tenants" (vendors). 
- **Split Payments:** When an order is processed, the `payments` service automatically splits the revenue: X% goes to the Platform (us), Y% goes to the Tenant, and Z% goes to the Sub-Tenant vendor.
- **Ledger & Reconciliation:** An immutable ledger database tracks all financial movements, handling refunds and chargebacks automatically across the split parties without manual accounting.

### 28.5 Zero-ETL "Bring Your Own Database" (BYOD) Sync
Similar to **Supabase Wrappers / Stripe Data Pipeline**:
- **Real-time Warehouse Sync:** Large merchants need their raw data in Snowflake or BigQuery for advanced BI. Instead of forcing them to use our API to extract data, the `analytics` RabbitMQ firehose can be configured to stream events directly into the tenant's own data warehouse.
- **Zero-Config Streaming:** The tenant simply provides a warehouse connection string in the dashboard, and the platform automatically maintains a 1-to-1 mirror of their catalog, orders, and customer data in their environment.

### 28.6 Unified SDK with Offline-First Caching
Similar to **Firebase Client SDK**:
- **Optimistic UI Updates:** The provided TypeScript/Swift SDKs will maintain a local SQLite/IndexedDB cache of the cart and product catalog. If a buyer adds an item to the cart while in a subway tunnel with no cell service, the SDK accepts the action locally and syncs it with the `order-service` via a background sync queue once connectivity is restored.
- **Real-Time Order Subscriptions:** Utilizing WebSockets mapped to our RabbitMQ events, a frontend client can subscribe to an order ID. The UI will instantly update when the logistics service marks the order as "Shipped" without requiring long-polling.

## 29. API Gateway & Traffic Transformation (Stripe-Level Resilience)
To transition from an internal application to a global SaaS platform, the API Gateway (currently NGINX) must evolve into a deeply intelligent routing layer (e.g., Kong, Tyk, or a custom Rust proxy).
- **Tier-Based Rate Limiting:** Rate limits are no longer global. They are bound to the tenant's API key and their subscription tier (e.g., Free: 10 req/s, Growth: 100 req/s, Enterprise: Custom). The gateway utilizes Redis token buckets to enforce this at the edge before traffic hits the microservices.
- **API Versioning via Headers:** To guarantee backwards compatibility for enterprise clients (like Stripe does), the platform will support header-based versioning (`Platform-Version: 2026-08-19`). If the internal API models change, a middleware layer translates legacy request/response shapes into the current internal standard.
- **Idempotency Locks:** Every mutating endpoint (`POST`, `PUT`, `DELETE`) requires an `Idempotency-Key` header. The gateway will lock on this key using Redis. If a client's network drops and they retry charging a credit card or creating an order, the gateway intercepts the retry and returns the exact cached response from the first successful request, preventing double-billing or duplicate catalog entries.

## 30. The Extensibility & App Store Model (Shopify/Stripe Apps)
A true SaaS platform acts as an operating system. To allow 3rd party developers to build extensions:
- **OAuth 2.0 Scopes & Consent:** We will implement an OAuth2 Authorization Server. Tenants can install "Apps" (e.g., an automated accounting sync tool) and grant them restricted granular scopes like `orders:read` and `payments:read`, without giving up their master API key.
- **Sync Webhook Subscriptions:** Installed apps automatically register dynamic webhook subscriptions for the tenant, allowing 3rd party developers to listen to events securely.

## 31. Zero Trust Security Transformation
Currently, microservices sitting behind the API gateway might implicitly trust each other. A public SaaS requires Zero Trust.
- **Service-to-Service JWTs (mTLS):** All inter-service communication (e.g., `order-service` calling `product-catalog`) must be authenticated. The API gateway issues a short-lived internal JWT containing the specific `tenant_id`, passing it down the stack. If a microservice is ever compromised, it cannot query data outside of the explicit `tenant_id` context bound in the JWT.
- **Signed Egress Webhooks:** All outbound webhooks sent to tenants or apps are cryptographically signed using HMAC SHA-256 (`Platform-Signature` header). This enables clients to definitively verify that the webhook originated from our infrastructure and prevents spoofing attacks.

## 32. Automated Compliance & Data Sovereignty Engine
Handling B2B data means handling strict GDPR (Europe), CCPA (California), and SOC2 compliance.
- **The Right to be Forgotten (Distributed Deletion Saga):** When a tenant deletes a user or their own account, we cannot just delete a row in one database. The platform will dispatch a `TenantDeleted` RabbitMQ event. Every microservice (`analytics`, `inventory`, `payments`) listens to this event and scrubs the tenant's PII from their isolated Postgres shards and TimescaleDB partitions automatically.
- **PII Masking at the Firehose Level:** Before events hit the `analytics` DB or the data warehouse sync pipeline, a masking layer replaces raw emails and phone numbers with irreversible hashes, ensuring BI tools and data lakes remain free of regulated plaintext PII.

## 33. The Developer Portal & API Sandbox
To achieve elite Developer Experience (DX), static documentation is insufficient.
- **Live/Test Mode Split:** Every tenant receives two sets of keys: `pk_test_...`/`sk_test_...` and `pk_live_...`/`sk_live_...`. The platform strictly isolates Test mode data from Live mode data (using separate DB schemas or a `is_live=false` flag). This allows developers to run integration tests against our production API safely.
- **Interactive Developer Dashboard:** The developer dashboard will feature a real-time request logger (similar to Stripe's developer logs), displaying the exact request payload, response body, and execution latency for every API call made with their keys, drastically reducing their integration debugging time.
