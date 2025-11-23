-- pgcrypto
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- drop type and table if they exist (dev only)
DROP TABLE IF EXISTS orders CASCADE;
DROP TYPE IF EXISTS order_status;

-- enum
CREATE TYPE order_status AS ENUM ('pending', 'confirmed', 'failed', 'cancelled', 'shipped', 'delivered');

-- table
CREATE TABLE orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID DEFAULT gen_random_uuid(),
    supplier_id UUID DEFAULT gen_random_uuid(),
    product_id UUID DEFAULT gen_random_uuid(),
    items JSONB NOT NULL,
    qty INT NOT NULL DEFAULT 0,
    status order_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ DEFAULT (NOW() + INTERVAL '2 days'),
    order_timestamp TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_order ON orders(id);
