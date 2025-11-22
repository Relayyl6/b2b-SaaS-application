CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TYPE order_status AS ENUM ('pending', 'confirmed', 'failed', 'cancelled', 'shipped', 'delivered');

CREATE TABLE orders (
    id PRIMARY KEY UUID DEFAULT gen_random_uuid(),
    user_id UUID DEFAULT gen_random_uuid(),
    supplier_id UUID DEFAULT gen_random_uuid(),
    product_id UUID DEFAULT gen_random_uuid(),
    items JSONB NOT NULL,
    status order_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(id)
);

CREATE INDEX IF NOT EXISTS idx_order ON orders(id);

ALTER TABLE orders
ADD COLUMN expires_at TIMESTAMPTZ DEFAULT NOW() + INTERVAL '2 days';

ALTER TABLE orders
ADD COLUMN timestamp TIMESTAMPTZ DEFAULT NOW();