CREATE EXTENSION IF NOT EXISTS "pgcrypto";

DO $$ BEGIN
    CREATE TYPE supplier_status AS ENUM ('pending', 'active', 'suspended', 'rejected');
EXCEPTION WHEN duplicate_object THEN null;
END $$;

CREATE TABLE IF NOT EXISTS suppliers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_user_id UUID NOT NULL,
    legal_name TEXT NOT NULL,
    display_name TEXT NOT NULL,
    tax_id TEXT,
    country TEXT NOT NULL DEFAULT 'NG',
    status supplier_status NOT NULL DEFAULT 'pending',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(owner_user_id, legal_name)
);

CREATE INDEX IF NOT EXISTS idx_suppliers_owner ON suppliers(owner_user_id);
CREATE INDEX IF NOT EXISTS idx_suppliers_status ON suppliers(status);
