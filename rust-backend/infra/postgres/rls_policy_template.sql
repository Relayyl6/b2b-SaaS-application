-- Canonical RLS Policy Template
ALTER TABLE {table_name} ENABLE ROW LEVEL SECURITY;
CREATE POLICY {table_name}_isolation_policy ON {table_name}
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);
