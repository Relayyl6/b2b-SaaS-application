-- ============================================================
-- CaaS Platform — Shared DB Initialization
-- Runs once on first docker compose up
-- ============================================================

-- Enable UUID generation
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================================
-- SET UP ROW-LEVEL SECURITY INFRASTRUCTURE
-- ============================================================

-- GUC variable for tenant context propagation
-- Services call: SET LOCAL app.current_tenant_id = '...'
ALTER DATABASE commerce_shared SET app.current_tenant_id = '';

-- App role (services connect as this — cannot bypass RLS)
DO $$
BEGIN
  IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'commerce_app') THEN
    CREATE ROLE commerce_app NOINHERIT LOGIN PASSWORD 'commerce_app_pass';
  END IF;
END $$;

-- Grant permissions to app role
GRANT CONNECT ON DATABASE commerce_shared TO commerce_app;
GRANT USAGE ON SCHEMA public TO commerce_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO commerce_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO commerce_app;

-- ============================================================
-- SAMPLE SEED DATA (for tests)
-- ============================================================

-- Insert a seed tenant so tests can reference a real tenant_id
-- Tests that use this should reference: tenant_id = '00000000-0000-0000-0000-000000000001'
INSERT INTO tenants (id, name, slug, tier)
VALUES 
  ('00000000-0000-0000-0000-000000000001', 'Test Tenant A', 'test-tenant-a', 'growth'),
  ('00000000-0000-0000-0000-000000000002', 'Test Tenant B', 'test-tenant-b', 'free')
ON CONFLICT (id) DO NOTHING;
