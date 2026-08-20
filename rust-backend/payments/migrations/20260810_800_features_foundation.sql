-- Auto-generated foundation from 800+ feature architecture blueprints

CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON transactions (tenant_id, created_at DESC);

CREATE TABLE idempotency_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON idempotency_keys (tenant_id, created_at DESC);

CREATE TABLE usage_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON usage_events (tenant_id, created_at DESC);

CREATE TABLE revenue_splits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON revenue_splits (tenant_id, created_at DESC);

CREATE TABLE monetary_values (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON monetary_values (tenant_id, created_at DESC);

CREATE TABLE financial_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON financial_events (tenant_id, created_at DESC);

CREATE TABLE outbox_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON outbox_messages (tenant_id, created_at DESC);

CREATE TABLE tax_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON tax_records (tenant_id, created_at DESC);

CREATE TABLE wallet_balances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON wallet_balances (tenant_id, created_at DESC);

CREATE TABLE fx_rates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON fx_rates (tenant_id, created_at DESC);

CREATE TABLE virtual_ibans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON virtual_ibans (tenant_id, created_at DESC);

CREATE TABLE pricing_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON pricing_plans (tenant_id, created_at DESC);

CREATE TABLE fraud_evaluations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON fraud_evaluations (tenant_id, created_at DESC);

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON audit_logs (tenant_id, created_at DESC);

CREATE TABLE gateway_routes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON gateway_routes (tenant_id, created_at DESC);

CREATE TABLE disputes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON disputes (tenant_id, created_at DESC);

CREATE TABLE dunning_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON dunning_schedules (tenant_id, created_at DESC);

CREATE TABLE escrow_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON escrow_accounts (tenant_id, created_at DESC);

CREATE TABLE custom_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON custom_rules (tenant_id, created_at DESC);

CREATE TABLE reconciliation_exceptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON reconciliation_exceptions (tenant_id, created_at DESC);

CREATE TABLE subscription_changes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON subscription_changes (tenant_id, created_at DESC);

CREATE TABLE factoring_offers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON factoring_offers (tenant_id, created_at DESC);

CREATE TABLE virtual_cards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON virtual_cards (tenant_id, created_at DESC);

CREATE TABLE tax_nexuses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON tax_nexuses (tenant_id, created_at DESC);

CREATE TABLE chargeback_sagas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON chargeback_sagas (tenant_id, created_at DESC);

CREATE TABLE invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON invoices (tenant_id, created_at DESC);

CREATE TABLE revenue_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON revenue_schedules (tenant_id, created_at DESC);

CREATE TABLE treasury_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON treasury_reports (tenant_id, created_at DESC);

CREATE TABLE bank_mandates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON bank_mandates (tenant_id, created_at DESC);

CREATE TABLE order_installments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON order_installments (tenant_id, created_at DESC);

CREATE TABLE hybrid_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON hybrid_subscriptions (tenant_id, created_at DESC);

CREATE TABLE crypto_payouts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON crypto_payouts (tenant_id, created_at DESC);

CREATE TABLE ledger_hashes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ledger_hashes (tenant_id, created_at DESC);

CREATE TABLE rwa_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON rwa_tokens (tenant_id, created_at DESC);

CREATE TABLE liquidity_pools (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON liquidity_pools (tenant_id, created_at DESC);

CREATE TABLE trading_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON trading_orders (tenant_id, created_at DESC);

CREATE TABLE yield_strategies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON yield_strategies (tenant_id, created_at DESC);

CREATE TABLE insurance_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON insurance_policies (tenant_id, created_at DESC);

CREATE TABLE zkp_verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON zkp_verifications (tenant_id, created_at DESC);

CREATE TABLE mpc_signatures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    amount BIGINT NOT NULL, -- stored in cents
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON mpc_signatures (tenant_id, created_at DESC);

CREATE TABLE ledger_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID NOT NULL,
    amount DECIMAL(19, 4) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ledger_entries (tenant_id, account_id);

-- ClickHouse Table
  CREATE TABLE usage_events (
    tenant_id UUID,
    subscription_id UUID,
    metric_id String,
    value UInt64,
    event_time DateTime
  ) ENGINE = MergeTree()
  ORDER BY (tenant_id, subscription_id, event_time);

CREATE TABLE dunning_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    invoice_id UUID NOT NULL,
    next_retry_at TIMESTAMPTZ NOT NULL,
    ai_confidence DECIMAL(3, 2),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON dunning_schedules (next_retry_at);

CREATE TABLE tax_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    transaction_id UUID NOT NULL,
    tax_amount DECIMAL(19, 4) NOT NULL,
    jurisdiction VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE churn_predictions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    subscription_id UUID NOT NULL,
    risk_score DECIMAL(3,2) NOT NULL,
    factors JSONB NOT NULL,
    evaluated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON churn_predictions (subscription_id, evaluated_at DESC);

CREATE TABLE payment_intents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL,
    total_amount DECIMAL(19, 4) NOT NULL,
    status VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE TABLE payment_splits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    intent_id UUID NOT NULL REFERENCES payment_intents(id),
    method VARCHAR(50) NOT NULL,
    amount DECIMAL(19, 4) NOT NULL,
    status VARCHAR(20) NOT NULL
  );

CREATE TABLE reconciled_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    invoice_id UUID NOT NULL,
    statement_line_id UUID NOT NULL,
    match_score DECIMAL(3, 2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE escrows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    buyer_id UUID NOT NULL,
    seller_id UUID NOT NULL,
    amount DECIMAL(19, 4) NOT NULL,
    status VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE early_payment_terms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    invoice_id UUID NOT NULL,
    discount_percentage DECIMAL(5, 4) NOT NULL,
    valid_until TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE credit_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    entity_id UUID NOT NULL,
    parent_entity_id UUID,
    max_limit DECIMAL(19, 4) NOT NULL,
    current_utilization DECIMAL(19, 4) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE fraud_evaluations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL,
    risk_score DECIMAL(3, 2) NOT NULL,
    decision VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE revrec_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    invoice_line_id UUID NOT NULL,
    recognition_date DATE NOT NULL,
    amount DECIMAL(19, 4) NOT NULL,
    status VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE wallets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    company_id UUID NOT NULL,
    balance DECIMAL(19, 4) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE payout_routes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    charge_id UUID NOT NULL,
    vendor_id UUID NOT NULL,
    amount DECIMAL(19, 4) NOT NULL,
    status VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE subscription_changes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    subscription_id UUID NOT NULL,
    old_plan_id VARCHAR(50),
    new_plan_id VARCHAR(50),
    net_charge DECIMAL(19, 4) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    table_name VARCHAR(50) NOT NULL,
    record_id UUID NOT NULL,
    previous_hash VARCHAR(64) NOT NULL,
    current_hash VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE fx_locks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    base_currency VARCHAR(3) NOT NULL,
    target_currency VARCHAR(3) NOT NULL,
    locked_rate DECIMAL(10, 6) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE late_fees (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    invoice_id UUID NOT NULL,
    fee_amount DECIMAL(19, 4) NOT NULL,
    calculation_rule VARCHAR(100) NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE treasury_sweeps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    vendor_id UUID NOT NULL,
    amount DECIMAL(19, 4) NOT NULL,
    status VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE contract_pricing (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    customer_id UUID NOT NULL,
    sku VARCHAR(50) NOT NULL,
    tier_start INT NOT NULL,
    tier_end INT,
    price DECIMAL(19, 4) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON contract_pricing (customer_id, sku);

CREATE TABLE usage_anomalies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    subscription_id UUID NOT NULL,
    metric_id VARCHAR(50) NOT NULL,
    deviation_score DECIMAL(5, 2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE instant_payouts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    vendor_id UUID NOT NULL,
    amount DECIMAL(19, 4) NOT NULL,
    network VARCHAR(20) NOT NULL,
    cleared_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE consolidated_invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    parent_company_id UUID NOT NULL,
    total_amount DECIMAL(19, 4) NOT NULL,
    period VARCHAR(10) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE dispute_evidence (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    dispute_id UUID NOT NULL,
    evidence_type VARCHAR(50) NOT NULL,
    s3_path VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE spend_approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    employee_id UUID NOT NULL,
    approver_id UUID NOT NULL,
    amount DECIMAL(19, 4) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

