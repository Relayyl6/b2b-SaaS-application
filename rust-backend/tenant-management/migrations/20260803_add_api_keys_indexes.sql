-- Add composite indexes for lookup performance
CREATE INDEX IF NOT EXISTS idx_api_keys_tenant_env ON api_keys(tenant_id, environment) WHERE is_active = TRUE;
