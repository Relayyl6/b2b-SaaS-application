CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    supplier_id UUID DEFAULT gen_random_uuid(),
    product_id UUID DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    price NUMERIC(12,2) NOT NULL DEFAULT 0.00,
    description TEXT,
    category TEXT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 0,
    low_stock_threshold INTEGER NOT NULL DEFAULT 10,
    unit TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(product_id, id)
);

CREATE INDEX IF NOT EXISTS idx_inventory_supplier ON inventory(supplier_id);

-- Convert price column to DOUBLE PRECISION for Rust f64 compatibility
ALTER TABLE inventory
ALTER COLUMN price TYPE DOUBLE PRECISION USING price::double precision;

ALTER TABLE inventory
ADD COLUMN available BOOLEAN NOT NULL DEFAULT TRUE;

ALTER TABLE inventory 
ADD COLUMN IF NOTE EXISTS reserved INTEGER NOT NULL DEFAULT 0;

CREATE TABLE IF NOT EXISTS reservations (
    reservation_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL UNIQUE,
    product_id UUID NOT NULL,
    qty INT NOT NULL,
    expires_at timestamptz,
    created_at timestamptz DEFAULT now(),
    released BOOLEAN NOT NULL DEFAULT FALSE
);