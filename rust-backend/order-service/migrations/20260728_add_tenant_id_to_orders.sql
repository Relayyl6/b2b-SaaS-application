ALTER TABLE orders ADD COLUMN tenant_id UUID DEFAULT gen_random_uuid();

ALTER TABLE orders ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON orders
    FOR ALL
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE INDEX idx_orders_tenant_id ON orders(tenant_id);
