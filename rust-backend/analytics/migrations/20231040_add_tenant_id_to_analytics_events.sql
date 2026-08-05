-- =====================================
-- Add tenant_id to analytics.events & RLS
-- =====================================

ALTER TABLE analytics.events ADD COLUMN IF NOT EXISTS tenant_id UUID;
CREATE INDEX IF NOT EXISTS idx_analytics_tenant_id ON analytics.events(tenant_id);

ALTER TABLE analytics.events ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation_policy ON analytics.events;
CREATE POLICY tenant_isolation_policy ON analytics.events
    FOR ALL
    USING (tenant_id = NULLIF(current_setting('app.current_tenant_id', true), '')::uuid);
