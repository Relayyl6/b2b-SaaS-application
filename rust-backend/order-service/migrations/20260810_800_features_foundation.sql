-- Auto-generated foundation from 800+ feature architecture blueprints

CREATE TABLE rfq_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID, status VARCHAR(50), total_value BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON rfq_requests (tenant_id, account_id);

CREATE TABLE po_approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    po_id UUID, approver_id UUID, status VARCHAR(20),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON po_approvals (tenant_id, po_id);

CREATE TABLE price_books (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID, sku VARCHAR(50), price_cents BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON price_books (tenant_id, account_id);

CREATE TABLE edi_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sender_id UUID, doc_type VARCHAR(10), payload JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON edi_documents (tenant_id, sender_id);

CREATE TABLE blanket_pos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID, budget_cents BIGINT, used_cents BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON blanket_pos (tenant_id, account_id);

CREATE TABLE credit_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID, limit_cents BIGINT, balance_cents BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON credit_accounts (tenant_id, account_id);

CREATE TABLE negotiation_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    rfq_id UUID, offer_cents BIGINT, side VARCHAR(10),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON negotiation_logs (tenant_id, rfq_id);

CREATE TABLE invoice_matches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    po_id UUID, invoice_id UUID, match_status VARCHAR(20),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON invoice_matches (tenant_id, po_id);

CREATE TABLE drop_shipments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID, vendor_id UUID, tracking VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON drop_shipments (tenant_id, vendor_id);

CREATE TABLE vmi_inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    location_id UUID, sku VARCHAR(50), qty INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON vmi_inventory (tenant_id, location_id);

CREATE TABLE back_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID, sku VARCHAR(50), promise_date DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON back_orders (tenant_id, order_id);

CREATE TABLE catalog_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID, category_id UUID, is_visible BOOLEAN,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON catalog_rules (tenant_id, account_id);

CREATE TABLE product_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(50), valid_options JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON product_configs (tenant_id, sku);

CREATE TABLE split_allocations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID, fulfillment_node UUID, items JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON split_allocations (tenant_id, order_id);

CREATE TABLE multi_destinations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID, address_id UUID, items JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON multi_destinations (tenant_id, order_id);

CREATE TABLE account_hierarchies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    child_id UUID, parent_id UUID, depth INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON account_hierarchies (tenant_id, parent_id);

CREATE TABLE delegated_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID, max_spend_cents BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON delegated_roles (tenant_id, user_id);

CREATE TABLE requisitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    creator_id UUID, status VARCHAR(20), items JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON requisitions (tenant_id, creator_id);

CREATE TABLE supplier_asns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    vendor_id UUID, po_id UUID, eta TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON supplier_asns (tenant_id, vendor_id);

CREATE TABLE order_amendments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID, previous_state JSONB, new_state JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON order_amendments (tenant_id, order_id);

CREATE TABLE rma_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID, reason VARCHAR(100), status VARCHAR(20),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON rma_requests (tenant_id, order_id);

CREATE TABLE warranty_claims (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    serial_num VARCHAR(100), claim_date DATE, status VARCHAR(20),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON warranty_claims (tenant_id, serial_num);

CREATE TABLE substitutions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    out_of_stock_sku VARCHAR(50), replacement_sku VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON substitutions (tenant_id, out_of_stock_sku);

CREATE TABLE reorder_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(50), min_qty INT, max_qty INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON reorder_policies (tenant_id, sku);

CREATE TABLE routing_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    zip_prefix VARCHAR(10), node_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON routing_rules (tenant_id, zip_prefix);

CREATE TABLE freight_quotes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID, carrier VARCHAR(50), cost_cents BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON freight_quotes (tenant_id, order_id);

CREATE TABLE tax_certs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID, cert_url VARCHAR(255), expires_at DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON tax_certs (tenant_id, account_id);

CREATE TABLE credit_holds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID, order_id UUID, amount_cents BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON credit_holds (tenant_id, account_id);

CREATE TABLE dock_appointments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    warehouse_id UUID, asn_id UUID, slot TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON dock_appointments (tenant_id, warehouse_id);

CREATE TABLE bom_components (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    parent_sku VARCHAR(50), child_sku VARCHAR(50), qty INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON bom_components (tenant_id, parent_sku);

CREATE TABLE subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID, sku VARCHAR(50), cron_expr VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON subscriptions (tenant_id, account_id);

CREATE TABLE vendor_bids (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    rfq_id UUID, vendor_id UUID, amount_cents BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON vendor_bids (tenant_id, rfq_id);

CREATE TABLE payment_terms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    invoice_id UUID, due_date DATE, discount_pct DECIMAL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON payment_terms (tenant_id, invoice_id);

CREATE TABLE consigned_stock (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID, sku VARCHAR(50), qty INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON consigned_stock (tenant_id, account_id);

CREATE TABLE kitting_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID, kit_sku VARCHAR(50), status VARCHAR(20),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON kitting_orders (tenant_id, order_id);

CREATE TABLE pod_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    delivery_id UUID, signature_s3_key VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON pod_records (tenant_id, delivery_id);

CREATE TABLE consolidations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    master_shipment_id UUID, po_ids UUID[],
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON consolidations (tenant_id, master_shipment_id);

CREATE TABLE sample_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID, year INT, samples_used INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON sample_limits (tenant_id, account_id);

CREATE TABLE hazmat_checks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID, is_cleared BOOLEAN, checked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON hazmat_checks (tenant_id, order_id);

CREATE TABLE trade_compliance (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID, ofac_cleared BOOLEAN,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON trade_compliance (tenant_id, account_id);

CREATE TABLE order_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID, created_at TIMESTAMPTZ, fulfilled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON order_metrics (tenant_id, order_id);

CREATE TABLE contracts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID, docusign_env_id VARCHAR(100), status VARCHAR(20),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON contracts (tenant_id, account_id);

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    entity_id UUID, entity_type VARCHAR(50), event_type VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON audit_logs (tenant_id, entity_id);

CREATE TABLE approval_workflows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL REFERENCES orders(id),
    state JSONB NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON approval_workflows (tenant_id, order_id);

CREATE TABLE quote_revisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    quote_id UUID NOT NULL,
    proposed_price DECIMAL(12, 4) NOT NULL,
    revision_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON quote_revisions (tenant_id, quote_id);

CREATE EXTENSION IF NOT EXISTS ltree;
  CREATE TABLE account_hierarchies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    path ltree NOT NULL,
    budget_limit DECIMAL(15, 2),
    spent_amount DECIMAL(15, 2) DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON account_hierarchies USING GIST (path);

CREATE TABLE punchout_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    buyer_cookie VARCHAR(255) NOT NULL,
    payload TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON punchout_sessions (buyer_cookie);

CREATE TABLE contract_prices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID NOT NULL,
    sku VARCHAR(100) NOT NULL,
    min_qty INT NOT NULL DEFAULT 1,
    price DECIMAL(12, 4) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON contract_prices (tenant_id, account_id, sku);

CREATE TABLE subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID NOT NULL,
    cron_schedule VARCHAR(50) NOT NULL,
    next_run TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON subscriptions (next_run);

CREATE TABLE routing_decisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL,
    ai_confidence DECIMAL(5, 4),
    selected_warehouse VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON routing_decisions (tenant_id, order_id);

CREATE TABLE trade_credits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    invoice_id UUID NOT NULL,
    provider VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    due_date TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON trade_credits (due_date);

CREATE TABLE fulfillments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL REFERENCES orders(id),
    status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE TABLE fulfillment_lines (
    id UUID PRIMARY KEY,
    fulfillment_id UUID REFERENCES fulfillments(id),
    sku VARCHAR(100),
    qty INT
  );

CREATE TABLE delay_predictions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL,
    risk_score DECIMAL(3, 2),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON delay_predictions (tenant_id, order_id);

CREATE TABLE rmas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL,
    status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON rmas (status);

CREATE TABLE edi_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    raw_payload TEXT NOT NULL,
    parsed_json JSONB,
    status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE dealer_quotas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    dealer_id UUID NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    target_amount DECIMAL(15, 2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE collaborative_carts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    crdt_state BYTEA NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE freight_quotes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    cart_id UUID NOT NULL,
    carrier VARCHAR(100) NOT NULL,
    rate DECIMAL(10, 2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE tax_exemptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID NOT NULL,
    state VARCHAR(2) NOT NULL,
    s3_path VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL,
    expires_at DATE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE dropship_vendors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(100) NOT NULL,
    hashed_api_key VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE fx_reconciliations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL,
    locked_rate DECIMAL(10, 6) NOT NULL,
    settlement_rate DECIMAL(10, 6) NOT NULL,
    variance DECIMAL(10, 2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE reorder_points (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID NOT NULL,
    sku VARCHAR(100) NOT NULL,
    threshold INT NOT NULL,
    replenish_qty INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE rebates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    contract_id UUID NOT NULL,
    accrued_amount DECIMAL(15, 2) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE assortment_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID NOT NULL,
    rule_type VARCHAR(20) NOT NULL, -- 'include' or 'exclude'
    sku_pattern VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE serialized_items (
    serial_number VARCHAR(100) PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(100) NOT NULL,
    order_id UUID,
    status VARCHAR(50) NOT NULL DEFAULT 'in_stock',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON serialized_items (tenant_id, sku, status);

CREATE TABLE consignment_stock (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    owner_id UUID NOT NULL,
    location_id UUID NOT NULL,
    sku VARCHAR(100) NOT NULL,
    qty INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE bulk_import_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    file_name VARCHAR(255) NOT NULL,
    error_report JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE dunning_states (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID NOT NULL,
    current_stage INT NOT NULL DEFAULT 1,
    last_contacted TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

