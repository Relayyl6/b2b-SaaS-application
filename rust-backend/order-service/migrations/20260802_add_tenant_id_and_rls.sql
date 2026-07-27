ALTER TABLE orders ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_orders_tenant_id ON orders(tenant_id, created_at DESC);
ALTER TABLE orders ENABLE ROW LEVEL SECURITY;
ALTER TABLE orders FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS orders_tenant_isolation_policy ON orders;
CREATE POLICY orders_tenant_isolation_policy ON orders
    FOR ALL
    USING (tenant_id = NULLIF(current_setting('app.current_tenant_id', true), '')::uuid)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.current_tenant_id', true), '')::uuid);

ALTER TABLE order_audit_logs ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_order_audit_logs_tenant_id ON order_audit_logs(tenant_id, order_id);
ALTER TABLE order_audit_logs ENABLE ROW LEVEL SECURITY;
ALTER TABLE order_audit_logs FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS order_audit_logs_tenant_isolation_policy ON order_audit_logs;
CREATE POLICY order_audit_logs_tenant_isolation_policy ON order_audit_logs
    FOR ALL
    USING (tenant_id = NULLIF(current_setting('app.current_tenant_id', true), '')::uuid)
    WITH CHECK (tenant_id = NULLIF(current_setting('app.current_tenant_id', true), '')::uuid);
