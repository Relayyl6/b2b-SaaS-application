CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    tier VARCHAR(50) NOT NULL DEFAULT 'free',
    stripe_customer_id VARCHAR(255),
    db_connection_url TEXT,
    payment_config JSONB DEFAULT '{}',
    notification_config JSONB DEFAULT '{}',
    feature_flags JSONB DEFAULT '{}',
    webhook_config JSONB DEFAULT '{}',
    rate_limit_override INTEGER,
    metadata_schema JSONB DEFAULT NULL,
    ip_allowlist TEXT[],
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    key_prefix VARCHAR(16) NOT NULL UNIQUE,
    key_hash VARCHAR(64) NOT NULL UNIQUE,
    key_type VARCHAR(8) NOT NULL,
    environment VARCHAR(8) NOT NULL,
    scopes TEXT[] NOT NULL DEFAULT '{}',
    rate_limit_override INTEGER,
    last_used_at TIMESTAMPTZ,
    last_used_ip INET,
    usage_count BIGINT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID
);

CREATE INDEX idx_api_keys_prefix ON api_keys(key_prefix) WHERE is_active = TRUE;
CREATE INDEX idx_api_keys_tenant ON api_keys(tenant_id) WHERE is_active = TRUE;
