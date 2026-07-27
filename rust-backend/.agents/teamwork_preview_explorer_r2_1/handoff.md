# Handoff Report: Milestone R2 - Hybrid Database Multi-Tenancy Architecture Analysis

## 1. Observation

### 1.1 Microservices & Workspace Architecture
- **Root Configuration**: `Cargo.toml` lines 1-15 defines 11 workspace members: `"analytics"`, `"e2e-tests"`, `"inventory-management"`, `"logistics"`, `"notifications"`, `"order-service"`, `"payments"`, `"platform"`, `"product-catalog"`, `"supplier-management"`, `"user-management"`, `"gateway-tests"`.
- **Database Dependency**: `Cargo.toml` line 35 specifies `sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres", "uuid", "chrono"] }`.
- **Shared Utilities**: `platform/src/lib.rs` exports `metrics`, `observability`, and `streams`.

### 1.2 Migration Files & Database Tables
A total of 17 `.sql` migration files were located across 9 domain services:

1. **`user-management`**:
   - `user-management/migrations/20231033_create_user.sql`: Defines `users` table (`id UUID PRIMARY KEY DEFAULT gen_random_uuid()`, `email TEXT NOT NULL`, `password TEXT NOT NULL`, `full_name TEXT NOT NULL`, `role user_role NOT NULL DEFAULT 'user'`, `is_active BOOLEAN NOT NULL DEFAULT TRUE`, `created_at`, `updated_at`, `UNIQUE(id, email)`) and `revoked_tokens` table.
   - `user-management/migrations/20240724_add_email_verified.sql`: Adds `email_verified BOOLEAN NOT NULL DEFAULT FALSE` to `users`.

2. **`supplier-management`**:
   - `supplier-management/migrations/20260516_create_suppliers.sql`: Defines `suppliers` table (`id UUID PRIMARY KEY DEFAULT gen_random_uuid()`, `owner_user_id UUID NOT NULL`, `legal_name TEXT NOT NULL`, `display_name TEXT NOT NULL`, `tax_id TEXT`, `country TEXT NOT NULL DEFAULT 'NG'`, `status supplier_status NOT NULL DEFAULT 'pending'`, `metadata JSONB`, `created_at`, `updated_at`, `UNIQUE(owner_user_id, legal_name)`).

3. **`product-catalog`**:
   - `product-catalog/migrations/20231032__create_products_table.sql`: Defines `products` table (`id UUID PRIMARY KEY`, `product_id UUID NOT NULL UNIQUE`, `supplier_id UUID NOT NULL`, `name TEXT NOT NULL`, `description JSONB`, `category TEXT NOT NULL`, `price DOUBLE PRECISION NOT NULL`, `unit TEXT NOT NULL`, `quantity INTEGER NOT NULL DEFAULT 0`, `available BOOLEAN NOT NULL DEFAULT TRUE`, `low_stock_threshold INTEGER NOT NULL DEFAULT 10`, `created_at`, `updated_at`, `UNIQUE(product_id, id)`).
   - `product-catalog/migrations/20231102_create_product_assets_table.sql`: Defines `product_assets` table (`id UUID PRIMARY KEY`, `product_id UUID NOT NULL REFERENCES products(product_id) ON DELETE CASCADE`, `supplier_id UUID NOT NULL`, `provider TEXT`, `public_id TEXT NOT NULL`, `url TEXT`, `secure_url TEXT`, `is_primary BOOLEAN`).
   - `product-catalog/migrations/20260724_product_catalog_phase6.sql`: Adds `sku TEXT`, `variants JSONB`, `deleted_at TIMESTAMPTZ NULL`, and partial unique indexes `idx_products_supplier_name_unique`, `idx_products_supplier_sku_unique`.

4. **`order-service`**:
   - `order-service/migrations/20231034_order_service.sql`: Defines `orders` table (`id UUID PRIMARY KEY DEFAULT gen_random_uuid()`, `user_id UUID`, `supplier_id UUID`, `product_id UUID`, `items JSONB NOT NULL`, `qty INT NOT NULL DEFAULT 0`, `status order_status NOT NULL DEFAULT 'pending'`, `created_at`, `updated_at`, `expires_at`, `order_timestamp`).
   - `order-service/migrations/20240724_add_order_version.sql`: Adds `version INTEGER NOT NULL DEFAULT 1`.
   - `order-service/migrations/20240724_alter_order_status.sql`: Adds `processing` and `refunded` to `order_status`.
   - `order-service/migrations/20260725_production_order_features.sql`: Adds `deleted_at TIMESTAMPTZ NULL` to `orders` and creates `order_audit_logs` table (`id UUID PRIMARY KEY`, `order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE`, `previous_status VARCHAR(50)`, `new_status VARCHAR(50) NOT NULL`, `changed_at TIMESTAMPTZ`).

5. **`inventory-management`**:
   - `inventory-management/migrations/20231031_create_inventory.sql`: Defines `inventory` table (`id UUID PRIMARY KEY`, `supplier_id UUID`, `product_id UUID`, `name TEXT NOT NULL`, `price DOUBLE PRECISION`, `description JSONB`, `category TEXT NOT NULL`, `quantity INTEGER`, `reserved INTEGER NOT NULL DEFAULT 0`, `created_at`, `updated_at`) and `reservations` table (`reservation_id UUID PRIMARY KEY`, `order_id UUID NOT NULL UNIQUE`, `product_id UUID NOT NULL`, `qty INTEGER NOT NULL`, `user_id UUID NOT NULL`, `expires_at TIMESTAMPTZ`, `released BOOLEAN NOT NULL DEFAULT FALSE`).

6. **`logistics`**:
   - `logistics/migrations/20231101_create_shipments.sql`: Defines `shipments` table (`id UUID PRIMARY KEY`, `order_id UUID NOT NULL UNIQUE`, `user_id UUID NOT NULL`, `supplier_id UUID NOT NULL`, `product_id UUID NOT NULL`, `tracking_number TEXT NOT NULL UNIQUE`, `status shipment_status NOT NULL DEFAULT 'pending'`, `notes TEXT`, `created_at`, `updated_at`, `dispatched_at`, `delivered_at`).

7. **`notifications`**:
   - `notifications/migrations/20240505_create_notifications.sql`: Defines `notifications` table (`id UUID PRIMARY KEY`, `user_id UUID`, `supplier_id UUID`, `order_id UUID`, `event_type TEXT NOT NULL`, `channel notification_channel`, `priority notification_priority`, `recipient TEXT NOT NULL`, `subject TEXT`, `body TEXT NOT NULL`, `payload JSONB`, `status notification_status`, `attempts INTEGER`, `last_error TEXT`, `sent_at`, `read_at`, `created_at`, `updated_at`), `notification_devices` table, and `notification_preferences` table (handled in `notifications/src/db.rs` lines 246-291).

8. **`payments`**:
   - `payments/migrations/20260516_create_payments.sql` & `20240724_alter_payments_amount.sql`: Defines `payment_intents` table (`id UUID PRIMARY KEY`, `idempotency_key TEXT NOT NULL UNIQUE`, `order_id UUID NOT NULL`, `user_id UUID NOT NULL`, `supplier_id UUID NOT NULL`, `product_id UUID NOT NULL`, `quantity INTEGER NOT NULL DEFAULT 1`, `amount BIGINT NOT NULL`, `currency TEXT NOT NULL DEFAULT 'NGN'`, `provider TEXT NOT NULL DEFAULT 'manual'`, `provider_reference TEXT UNIQUE`, `status payment_status`, `metadata JSONB`, `created_at`, `updated_at`).

9. **`analytics`**:
   - `analytics/migrations/20231037_create_analytics_and_events.sql`: Sets up TimescaleDB extension and `analytics.events` hypertable (`id UUID PRIMARY KEY`, `event_type TEXT NOT NULL`, `event_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()`, `data JSONB NOT NULL`).
   - `analytics/migrations/20231039_create_materilised_view.sql`: Continuous aggregate views (`user_signups_daily`, `orders_daily`, `revenue_daily`, `product_views_daily`, `product_metrics_daily`, `inventory_daily`, `delivery_performance_daily`, `payments_daily`, `notifications_daily`).

### 1.3 Database Connection Management & Query Setup
- **Connection Pools**: In all microservices, `sqlx::PgPool::connect(&database_url)` is called during service initialization in `main.rs` (e.g., `user-management/src/main.rs:42`, `product-catalog/src/main.rs:41`, `order-service/src/db.rs:8`).
- **Migrations Run at Startup**: Every service executes `sqlx::migrate!("./migrations").run(&pool).await` on startup.
- **SQL Execution**: Database queries are written using `sqlx::query_as::<_, Model>(...)` or `sqlx::query(...)` runtime strings with positional binding (`$1`, `$2`, etc.). Currently, compile-time `sqlx::query!` macros and `sqlx-data.json` offline metadata files are not present.

---

## 2. Logic Chain

1. **Step 1: Baseline Architecture Assessment**
   - The current system employs microservice-per-database pattern where each microservice owns a separate PostgreSQL database (`users`, `suppliers`, `products`, `orders`, `inventory`, `logistics`, `notifications`, `payments`, `analytics_service`).
   - While tables contain domain entity relationships (e.g., `supplier_id`, `user_id`), there is no explicit `tenant_id` column on any domain table.

2. **Step 2: Shared Multi-Tenancy Requirement Analysis**
   - For Free and Growth tier tenants, data will co-exist in the shared microservice databases.
   - To achieve multi-tenant isolation without data leaks, every domain table in the shared database must be modified to include a mandatory `tenant_id (UUID)` column linked to a core `tenants` reference table via foreign key constraints and indexed via composite indexes `(tenant_id, ...)` to ensure fast tenant-scoped lookups.

3. **Step 3: Row-Level Security (RLS) Policy Design**
   - Relying solely on application-level `WHERE tenant_id = $1` filters leaves the system vulnerable to developer oversights and accidental cross-tenant data leaks.
   - PostgreSQL RLS enforces tenant boundaries at the database engine level. By setting `ALTER TABLE <table_name> ENABLE ROW LEVEL SECURITY;` and `FORCE ROW LEVEL SECURITY;`, PostgreSQL will block any query that attempts to read/write rows not matching `app.current_tenant_id`.

4. **Step 4: Dynamic Connection Routing for Enterprise Tenants**
   - Enterprise tier customers require physical database isolation (dedicated Postgres instances/pools) for regulatory compliance, zero noisy-neighbor risk, and dedicated compute performance.
   - Microservices currently hold a static single `PgPool`. Implementing hybrid multi-tenancy requires a `DynamicPoolManager` abstraction that inspects `TenantContext` on each request, returning either the shared `PgPool` or fetching/instantiating a dedicated `PgPool` mapped to the enterprise tenant's database connection URL.

5. **Step 5: SQLx Compilation & Preparation Strategy**
   - Moving from local development to CI/CD requires `cargo sqlx prepare` metadata generation (`.sqlx/` directory).
   - Once `tenant_id` parameters are added to queries, running `cargo sqlx prepare --workspace` will lock in query types and schema signatures for offline build checks (`SQLX_OFFLINE=true`).

---

## 3. Caveats

1. **Read-Only Scope**: This analysis was conducted under strict read-only constraints. No source files, migrations, or tests were executed or edited during this phase.
2. **Database Engine Differences**: `analytics` uses TimescaleDB hypertables (`analytics.events`). Adding `tenant_id` to hypertable JSON data payloads (`data->>'tenant_id'`) or as a column requires updating continuous aggregate view definitions.
3. **Existing Dev Data / NULL tenant_id**: Initial migrations on live or staging environments require `tenant_id` columns to be nullable during column creation, populated with a default system tenant ID, and then altered to `NOT NULL`.

---

## 4. Conclusion & Implementation Plan

### 4.1 Master Domain Tables & Migration Schema

The following 13 domain tables across shared PostgreSQL databases must be migrated to include `tenant_id`:

| Microservice | Database Table | Primary Key | Proposed Index for Multi-Tenancy |
|---|---|---|---|
| `user-management` | `users` | `id` | `idx_users_tenant_id ON users(tenant_id, id)` |
| `supplier-management` | `suppliers` | `id` | `idx_suppliers_tenant_id ON suppliers(tenant_id, owner_user_id)` |
| `product-catalog` | `products` | `id` | `idx_products_tenant_id ON products(tenant_id, product_id)` |
| `product-catalog` | `product_assets` | `id` | `idx_product_assets_tenant_id ON product_assets(tenant_id, product_id)` |
| `order-service` | `orders` | `id` | `idx_orders_tenant_id ON orders(tenant_id, created_at DESC)` |
| `order-service` | `order_audit_logs` | `id` | `idx_order_audit_logs_tenant_id ON order_audit_logs(tenant_id, order_id)` |
| `inventory-management` | `inventory` | `id` | `idx_inventory_tenant_id ON inventory(tenant_id, product_id)` |
| `inventory-management` | `reservations` | `reservation_id` | `idx_reservations_tenant_id ON reservations(tenant_id, order_id)` |
| `logistics` | `shipments` | `id` | `idx_shipments_tenant_id ON shipments(tenant_id, created_at DESC)` |
| `notifications` | `notifications` | `id` | `idx_notifications_tenant_id ON notifications(tenant_id, created_at DESC)` |
| `notifications` | `notification_devices` | `id` | `idx_notification_devices_tenant_id ON notification_devices(tenant_id, user_id)` |
| `notifications` | `notification_preferences` | `user_id` | `idx_notification_preferences_tenant_id ON notification_preferences(tenant_id, user_id)` |
| `payments` | `payment_intents` | `id` | `idx_payment_intents_tenant_id ON payment_intents(tenant_id, created_at DESC)` |

### 4.2 Central Tenant Control Table (Migration Definition)

```sql
-- Migration: 20260801_create_tenants.sql (in supplier-management or platform system DB)
CREATE TYPE tenant_tier AS ENUM ('free', 'growth', 'enterprise');

CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    tier tenant_tier NOT NULL DEFAULT 'free',
    db_connection_url TEXT NULL,
    max_orders_per_month INTEGER NOT NULL DEFAULT 100,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tenants_slug ON tenants(slug);
CREATE INDEX idx_tenants_tier ON tenants(tier);
```

### 4.3 Standard Microservice Tenant Migration Template

For each microservice database (e.g. `order-service/migrations/20260802_add_tenant_id_and_rls.sql`):

```sql
-- Step 1: Add tenant_id column
ALTER TABLE orders 
ADD COLUMN tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';

-- Step 2: Add composite tenant indexes
CREATE INDEX idx_orders_tenant_lookup ON orders(tenant_id, id);
CREATE INDEX idx_orders_tenant_created ON orders(tenant_id, created_at DESC);

-- Step 3: Enable and Force Row-Level Security
ALTER TABLE orders ENABLE ROW LEVEL SECURITY;
ALTER TABLE orders FORCE ROW LEVEL SECURITY;

-- Step 4: Create RLS Policy
CREATE POLICY orders_tenant_isolation_policy ON orders
    FOR ALL
    USING (tenant_id = NULLIF(current_setting('app.current_tenant_id', true), '')::uuid)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.current_tenant_id', true), '')::uuid);
```

### 4.4 Rust Tenant Context & Transaction RLS Enforcer Pattern

```rust
// platform/src/tenant.rs
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum PricingTier {
    Free,
    Growth,
    Enterprise,
}

#[derive(Debug, Clone)]
pub struct TenantContext {
    pub tenant_id: Uuid,
    pub tier: PricingTier,
    pub db_connection_url: Option<String>,
}

impl TenantContext {
    pub async fn apply_rls<'c, E>(&self, executor: E) -> Result<(), sqlx::Error>
    where
        E: sqlx::Executor<'c, Database = sqlx::Postgres>,
    {
        let query = format!("SET LOCAL app.current_tenant_id = '{}'", self.tenant_id);
        sqlx::query(&query).execute(executor).await?;
        Ok(())
    }
}
```

### 4.5 Dynamic Connection Pool Manager (Enterprise Routing)

```rust
// platform/src/db_router.rs
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use sqlx::PgPool;
use uuid::Uuid;
use crate::tenant::{TenantContext, PricingTier};

#[derive(Clone)]
pub struct DynamicPoolRouter {
    shared_pool: PgPool,
    dedicated_pools: Arc<RwLock<HashMap<Uuid, PgPool>>>,
}

impl DynamicPoolRouter {
    pub fn new(shared_pool: PgPool) -> Self {
        Self {
            shared_pool,
            dedicated_pools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_pool(&self, ctx: &TenantContext) -> Result<PgPool, sqlx::Error> {
        match ctx.tier {
            PricingTier::Enterprise => {
                if let Some(ref db_url) = ctx.db_connection_url {
                    {
                        let pools = self.dedicated_pools.read().await;
                        if let Some(pool) = pools.get(&ctx.tenant_id) {
                            return Ok(pool.clone());
                        }
                    }
                    let mut pools = self.dedicated_pools.write().await;
                    if let Some(pool) = pools.get(&ctx.tenant_id) {
                        return Ok(pool.clone());
                    }
                    let new_pool = PgPool::connect(db_url).await?;
                    pools.insert(ctx.tenant_id, new_pool.clone());
                    Ok(new_pool)
                } else {
                    Ok(self.shared_pool.clone())
                }
            }
            PricingTier::Free | PricingTier::Growth => Ok(self.shared_pool.clone()),
        }
    }
}
```

### 4.6 Cargo SQLx Prepare Workflow Strategy

To maintain offline build capabilities (`SQLX_OFFLINE=true`):
1. **Local Migration Execution**: Spin up local Postgres databases via docker-compose and run `sqlx migrate run`.
2. **Metadata Generation**: Run `cargo sqlx prepare --workspace -- --all-targets` from the repository root.
3. **Version Control**: Commit generated `.sqlx` directory containing query hash definitions.
4. **CI Pipeline Validation**: Execute `cargo check --workspace` with `SQLX_OFFLINE=true`.

---

## 5. Verification Method

To verify the investigation findings and downstream implementation:

1. **Verify Existing Schema Definitions**:
   - Inspect `.sql` files in each service's `migrations/` folder (e.g. `view_file` on `order-service/migrations/20231034_order_service.sql`).
2. **Verify SQL Queries in Codebase**:
   - Inspect repository struct implementations in `db.rs` for each microservice (e.g., `user-management/src/db.rs`, `product-catalog/src/db.rs`, `order-service/src/routes.rs`).
3. **Verification Command Post-Implementation**:
   - Run `cargo check --workspace` to ensure all crates compile without syntax or type errors.
   - Run `cargo test --workspace` to execute co-located unit and integration tests.
4. **RLS Invalidation Condition**:
   - Attempting to query `SELECT * FROM orders` without setting `app.current_tenant_id` session parameter should return 0 rows when RLS is active.
