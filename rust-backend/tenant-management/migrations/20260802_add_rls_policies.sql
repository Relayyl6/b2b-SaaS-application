-- Add RLS policies for tenants
ALTER TABLE api_keys ENABLE ROW LEVEL SECURITY;
CREATE POLICY api_keys_tenant_isolation_policy ON api_keys
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);
