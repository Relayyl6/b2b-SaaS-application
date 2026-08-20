-- Auto-generated foundation from 800+ feature architecture blueprints

CREATE TABLE variant_matrices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    base_product_id UUID NOT NULL REFERENCES products(id),
    dimensions JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON variant_matrices (tenant_id);

CREATE TABLE inventory_reservations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL,
    reserved_items JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
  );
  CREATE INDEX ON inventory_reservations (tenant_id);

CREATE TABLE product_assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    product_id UUID NOT NULL REFERENCES products(id),
    asset_type VARCHAR(50) NOT NULL,
    s3_key VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON product_assets (tenant_id, product_id);

CREATE TABLE inventory_ledger (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku_id UUID NOT NULL,
    delta INTEGER NOT NULL,
    reason_code VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON inventory_ledger (tenant_id, sku_id);

CREATE TABLE bulk_import_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    file_url VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON bulk_import_jobs (tenant_id, status);

CREATE TABLE product_bundles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    bundle_sku_id UUID NOT NULL,
    component_sku_id UUID NOT NULL,
    qty INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON product_bundles (tenant_id, bundle_sku_id);

CREATE TABLE price_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(255) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE TABLE price_list_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    price_list_id UUID NOT NULL REFERENCES price_lists(id),
    sku_id UUID NOT NULL,
    price DECIMAL(12, 4) NOT NULL
  );
  CREATE INDEX ON price_list_entries (price_list_id, sku_id);

CREATE TABLE product_lifecycle_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    product_id UUID NOT NULL REFERENCES products(id),
    target_state VARCHAR(50) NOT NULL,
    scheduled_for TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON product_lifecycle_events (scheduled_for) WHERE target_state = 'pending';

CREATE TABLE search_sync_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sync_status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE inventory_aging_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    report_url VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE serialized_inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku_id UUID NOT NULL,
    serial_number VARCHAR(255) NOT NULL UNIQUE,
    status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON serialized_inventory (serial_number);

CREATE TABLE inventory_lots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku_id UUID NOT NULL,
    lot_number VARCHAR(100) NOT NULL,
    expiry_date TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON inventory_lots (expiry_date);

CREATE TABLE location_inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    location_id UUID NOT NULL,
    sku_id UUID NOT NULL,
    qty_available INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON location_inventory (tenant_id, sku_id);

CREATE TABLE reorder_points (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku_id UUID NOT NULL,
    suggested_qty INTEGER NOT NULL,
    confidence_score DECIMAL(5, 2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE product_substitutions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    original_sku_id UUID NOT NULL,
    substitute_sku_id UUID NOT NULL,
    rule_type VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON product_substitutions (original_sku_id);

CREATE TABLE product_barcodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku_id UUID NOT NULL,
    barcode_data TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE catalog_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    product_id UUID NOT NULL,
    snapshot JSONB NOT NULL,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE attribute_schemas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    category_id UUID NOT NULL,
    schema_definition JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE product_tariffs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku_id UUID NOT NULL,
    hs_code VARCHAR(20) NOT NULL,
    country_of_origin VARCHAR(2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE webhook_endpoints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    target_url VARCHAR(255) NOT NULL,
    secret_key VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE sku_registry (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(255) NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, sku)
  );
  CREATE INDEX ON sku_registry (tenant_id);

CREATE TABLE inventory_locks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku_id UUID NOT NULL REFERENCES sku_registry(id),
    warehouse_id UUID NOT NULL,
    quantity INT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON inventory_locks (tenant_id, sku_id);

CREATE TABLE pricing_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID NOT NULL,
    rule_script TEXT NOT NULL,
    priority INT DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON pricing_rules (account_id);

CREATE TABLE product_tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES sku_registry(id),
    tag VARCHAR(100) NOT NULL,
    confidence FLOAT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON product_tags (product_id);

CREATE TABLE categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    parent_id UUID REFERENCES categories(id),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON categories (tenant_id, parent_id);

CREATE TABLE product_variants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES sku_registry(id),
    attributes JSONB NOT NULL,
    sku VARCHAR(255) UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON product_variants USING GIN (attributes);

CREATE TABLE inventory_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    total_records INT NOT NULL,
    processed_records INT DEFAULT 0,
    status VARCHAR(50) DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE low_stock_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sku_id UUID NOT NULL REFERENCES sku_registry(id),
    predicted_stockout_date DATE NOT NULL,
    confidence FLOAT NOT NULL,
    is_acknowledged BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON low_stock_alerts (sku_id, is_acknowledged);

CREATE TABLE bulk_validations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    total_lines INT NOT NULL,
    error_count INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE pricelists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    currency VARCHAR(3) NOT NULL,
    rates JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON pricelists (tenant_id, currency);

CREATE TABLE inventory_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sku_id UUID NOT NULL REFERENCES sku_registry(id),
    snapshot_date DATE NOT NULL,
    quantity INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON inventory_snapshots (sku_id, snapshot_date);

CREATE TABLE supplier_catalogs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    supplier_id UUID NOT NULL,
    raw_data JSONB NOT NULL,
    normalized_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE product_bundles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    bundle_name VARCHAR(255) NOT NULL,
    graph_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE backorders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL,
    sku_id UUID NOT NULL REFERENCES sku_registry(id),
    pending_qty INT NOT NULL,
    status VARCHAR(50) DEFAULT 'awaiting_stock',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON backorders (sku_id, status);

CREATE TABLE syndicated_products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_tenant_id UUID NOT NULL,
    target_tenant_id UUID NOT NULL,
    sku_id UUID NOT NULL,
    overrides JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON syndicated_products (target_tenant_id);

CREATE TABLE seo_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sku_id UUID NOT NULL REFERENCES sku_registry(id),
    meta_title VARCHAR(255) NOT NULL,
    meta_description TEXT NOT NULL,
    auto_generated BOOLEAN DEFAULT TRUE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE volume_pricing (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sku_id UUID NOT NULL REFERENCES sku_registry(id),
    min_qty INT NOT NULL,
    price_per_unit DECIMAL(10,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON volume_pricing (sku_id, min_qty);

CREATE TABLE stock_allocations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cart_id UUID NOT NULL,
    sku_id UUID NOT NULL,
    qty INT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE sourcing_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    warehouse_id UUID NOT NULL,
    geo_polygon JSONB NOT NULL,
    priority INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE lifecycle_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sku_id UUID NOT NULL REFERENCES sku_registry(id),
    previous_state VARCHAR(50) NOT NULL,
    new_state VARCHAR(50) NOT NULL,
    trigger_action VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE translations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sku_id UUID NOT NULL REFERENCES sku_registry(id),
    locale VARCHAR(10) NOT NULL,
    translated_content JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(sku_id, locale)
  );
  CREATE INDEX ON translations (sku_id, locale);

CREATE TABLE product_relations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_sku_id UUID NOT NULL,
    target_sku_id UUID NOT NULL,
    relation_type VARCHAR(50) NOT NULL, -- e.g., 'accessory', 'upsell'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON product_relations (source_sku_id);

CREATE TABLE omni_sync_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pos_id UUID NOT NULL,
    event_payload JSONB NOT NULL,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE spec_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category_id UUID NOT NULL REFERENCES categories(id),
    json_schema JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE catalog_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    version_name VARCHAR(255) NOT NULL,
    changeset JSONB NOT NULL,
    status VARCHAR(50) DEFAULT 'draft',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE import_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    file_url TEXT NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    total_records INT DEFAULT 0,
    processed_records INT DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON import_jobs (tenant_id, status);

CREATE TABLE inventory_levels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(255) NOT NULL,
    warehouse_id UUID NOT NULL REFERENCES warehouses(id),
    available_qty INT NOT NULL DEFAULT 0,
    reserved_qty INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, sku, warehouse_id)
  );
  CREATE INDEX ON inventory_levels (tenant_id, sku);

CREATE TABLE pricing_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    customer_group_id UUID,
    sku VARCHAR(255),
    min_qty INT DEFAULT 1,
    price_modifier JSONB NOT NULL,
    priority INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON pricing_rules (tenant_id, customer_group_id, sku);

CREATE TABLE category_ml_features (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    buyer_id UUID NOT NULL,
    category_id UUID NOT NULL,
    feature_vector VECTOR(128),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON category_ml_features USING ivfflat (feature_vector vector_cosine_ops);

CREATE TABLE bundle_components (
    bundle_sku VARCHAR(255) NOT NULL,
    component_sku VARCHAR(255) NOT NULL,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    qty_required INT NOT NULL DEFAULT 1,
    PRIMARY KEY (tenant_id, bundle_sku, component_sku)
  );

CREATE TABLE inventory_reservations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(255) NOT NULL,
    qty INT NOT NULL,
    cart_id UUID NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON inventory_reservations (expires_at);

CREATE TABLE buyer_catalog_entitlements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    buyer_org_id UUID NOT NULL,
    sku_inclusion_list TEXT[],
    category_inclusion_list TEXT[],
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON buyer_catalog_entitlements (tenant_id, buyer_org_id);

CREATE TABLE pricing_suggestions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(255) NOT NULL,
    suggested_price NUMERIC(10, 2) NOT NULL,
    confidence FLOAT NOT NULL,
    applied BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON pricing_suggestions (tenant_id, sku, applied);

CREATE TABLE products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(255) NOT NULL UNIQUE,
    attributes JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX idx_products_attributes ON products USING GIN (attributes);

CREATE TABLE product_compliance (
    sku VARCHAR(255) PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    hts_code VARCHAR(50),
    eccn VARCHAR(50),
    is_hazardous BOOLEAN DEFAULT FALSE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE product_variants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_sku VARCHAR(255) NOT NULL,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    variant_sku VARCHAR(255) NOT NULL UNIQUE,
    option_hash VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON product_variants (tenant_id, parent_sku);

CREATE TABLE inventory_future_pools (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(255) NOT NULL,
    po_number VARCHAR(255) NOT NULL,
    qty_expected INT NOT NULL,
    qty_consumed INT NOT NULL DEFAULT 0,
    expected_arrival DATE NOT NULL
  );
  CREATE INDEX ON inventory_future_pools (tenant_id, sku, expected_arrival);

CREATE TABLE purchase_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    supplier_id UUID NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    line_items JSONB NOT NULL,
    created_by_system BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE uom_conversions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(255) NOT NULL,
    uom VARCHAR(50) NOT NULL,
    conversion_factor NUMERIC(15, 6) NOT NULL,
    base_uom VARCHAR(50) NOT NULL,
    UNIQUE(tenant_id, sku, uom)
  );

CREATE TABLE product_overrides (
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(255) NOT NULL,
    overridden_fields JSONB NOT NULL,
    PRIMARY KEY (tenant_id, sku)
  );

CREATE TABLE product_images (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sku VARCHAR(255) NOT NULL,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    base_url TEXT NOT NULL,
    is_primary BOOLEAN DEFAULT FALSE
  );
  CREATE INDEX ON product_images (tenant_id, sku);

CREATE TABLE inventory_lots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(255) NOT NULL,
    lot_number VARCHAR(100) NOT NULL,
    qty_available INT NOT NULL,
    expiration_date DATE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX idx_lot_allocation ON inventory_lots (tenant_id, sku, expiration_date ASC);

CREATE TABLE scheduled_price_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    target_category_id UUID NOT NULL,
    modifier_type VARCHAR(20) NOT NULL,
    modifier_value NUMERIC(5,2) NOT NULL,
    execution_time TIMESTAMPTZ NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending'
  );

CREATE TABLE supplier_inventory (
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    supplier_id UUID NOT NULL,
    sku VARCHAR(255) NOT NULL,
    available_qty INT NOT NULL DEFAULT 0,
    PRIMARY KEY (tenant_id, supplier_id, sku)
  );

CREATE TABLE product_relationships (
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    source_sku VARCHAR(255) NOT NULL,
    target_sku VARCHAR(255) NOT NULL,
    relationship_type VARCHAR(50) NOT NULL, -- 'requires', 'upsell', 'replacement'
    PRIMARY KEY (tenant_id, source_sku, target_sku, relationship_type)
  );
  CREATE INDEX ON product_relationships (source_sku, relationship_type);

CREATE TABLE moq_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(255) NOT NULL,
    rhai_script TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE catalog_visibility (
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    customer_group_id UUID NOT NULL,
    blocked_brands TEXT[] NOT NULL DEFAULT '{}',
    blocked_categories TEXT[] NOT NULL DEFAULT '{}',
    PRIMARY KEY (tenant_id, customer_group_id)
  );

CREATE TABLE serialized_inventory (
    serial_number VARCHAR(100) PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'in_stock', -- 'in_stock', 'sold'
    order_id UUID,
    warranty_expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON serialized_inventory (sku, status);

