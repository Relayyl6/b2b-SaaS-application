-- Webhook configuration table
CREATE TABLE tenant_webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    secret VARCHAR(255) NOT NULL,
    events TEXT[] NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
ALTER TABLE tenant_webhooks ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_webhooks_isolation_policy ON tenant_webhooks
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);
