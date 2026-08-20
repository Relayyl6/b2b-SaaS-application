-- Auto-generated foundation from 800+ feature architecture blueprints

CREATE TABLE notification_routes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    event_type VARCHAR(255) NOT NULL,
    provider_config JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON notification_routes (tenant_id, event_type);

CREATE TABLE delivery_predictions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    recipient_id UUID NOT NULL,
    optimal_hour SMALLINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON delivery_predictions (tenant_id, recipient_id);

CREATE TABLE dead_letter_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    payload JSONB NOT NULL,
    error_reason TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON dead_letter_messages (tenant_id);

CREATE TABLE websocket_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    connection_id VARCHAR(100) UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON websocket_sessions (tenant_id);

CREATE TABLE rate_limit_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    channel VARCHAR(50) NOT NULL,
    max_requests_per_hour INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON rate_limit_policies (tenant_id);

CREATE TABLE transactional_emails (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    recipient VARCHAR(255) NOT NULL,
    priority SMALLINT DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON transactional_emails (tenant_id, priority);

CREATE TABLE templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(255) NOT NULL,
    liquid_content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON templates (tenant_id);

CREATE TABLE email_reputation_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    domain VARCHAR(255) NOT NULL,
    bounce_score FLOAT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON email_reputation_logs (domain);

CREATE TABLE notification_state (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    is_read BOOLEAN DEFAULT FALSE,
    synced_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON notification_state (tenant_id, is_read);

CREATE TABLE webhook_endpoints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    url VARCHAR(2048) NOT NULL,
    secret_key VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON webhook_endpoints (tenant_id);

CREATE TABLE device_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    platform VARCHAR(10) NOT NULL,
    token VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON device_tokens (tenant_id);

CREATE TABLE approval_workflows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    current_state JSONB NOT NULL,
    graph_definition JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON approval_workflows (tenant_id);

CREATE TABLE inventory_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(100) NOT NULL,
    customer_email VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON inventory_subscriptions (tenant_id, sku);

CREATE TABLE vendor_communications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    vendor_id UUID NOT NULL,
    message_body TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON vendor_communications (tenant_id, vendor_id);

CREATE TABLE notification_audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    recipient_id UUID NOT NULL,
    content_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON notification_audit_logs (tenant_id, recipient_id);

CREATE TABLE notification_digests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL,
    payloads JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON notification_digests (tenant_id, user_id);

CREATE TABLE communication_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL UNIQUE,
    preferences JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON communication_preferences (tenant_id, user_id);

CREATE TABLE in_app_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL,
    message TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON in_app_notifications (user_id, created_at DESC);

CREATE TABLE user_sequences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL,
    current_step INT DEFAULT 0,
    next_run_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON user_sequences (tenant_id, next_run_at);

CREATE TABLE escalation_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    policy_name VARCHAR(100) NOT NULL,
    steps JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON escalation_policies (tenant_id);

CREATE TABLE notification_traces (
    trace_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    span_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON notification_traces (tenant_id);

CREATE TABLE secure_attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    s3_key VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON secure_attachments (tenant_id);

CREATE TABLE translations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    locale VARCHAR(10) NOT NULL,
    key VARCHAR(100) NOT NULL,
    value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON translations (tenant_id, locale, key);

CREATE TABLE emergency_broadcasts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    message TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON emergency_broadcasts (tenant_id);

CREATE TABLE organization_broadcasts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    org_id UUID NOT NULL,
    message TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON organization_broadcasts (tenant_id, org_id);

CREATE TABLE actionable_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    action_token VARCHAR(255) NOT NULL,
    resolved BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON actionable_messages (action_token);

CREATE TABLE web_push_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL,
    endpoint VARCHAR(512) NOT NULL,
    p256dh VARCHAR(255) NOT NULL,
    auth VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON web_push_subscriptions (tenant_id, user_id);

CREATE TABLE custom_event_schemas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    event_type VARCHAR(100) NOT NULL,
    json_schema JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON custom_event_schemas (tenant_id, event_type);

CREATE TABLE ab_test_allocations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL,
    variant VARCHAR(10) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ab_test_allocations (tenant_id, user_id);

CREATE TABLE firehose_configurations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    aws_kinesis_arn VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON firehose_configurations (tenant_id);

CREATE TABLE notification_dispatches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    recipient_id UUID NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    status VARCHAR(50) DEFAULT 'queued',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON notification_dispatches (tenant_id, status);

CREATE TABLE delivery_fallbacks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL,
    primary_channel VARCHAR(50),
    fallback_channel VARCHAR(50),
    timeout_ms INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON delivery_fallbacks (tenant_id, user_id);

CREATE TABLE webhook_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    target_url TEXT NOT NULL,
    secret_key TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON webhook_subscriptions (tenant_id);

CREATE TABLE anomaly_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    metric_name VARCHAR(100) NOT NULL,
    deviation_score FLOAT NOT NULL,
    alert_dispatched BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON anomaly_alerts (tenant_id, created_at);

CREATE TABLE notification_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    event_type VARCHAR(100) NOT NULL,
    subject_template TEXT NOT NULL,
    body_template TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, event_type)
  );

CREATE TABLE email_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    recipient TEXT NOT NULL,
    message_id TEXT NOT NULL,
    status VARCHAR(50) DEFAULT 'delivered',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON email_logs (tenant_id, recipient);

CREATE TABLE device_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL,
    token TEXT NOT NULL,
    platform VARCHAR(10) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON device_tokens (user_id);

CREATE TABLE sms_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    phone_number TEXT NOT NULL,
    content TEXT NOT NULL,
    provider_response TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON sms_logs (tenant_id, phone_number);

CREATE TABLE rate_limit_configs (
    tenant_id UUID PRIMARY KEY REFERENCES tenants(id),
    email_limit INTEGER NOT NULL,
    sms_limit INTEGER NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE message_receipts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL,
    opened_at TIMESTAMPTZ,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON message_receipts (message_id);

CREATE TABLE approval_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    cart_id UUID NOT NULL,
    manager_id UUID NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON approval_requests (manager_id, status);

CREATE TABLE broadcasts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    segment_id UUID NOT NULL,
    status VARCHAR(20) DEFAULT 'processing',
    total_sent INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE supplier_alert_configs (
    supplier_id UUID PRIMARY KEY,
    digest_frequency VARCHAR(20) NOT NULL,
    events TEXT[] NOT NULL,
    last_digest_sent TIMESTAMPTZ
  );

CREATE TABLE in_app_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    content JSONB NOT NULL,
    read_status BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON in_app_notifications (user_id, read_status);

CREATE TABLE chat_integrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    provider VARCHAR(50) NOT NULL,
    webhook_url TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE sla_policies (
    vendor_id UUID PRIMARY KEY,
    max_fulfillment_hours INTEGER NOT NULL,
    escalation_email TEXT NOT NULL
  );

CREATE TABLE user_engagement_metrics (
    user_id UUID PRIMARY KEY,
    email_open_rate FLOAT DEFAULT 0.0,
    sms_click_rate FLOAT DEFAULT 0.0,
    app_push_open_rate FLOAT DEFAULT 0.0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE order_status_transitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL,
    previous_status VARCHAR(50),
    new_status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON order_status_transitions (order_id);

CREATE TABLE restock_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sku VARCHAR(100) NOT NULL,
    email TEXT NOT NULL,
    fulfilled BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON restock_subscriptions (sku, fulfilled);

CREATE TABLE rfq_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rfq_id UUID NOT NULL,
    author_id UUID NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON rfq_comments (rfq_id);

CREATE TABLE scheduled_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL,
    run_at TIMESTAMPTZ NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON scheduled_notifications (run_at, status);

CREATE TABLE user_time_preferences (
    user_id UUID PRIMARY KEY,
    timezone VARCHAR(50) NOT NULL,
    quiet_hours_start TIME NOT NULL,
    quiet_hours_end TIME NOT NULL
  );

CREATE TABLE notification_retries (
    dispatch_id UUID PRIMARY KEY,
    attempt_count INTEGER DEFAULT 0,
    next_retry_at TIMESTAMPTZ,
    last_error TEXT
  );

CREATE TABLE localized_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    locale VARCHAR(10) NOT NULL,
    translation_file TEXT NOT NULL,
    UNIQUE(tenant_id, locale)
  );

CREATE TABLE user_devices (
    user_id UUID NOT NULL,
    device_fingerprint TEXT NOT NULL,
    last_ip INET NOT NULL,
    last_login TIMESTAMPTZ NOT NULL,
    PRIMARY KEY(user_id, device_fingerprint)
  );

CREATE TABLE tenant_smtp_configs (
    tenant_id UUID PRIMARY KEY REFERENCES tenants(id),
    host TEXT NOT NULL,
    port INTEGER NOT NULL,
    username TEXT NOT NULL,
    password_encrypted TEXT NOT NULL
  );

CREATE TABLE webhook_redaction_rules (
    tenant_id UUID PRIMARY KEY REFERENCES tenants(id),
    redact_keys TEXT[] NOT NULL,
    mask_char VARCHAR(1) DEFAULT '*'
  );

CREATE TABLE user_opt_outs (
    email TEXT NOT NULL,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    category VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY(email, tenant_id, category)
  );

CREATE TABLE message_clicks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL,
    target_url TEXT NOT NULL,
    clicked_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON message_clicks (message_id);

CREATE TABLE siem_configs (
    tenant_id UUID PRIMARY KEY REFERENCES tenants(id),
    endpoint_url TEXT NOT NULL,
    auth_token TEXT NOT NULL
  );

