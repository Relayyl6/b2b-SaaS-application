-- V1__create_products_table.sql
CREATE EXTENSION IF NOT EXISTS "pgcrypto"; -- for gen_random_uuid()

CREATE TABLE products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    supplier_id UUID NOT NULL,
    name TEXT NOT NULL,
    description JSONB,
    category TEXT NOT NULL,
    price NUMERIC(12,2) NOT NULL DEFAULT 0.00,
    unit TEXT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 0,
    available BOOLEAN NOT NULL DEFAULT TRUE,
    low_stock_threshold INTEGER NOT NULL DEFAULT 10,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(product_id, id)
);

CREATE INDEX idx_products_supplier ON products(supplier_id);
CREATE INDEX idx_products_category ON products(category);
CREATE INDEX idx_products_price ON products(price);

-- Convert price column to DOUBLE PRECISION for Rust f64 compatibility
ALTER TABLE products
ALTER COLUMN price TYPE DOUBLE PRECISION USING price::double precision;