ALTER TABLE products ADD COLUMN sku TEXT;
ALTER TABLE products ADD COLUMN variants JSONB;
ALTER TABLE products ADD COLUMN deleted_at TIMESTAMPTZ NULL;

CREATE UNIQUE INDEX idx_products_supplier_name_unique
ON products(supplier_id, name)
WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX idx_products_supplier_sku_unique
ON products(supplier_id, sku)
WHERE sku IS NOT NULL AND deleted_at IS NULL;

CREATE INDEX idx_products_category_price 
ON products(category, price) 
WHERE deleted_at IS NULL;
