-- Auto-generated foundation from 800+ feature architecture blueprints

CREATE TABLE seller_kyb_applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    tax_id_hash VARCHAR(255) NOT NULL,
    kyb_status VARCHAR(50) NOT NULL,
    document_urls JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON seller_kyb_applications (tenant_id, seller_id);

CREATE TABLE revenue_splits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL REFERENCES orders(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    gross_amount_cents BIGINT NOT NULL,
    commission_cents BIGINT NOT NULL,
    net_amount_cents BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON revenue_splits (order_id, seller_id);

CREATE TABLE payouts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    amount_cents BIGINT NOT NULL,
    currency VARCHAR(3) NOT NULL,
    provider VARCHAR(50) NOT NULL,
    provider_payout_id VARCHAR(255),
    status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON payouts (seller_id, status);

CREATE TABLE commission_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID REFERENCES sellers(id),
    category_id UUID REFERENCES categories(id),
    rate_percentage NUMERIC(5,4) NOT NULL,
    condition_expression TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON commission_rules (tenant_id, seller_id);

CREATE TABLE escrow_holds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL REFERENCES orders(id),
    amount_cents BIGINT NOT NULL,
    hold_until TIMESTAMPTZ NOT NULL,
    release_condition VARCHAR(100),
    status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON escrow_holds (status, hold_until);

CREATE TABLE seller_daily_aggregates (
    seller_id UUID NOT NULL REFERENCES sellers(id),
    date DATE NOT NULL,
    gmv_cents BIGINT NOT NULL DEFAULT 0,
    order_count INT NOT NULL DEFAULT 0,
    return_count INT NOT NULL DEFAULT 0,
    PRIMARY KEY (seller_id, date)
);

CREATE TABLE order_disputes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL REFERENCES orders(id),
    raised_by_id UUID NOT NULL,
    reason VARCHAR(100) NOT NULL,
    resolution_status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON order_disputes (tenant_id, resolution_status);

CREATE TABLE product_moderation_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    product_id UUID NOT NULL REFERENCES products(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    status VARCHAR(50) NOT NULL DEFAULT "pending",
    rejection_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON product_moderation_queue (tenant_id, status);

CREATE TABLE seller_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    buyer_id UUID NOT NULL,
    rating SMALLINT CHECK (rating >= 1 AND rating <= 5),
    comment TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON seller_reviews (seller_id);

CREATE TABLE search_boost_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_tier VARCHAR(50) NOT NULL,
    boost_multiplier NUMERIC(4,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE category_commission_rates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    category_id UUID NOT NULL REFERENCES categories(id),
    take_rate_pct NUMERIC(5,4) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX ON category_commission_rates(category_id);

CREATE TABLE seller_scorecards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    period_start DATE NOT NULL,
    fulfillment_score NUMERIC(5,4),
    defect_rate NUMERIC(5,4),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE tax_remittances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL REFERENCES orders(id),
    jurisdiction VARCHAR(100) NOT NULL,
    tax_amount_cents BIGINT NOT NULL,
    remitted BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE inventory_display_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    display_mode VARCHAR(20) NOT NULL, -- exact, threshold, boolean
    threshold_qty INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE order_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    buyer_id UUID NOT NULL,
    total_cents BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
-- orders table has parent_group_id

CREATE TABLE seller_payout_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    interval VARCHAR(20) NOT NULL, -- daily, weekly, monthly
    anchor_day INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE chargeback_liabilities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    dispute_id VARCHAR(255) NOT NULL,
    liable_party VARCHAR(50) NOT NULL, -- operator, seller
    amount_cents BIGINT NOT NULL,
    deducted_from_payout UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE seller_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    tier_id VARCHAR(50) NOT NULL,
    valid_until TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE sponsored_listings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    product_id UUID NOT NULL REFERENCES products(id),
    cpc_bid_cents INT NOT NULL,
    budget_remaining_cents BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE reconciliation_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    period VARCHAR(10) NOT NULL,
    s3_key VARCHAR(255) NOT NULL,
    discrepancy_cents BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE mkt_feature_21 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON mkt_feature_21 (tenant_id, seller_id);

CREATE TABLE mkt_feature_22 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON mkt_feature_22 (tenant_id, seller_id);

CREATE TABLE mkt_feature_23 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON mkt_feature_23 (tenant_id, seller_id);

CREATE TABLE mkt_feature_24 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON mkt_feature_24 (tenant_id, seller_id);

CREATE TABLE mkt_feature_25 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON mkt_feature_25 (tenant_id, seller_id);

CREATE TABLE mkt_feature_26 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON mkt_feature_26 (tenant_id, seller_id);

CREATE TABLE mkt_feature_27 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON mkt_feature_27 (tenant_id, seller_id);

CREATE TABLE mkt_feature_28 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON mkt_feature_28 (tenant_id, seller_id);

CREATE TABLE mkt_feature_29 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON mkt_feature_29 (tenant_id, seller_id);

CREATE TABLE mkt_feature_30 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON mkt_feature_30 (tenant_id, seller_id);

CREATE TABLE mkt_feature_31 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON mkt_feature_31 (tenant_id, seller_id);

CREATE TABLE mkt_feature_32 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON mkt_feature_32 (tenant_id, seller_id);

CREATE TABLE mkt_feature_33 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON mkt_feature_33 (tenant_id, seller_id);

CREATE TABLE mkt_feature_34 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON mkt_feature_34 (tenant_id, seller_id);

CREATE TABLE mkt_feature_35 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON mkt_feature_35 (tenant_id, seller_id);

CREATE TABLE mkt_feature_36 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON mkt_feature_36 (tenant_id, seller_id);

CREATE TABLE mkt_feature_37 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON mkt_feature_37 (tenant_id, seller_id);

CREATE TABLE mkt_feature_38 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON mkt_feature_38 (tenant_id, seller_id);

CREATE TABLE mkt_feature_39 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON mkt_feature_39 (tenant_id, seller_id);

CREATE TABLE mkt_feature_40 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON mkt_feature_40 (tenant_id, seller_id);

CREATE TABLE mkt_feature_41 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON mkt_feature_41 (tenant_id, seller_id);

CREATE TABLE mkt_feature_42 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON mkt_feature_42 (tenant_id, seller_id);

CREATE TABLE mkt_feature_43 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    seller_id UUID NOT NULL REFERENCES sellers(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX ON mkt_feature_43 (tenant_id, seller_id);

CREATE TABLE marketplace_vendors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    company_name VARCHAR(255) NOT NULL,
    tax_id VARCHAR(50),
    kyc_status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON marketplace_vendors (tenant_id, kyc_status);

CREATE TABLE cart_splits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_cart_id UUID NOT NULL,
    vendor_id UUID NOT NULL REFERENCES marketplace_vendors(id),
    subtotal DECIMAL(10, 2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON cart_splits (parent_cart_id);

CREATE TABLE vendor_commission_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vendor_id UUID NOT NULL REFERENCES marketplace_vendors(id),
    rule_expression TEXT NOT NULL, -- e.g., 'total * 0.15'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON vendor_commission_rules (vendor_id);

CREATE TABLE vendor_inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    vendor_id UUID NOT NULL REFERENCES marketplace_vendors(id),
    sku VARCHAR(100) NOT NULL,
    quantity_on_hand INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (vendor_id, sku)
  );

CREATE TABLE vendor_risk_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vendor_id UUID NOT NULL REFERENCES marketplace_vendors(id),
    risk_score FLOAT NOT NULL,
    last_evaluated TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON vendor_risk_profiles (risk_score);

CREATE TABLE product_kits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    kit_sku VARCHAR(100) NOT NULL UNIQUE
  );
  CREATE TABLE kit_components (
    kit_id UUID NOT NULL REFERENCES product_kits(id),
    vendor_id UUID NOT NULL REFERENCES marketplace_vendors(id),
    component_sku VARCHAR(100) NOT NULL,
    qty INT NOT NULL
  );

CREATE TABLE search_sync_ledger (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vendor_id UUID NOT NULL,
    last_sync_seq BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE vendor_shipping_credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vendor_id UUID NOT NULL REFERENCES marketplace_vendors(id),
    carrier VARCHAR(50) NOT NULL,
    api_key_encrypted BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE vendor_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vendor_id UUID NOT NULL REFERENCES marketplace_vendors(id),
    role_name VARCHAR(50) NOT NULL,
    permissions JSONB NOT NULL
  );
  CREATE INDEX ON vendor_roles USING GIN (permissions);

CREATE TABLE vendor_tax_nexus (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vendor_id UUID NOT NULL REFERENCES marketplace_vendors(id),
    state_code VARCHAR(2) NOT NULL,
    is_mor_exempt BOOLEAN DEFAULT FALSE,
    UNIQUE (vendor_id, state_code)
  );

CREATE TABLE marketplace_disputes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL,
    ai_confidence FLOAT,
    resolution_action VARCHAR(50),
    status VARCHAR(20) DEFAULT 'open',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE vendor_webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vendor_id UUID NOT NULL REFERENCES marketplace_vendors(id),
    event_type VARCHAR(50) NOT NULL,
    target_url TEXT NOT NULL,
    failed_attempts INT DEFAULT 0
  );

CREATE TABLE vendor_import_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vendor_id UUID NOT NULL REFERENCES marketplace_vendors(id),
    total_rows INT,
    processed_rows INT DEFAULT 0,
    status VARCHAR(20) DEFAULT 'running'
  );

CREATE TABLE vendor_rmas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL,
    vendor_id UUID NOT NULL,
    status VARCHAR(20) DEFAULT 'pending_return',
    refund_amount DECIMAL(10, 2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE vendor_pricing_tiers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vendor_id UUID NOT NULL,
    sku VARCHAR(100) NOT NULL,
    buyer_group_id UUID,
    min_qty INT NOT NULL,
    price DECIMAL(10, 2) NOT NULL
  );
  CREATE INDEX ON vendor_pricing_tiers (vendor_id, sku, min_qty);

CREATE TABLE vendor_ledger_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vendor_id UUID NOT NULL REFERENCES marketplace_vendors(id),
    transaction_type VARCHAR(20) NOT NULL, -- sale, refund, fee, payout
    amount DECIMAL(10, 2) NOT NULL,
    reference_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON vendor_ledger_entries (vendor_id, created_at);

CREATE TABLE marketplace_rfqs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    buyer_id UUID NOT NULL,
    details TEXT NOT NULL,
    status VARCHAR(20) DEFAULT 'open',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE TABLE rfq_bids (
    rfq_id UUID NOT NULL REFERENCES marketplace_rfqs(id),
    vendor_id UUID NOT NULL REFERENCES marketplace_vendors(id),
    bid_amount DECIMAL(10, 2) NOT NULL
  );

CREATE TABLE vendor_threads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    buyer_id UUID NOT NULL,
    vendor_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE TABLE thread_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    thread_id UUID NOT NULL REFERENCES vendor_threads(id),
    sender_type VARCHAR(10) NOT NULL,
    content_encrypted BYTEA NOT NULL
  );

CREATE TABLE vendor_embeddings (
    vendor_id UUID PRIMARY KEY REFERENCES marketplace_vendors(id),
    feature_vector vector(768), -- Uses pgvector extension
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE vendor_slas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vendor_id UUID NOT NULL REFERENCES marketplace_vendors(id),
    max_fulfillment_hours INT NOT NULL,
    orders_breached INT DEFAULT 0
  );

CREATE TABLE vendor_penalties (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vendor_id UUID NOT NULL REFERENCES marketplace_vendors(id),
    amount DECIMAL(10, 2) NOT NULL,
    reason VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE marketplace_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    buyer_id UUID NOT NULL,
    stripe_sub_id VARCHAR(100) NOT NULL,
    next_billing_date TIMESTAMPTZ NOT NULL
  );

CREATE TABLE vendor_storefronts (
    vendor_id UUID PRIMARY KEY REFERENCES marketplace_vendors(id),
    theme_config JSONB NOT NULL,
    custom_slug VARCHAR(100) UNIQUE NOT NULL
  );
  CREATE INDEX ON vendor_storefronts (custom_slug);

CREATE TABLE vendor_webhook_secrets (
    vendor_id UUID PRIMARY KEY REFERENCES marketplace_vendors(id),
    hmac_secret BYTEA NOT NULL
  );

CREATE TABLE vendor_audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vendor_id UUID NOT NULL REFERENCES marketplace_vendors(id),
    actor_id UUID NOT NULL,
    action_type VARCHAR(50) NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  -- Partitioned by month for performance

