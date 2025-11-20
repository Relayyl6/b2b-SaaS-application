CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TYPE order_status AS ENUM ('pending', 'shipped', 'delivered');

CREATE TABLE orders (
    id PRIMARY KEY UUID DEFAULT gen_random_uuid(),
    restaurant_id UUID DEFAULT gen_random_uuid(),
    supplier_id UUID DEFAULT gen_random_uuid(),
    items TEXT NOT NULL,
    status order_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(id)
);

CREATE INDEX IF NOT EXISTS idx_order ON orders(id);
