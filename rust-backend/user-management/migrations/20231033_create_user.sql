CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL,
    password TEXT NOT NULL,
    full_name TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT "user",
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(id)
);

CREATE INDEX IF NOT EXISTS idx_user ON users(id);

CREATE TABLE IF NOT EXISTS revoked_tokens (
    id SERIAL PRIMARY KEY DEFAULT gen_random_uuid(),
    token TEXT NOT NULL,
    revoked_at TIMESTAMP DEFAULT NOW()
);

CREATE TYPE user_role AS ENUM ('user', 'admin', 'supplier');
