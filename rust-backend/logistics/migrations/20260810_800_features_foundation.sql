-- Auto-generated foundation from 800+ feature architecture blueprints

CREATE TABLE inventory_sync_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    warehouse_id VARCHAR(50) NOT NULL,
    sku VARCHAR(50) NOT NULL,
    quantity_delta INT NOT NULL,
    new_quantity INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON inventory_sync_logs (tenant_id);

CREATE TABLE fleet_routes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    fleet_id VARCHAR(50) NOT NULL,
    polyline TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON fleet_routes (tenant_id);

CREATE TABLE rfid_scans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    rfid_tag VARCHAR(255) NOT NULL,
    dock_id VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON rfid_scans (tenant_id);

CREATE TABLE third_party_dispatch (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL,
    provider_code VARCHAR(100) NOT NULL,
    provider_reference VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON third_party_dispatch (tenant_id);

CREATE TABLE delivery_reallocations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    package_id VARCHAR(50) NOT NULL,
    new_fleet_id VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON delivery_reallocations (tenant_id);

CREATE TABLE customs_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    shipment_id UUID NOT NULL,
    document_type VARCHAR(50) NOT NULL,
    s3_url TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON customs_documents (tenant_id);

CREATE TABLE rate_quotes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    carrier_name VARCHAR(100) NOT NULL,
    service_level VARCHAR(100) NOT NULL,
    price_usd NUMERIC(10,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON rate_quotes (tenant_id);

CREATE TABLE shipment_tracking (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    tracking_number VARCHAR(100) NOT NULL,
    carrier VARCHAR(50) NOT NULL,
    latest_status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON shipment_tracking (tenant_id);

CREATE TABLE reverse_logistics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    rma_number VARCHAR(100) NOT NULL,
    original_order_id UUID NOT NULL,
    condition VARCHAR(50) NOT NULL,
    resolution VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON reverse_logistics (tenant_id);

CREATE TABLE iot_temperature_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sensor_id VARCHAR(100) NOT NULL,
    shipment_id UUID NOT NULL,
    breach_celsius NUMERIC(5,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON iot_temperature_alerts (tenant_id);

CREATE TABLE hazmat_validations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    un_number VARCHAR(20) NOT NULL,
    is_compliant BOOLEAN NOT NULL,
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON hazmat_validations (tenant_id);

CREATE TABLE purchase_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    supplier_id VARCHAR(100) NOT NULL,
    po_status VARCHAR(50) NOT NULL,
    total_amount NUMERIC(15,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON purchase_orders (tenant_id);

CREATE TABLE vmi_stock_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    vendor_id VARCHAR(100) NOT NULL,
    location_id VARCHAR(100) NOT NULL,
    reported_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON vmi_stock_reports (tenant_id);

CREATE TABLE dropship_routes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL,
    manufacturer_id VARCHAR(100) NOT NULL,
    routing_status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON dropship_routes (tenant_id);

CREATE TABLE order_incoterms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL,
    incoterm VARCHAR(3) NOT NULL,
    named_port VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON order_incoterms (tenant_id);

CREATE TABLE dock_appointments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    carrier_id VARCHAR(100) NOT NULL,
    dock_door VARCHAR(20) NOT NULL,
    scheduled_time TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON dock_appointments (tenant_id);

CREATE TABLE freight_load_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    trailer_type VARCHAR(50) NOT NULL,
    utilization_percentage NUMERIC(5,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON freight_load_plans (tenant_id);

CREATE TABLE shipment_emissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    shipment_id UUID NOT NULL,
    co2_emissions_kg NUMERIC(10,2) NOT NULL,
    mode VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON shipment_emissions (tenant_id);

CREATE TABLE packaging_suggestions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    box_sku VARCHAR(50) NOT NULL,
    void_fill_percentage NUMERIC(5,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON packaging_suggestions (tenant_id);

CREATE TABLE equipment_health_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    equipment_id VARCHAR(100) NOT NULL,
    maintenance_required BOOLEAN NOT NULL,
    anomaly_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON equipment_health_logs (tenant_id);

CREATE TABLE yard_inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    container_id VARCHAR(100) NOT NULL,
    yard_slot VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON yard_inventory (tenant_id);

CREATE TABLE inventory_allocations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL,
    warehouse_id UUID NOT NULL,
    sku VARCHAR(255) NOT NULL,
    quantity INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON inventory_allocations (tenant_id, order_id);

CREATE TABLE carrier_rates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    carrier_id UUID NOT NULL,
    base_cost DECIMAL(10,2) NOT NULL,
    predicted_delay_prob FLOAT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON carrier_rates (tenant_id, carrier_id);

CREATE TABLE delivery_predictions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    shipment_id UUID NOT NULL,
    delay_hours INT NOT NULL,
    confidence FLOAT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON delivery_predictions (tenant_id, shipment_id);

CREATE TABLE cross_dock_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    asn_id UUID NOT NULL,
    lane_id VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON cross_dock_events (tenant_id, asn_id);

CREATE TABLE edi_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    partner_id UUID NOT NULL,
    edi_type VARCHAR(10) NOT NULL,
    raw_payload TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON edi_transactions (tenant_id, partner_id);

CREATE TABLE rmas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL,
    status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON rmas (tenant_id, status);

CREATE TABLE route_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    fleet_id UUID NOT NULL,
    optimized_route JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON route_plans (tenant_id, fleet_id);

CREATE TABLE freight_audits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    carrier_id UUID NOT NULL,
    variance_amount DECIMAL(10,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON freight_audits (tenant_id, carrier_id);

CREATE TABLE landed_costs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    hs_code VARCHAR(20) NOT NULL,
    calculated_duty DECIMAL(10,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON landed_costs (tenant_id, hs_code);

CREATE TABLE load_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    utilization FLOAT NOT NULL,
    plan_json JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON load_plans (tenant_id);

CREATE TABLE serial_tracking (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(255) NOT NULL,
    serial_number VARCHAR(255) NOT NULL,
    order_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON serial_tracking (tenant_id, serial_number);

CREATE TABLE shipment_legs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    journey_id UUID NOT NULL,
    leg_sequence INT NOT NULL,
    status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON shipment_legs (tenant_id, journey_id);

CREATE TABLE replenishment_forecasts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(255) NOT NULL,
    forecast_date DATE NOT NULL,
    qty INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON replenishment_forecasts (tenant_id, sku);

CREATE TABLE geofence_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    truck_id UUID NOT NULL,
    fence_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON geofence_events (tenant_id, fence_id);

CREATE TABLE temperature_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    shipment_id UUID NOT NULL,
    temp_c DECIMAL(5,2) NOT NULL,
    is_violation BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON temperature_logs (tenant_id, shipment_id);

CREATE TABLE hazmat_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    un_number VARCHAR(10) NOT NULL,
    rule_json JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON hazmat_rules (tenant_id, un_number);

CREATE TABLE proof_of_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    shipment_id UUID NOT NULL,
    s3_key VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON proof_of_deliveries (tenant_id, shipment_id);

CREATE TABLE shipment_consolidations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    buyer_id UUID NOT NULL,
    shipment_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON shipment_consolidations (tenant_id, buyer_id);

CREATE TABLE carrier_slas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    tracking_number VARCHAR(100) NOT NULL,
    promised_date TIMESTAMPTZ NOT NULL,
    actual_date TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON carrier_slas (tenant_id, promised_date);

CREATE TABLE yard_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    trailer_id VARCHAR(50) NOT NULL,
    dock_id VARCHAR(50),
    status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON yard_events (tenant_id, trailer_id);

CREATE TABLE dsv_inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    vendor_id UUID NOT NULL,
    sku VARCHAR(255) NOT NULL,
    qty INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON dsv_inventory (tenant_id, vendor_id, sku);

CREATE TABLE carbon_emissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    shipment_id UUID NOT NULL,
    co2_kg DECIMAL(10,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON carbon_emissions (tenant_id, shipment_id);

CREATE TABLE box_suggestions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL,
    box_sku VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON box_suggestions (tenant_id, order_id);

CREATE TABLE customs_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    shipment_id UUID NOT NULL,
    doc_url VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON customs_documents (tenant_id, shipment_id);

CREATE TABLE bulk_shipment_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL,
    schedule JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON bulk_shipment_plans (tenant_id, order_id);

CREATE TABLE inventory_allocations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL,
    warehouse_id UUID NOT NULL,
    status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON inventory_allocations (tenant_id, order_id);

CREATE TABLE delivery_routes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    fleet_id UUID NOT NULL,
    route_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON delivery_routes (tenant_id, fleet_id);

CREATE TABLE customs_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    shipment_id UUID NOT NULL,
    document_type VARCHAR(100) NOT NULL,
    s3_key VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON customs_documents (tenant_id, shipment_id);

CREATE TABLE sensor_telemetry (
    time TIMESTAMPTZ NOT NULL,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sensor_id VARCHAR(100) NOT NULL,
    temperature NUMERIC(5,2) NOT NULL
  );
  SELECT create_hypertable('sensor_telemetry', 'time');
  CREATE INDEX ON sensor_telemetry (tenant_id, sensor_id, time DESC);

CREATE TABLE dropship_routing (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_line_id UUID NOT NULL,
    vendor_id UUID NOT NULL,
    vendor_po_ref VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON dropship_routing (tenant_id, order_line_id);

CREATE TABLE freight_quotes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    request_hash VARCHAR(64) NOT NULL,
    forwarder VARCHAR(100) NOT NULL,
    price NUMERIC(10,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON freight_quotes (tenant_id, request_hash);

CREATE TABLE rma_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL,
    decision VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON rma_requests (tenant_id, order_id);

CREATE TABLE split_shipments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    original_order_id UUID NOT NULL,
    optimized_groups JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON split_shipments (tenant_id, original_order_id);

CREATE TABLE jit_replenishments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(100) NOT NULL,
    po_generated UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON jit_replenishments (tenant_id, sku);

CREATE TABLE robot_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    robot_id VARCHAR(50) NOT NULL,
    task_type VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON robot_tasks (tenant_id, robot_id, status);

CREATE TABLE geofences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    polygon GEOMETRY(Polygon, 4326) NOT NULL,
    site_id UUID NOT NULL
  );
  CREATE INDEX ON geofences USING GIST (polygon);

CREATE TABLE freight_loads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    load_type VARCHAR(10) NOT NULL,
    total_volume NUMERIC(10,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON freight_loads (tenant_id, load_type);

CREATE TABLE hazmat_manifests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    shipment_id UUID NOT NULL,
    un_numbers TEXT[] NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON hazmat_manifests (tenant_id, shipment_id);

CREATE TABLE carrier_rates_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    selected_carrier VARCHAR(50) NOT NULL,
    price NUMERIC(8,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON carrier_rates_log (tenant_id, created_at);

CREATE TABLE restock_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(100) NOT NULL,
    predicted_date DATE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON restock_alerts (tenant_id, predicted_date);

CREATE TABLE dock_appointments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    dock_id VARCHAR(50) NOT NULL,
    time_range TSTZRANGE NOT NULL,
    EXCLUDE USING GIST (dock_id WITH =, time_range WITH &&)
  );

CREATE TABLE picking_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    worker_id UUID NOT NULL,
    optimized_path JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON picking_batches (tenant_id, worker_id);

CREATE TABLE provenance_ledger (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    serial_number VARCHAR(100) NOT NULL,
    tx_hash VARCHAR(64) NOT NULL UNIQUE,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON provenance_ledger (tenant_id, serial_number);

CREATE TABLE proof_of_delivery (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    delivery_id UUID NOT NULL,
    signature_s3_key VARCHAR(255),
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON proof_of_delivery (tenant_id, delivery_id);

CREATE TABLE cross_dock_matches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    inbound_po VARCHAR(100) NOT NULL,
    outbound_order_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON cross_dock_matches (tenant_id, inbound_po);

CREATE TABLE carbon_emissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    shipment_id UUID NOT NULL,
    emissions_kg NUMERIC(10,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON carbon_emissions (tenant_id, shipment_id);

CREATE TABLE carrier_audits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    tracking_number VARCHAR(100) NOT NULL,
    sla_breach_mins INT NOT NULL,
    refund_status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON carrier_audits (tenant_id, tracking_number);

CREATE TABLE packaging_suggestions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL,
    suggested_box VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON packaging_suggestions (tenant_id, order_id);

CREATE TABLE kitting_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    kit_sku VARCHAR(100) NOT NULL,
    status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON kitting_jobs (tenant_id, status);

CREATE TABLE ugly_freight_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(100) NOT NULL,
    handling_flags INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ugly_freight_rules (tenant_id, sku);

