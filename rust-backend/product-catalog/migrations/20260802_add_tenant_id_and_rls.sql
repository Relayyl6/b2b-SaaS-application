ALTER TABLE products ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_products_tenant_id ON products(tenant_id, product_id);
ALTER TABLE products ENABLE ROW LEVEL SECURITY;
ALTER TABLE products FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS products_tenant_isolation_policy ON products;
CREATE POLICY products_tenant_isolation_policy ON products
    FOR ALL
    USING (tenant_id = NULLIF(current_setting('app.current_tenant_id', true), '')::uuid)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.current_tenant_id', true), '')::uuid);

ALTER TABLE product_assets ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_product_assets_tenant_id ON product_assets(tenant_id, product_id);
ALTER TABLE product_assets ENABLE ROW LEVEL SECURITY;
ALTER TABLE product_assets FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS product_assets_tenant_isolation_policy ON product_assets;
CREATE POLICY product_assets_tenant_isolation_policy ON product_assets
    FOR ALL
    USING (tenant_id = NULLIF(current_setting('app.current_tenant_id', true), '')::uuid)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.current_tenant_id', true), '')::uuid);
