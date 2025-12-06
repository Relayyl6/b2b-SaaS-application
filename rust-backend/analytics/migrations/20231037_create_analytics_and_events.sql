-- ==========================
-- 0. Setup: extensions & schemas
-- ==========================

CREATE EXTENSION IF NOT EXISTS timescaledb;
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE SCHEMA IF NOT EXISTS raw;
CREATE SCHEMA IF NOT EXISTS analytics;
CREATE SCHEMA IF NOT EXISTS derived;


SET search_path = raw, analytics, derived, public;

-- ==========================
-- 1. Raw events table (single unified event stream)
-- ==========================

CREATE TABLE IF NOT EXISTS analytics.events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type TEXT NOT NULL,
    event_timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOWW(),
    data JSONB NOT NULL
);

-- creating a hypertable for time-series analytics
SELECT create_hypertable(
    'analytics.events',
    'event_timestamp',
    if_not_exists => TRUE,
    migrate_data => TRUE
);

-- indices
CREATE INDEX IF NOT EXISTS idx_events_type on events(event_type);
CREATE INDEX IF NOT EXISTS idx_events_timestamp on events(event_timestamp);