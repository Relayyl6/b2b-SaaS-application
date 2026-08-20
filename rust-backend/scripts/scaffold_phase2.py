import os

base_path = r"c:\Users\USER\Documents\Previous\E-commerce\b2b-SaaS-application\rust-backend"

# --- 4. product-catalog ---
prod_cat_src = os.path.join(base_path, "product-catalog", "src")
prod_cat_mig = os.path.join(base_path, "product-catalog", "migrations")

with open(os.path.join(prod_cat_src, "search.rs"), "w") as f:
    f.write("// Search and pgvector query builder stub\n")
with open(os.path.join(prod_cat_src, "errors.rs"), "w") as f:
    f.write("// Product Catalog specific errors\n")
with open(os.path.join(prod_cat_src, "events.rs"), "w") as f:
    f.write("// Product Catalog events\n")

with open(os.path.join(prod_cat_mig, "20260803_add_vector_embeddings.sql"), "w") as f:
    f.write("""-- Add pgvector embeddings for semantic search
CREATE EXTENSION IF NOT EXISTS vector;
ALTER TABLE products ADD COLUMN IF NOT EXISTS embedding vector(1536);
""")

# --- 5. inventory-management ---
inv_mgt_src = os.path.join(base_path, "inventory-management", "src")
inv_mgt_mig = os.path.join(base_path, "inventory-management", "migrations")

with open(os.path.join(inv_mgt_src, "errors.rs"), "w") as f:
    f.write("// Inventory Management specific errors\n")
with open(os.path.join(inv_mgt_src, "events.rs"), "w") as f:
    f.write("// Inventory Management events (extracted from redis_sub/events.rs)\n")

with open(os.path.join(inv_mgt_mig, "20260803_create_reservations.sql"), "w") as f:
    f.write("""-- Stock reservations table
CREATE TABLE inventory_reservations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL,
    quantity INTEGER NOT NULL,
    reserved_until TIMESTAMPTZ NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active'
);
""")

# --- 6. order-service ---
order_svc_src = os.path.join(base_path, "order-service", "src")
order_svc_mig = os.path.join(base_path, "order-service", "migrations")

with open(os.path.join(order_svc_src, "errors.rs"), "w") as f:
    f.write("// Order Service specific errors\n")

with open(os.path.join(order_svc_mig, "20260803_create_quotes.sql"), "w") as f:
    f.write("""-- B2B Quote Approval workflow tables
CREATE TABLE b2b_quotes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    status VARCHAR(50) NOT NULL,
    total_amount DECIMAL NOT NULL
);
""")
with open(os.path.join(order_svc_mig, "20260804_create_saga_log.sql"), "w") as f:
    f.write("""-- Distributed saga state machine log table
CREATE TABLE saga_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL,
    state VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
""")

# --- 7. payments ---
pay_src = os.path.join(base_path, "payments", "src")
pay_mig = os.path.join(base_path, "payments", "migrations")

with open(os.path.join(pay_src, "errors.rs"), "w") as f:
    f.write("// Payments specific errors\n")
with open(os.path.join(pay_src, "events.rs"), "w") as f:
    f.write("// Payments events\n")
with open(os.path.join(pay_src, "ledger.rs"), "w") as f:
    f.write("// Double-entry ledger logic stub\n")
os.makedirs(os.path.join(pay_src, "worker"), exist_ok=True)
with open(os.path.join(pay_src, "worker", "mod.rs"), "w") as f:
    f.write("pub mod webhook_processor;\n")
with open(os.path.join(pay_src, "worker", "webhook_processor.rs"), "w") as f:
    f.write("// Webhook background processor\n")


with open(os.path.join(pay_mig, "20260803_create_ledger.sql"), "w") as f:
    f.write("""-- Double-entry ledger tables
CREATE TABLE ledger_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    balance DECIMAL NOT NULL DEFAULT 0.0
);
CREATE TABLE ledger_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES ledger_accounts(id),
    amount DECIMAL NOT NULL,
    direction VARCHAR(10) NOT NULL
);
""")
with open(os.path.join(pay_mig, "20260804_create_escrow.sql"), "w") as f:
    f.write("""-- Escrow holding account tables
CREATE TABLE escrow_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_id UUID NOT NULL,
    amount DECIMAL NOT NULL,
    status VARCHAR(50) NOT NULL
);
""")
with open(os.path.join(pay_mig, "20260805_create_invoices.sql"), "w") as f:
    f.write("""-- B2B Invoice tables
CREATE TABLE b2b_invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    status VARCHAR(50) NOT NULL,
    due_date TIMESTAMPTZ NOT NULL
);
""")

print("Phase 2 Commerce scaffolded successfully.")
