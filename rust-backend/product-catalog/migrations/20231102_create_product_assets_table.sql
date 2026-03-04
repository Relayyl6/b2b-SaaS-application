CREATE TABLE IF NOT EXISTS product_assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(product_id) ON DELETE CASCADE,
    supplier_id UUID NOT NULL,
    provider TEXT NOT NULL DEFAULT 'cloudinary',
    public_id TEXT NOT NULL,
    url TEXT NOT NULL,
    secure_url TEXT NOT NULL,
    width INTEGER,
    height INTEGER,
    bytes BIGINT,
    format TEXT,
    alt_text TEXT,
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (product_id, public_id)
);

CREATE INDEX IF NOT EXISTS idx_product_assets_lookup
    ON product_assets (supplier_id, product_id, created_at DESC);

CREATE UNIQUE INDEX IF NOT EXISTS idx_product_assets_primary_per_product
    ON product_assets(product_id)
    WHERE is_primary = TRUE;
