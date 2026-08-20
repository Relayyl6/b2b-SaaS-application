-- Auto-generated foundation from 800+ feature architecture blueprints

CREATE TABLE ai_search_embeddings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    product_id UUID NOT NULL REFERENCES products(id),
    embedding vector(384) NOT NULL,
    model_version VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_search_embeddings (tenant_id, created_at DESC);
  CREATE INDEX ON ai_search_embeddings USING hnsw (embedding vector_cosine_ops);

CREATE TABLE ai_pricing_recommendations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    product_id UUID NOT NULL,
    recommended_price NUMERIC(10, 2) NOT NULL,
    confidence FLOAT NOT NULL,
    model_version VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_pricing_recommendations (tenant_id, created_at DESC);

CREATE TABLE ai_inventory_forecasts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(100) NOT NULL,
    forecasted_demand INT NOT NULL,
    confidence FLOAT NOT NULL,
    model_version VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_inventory_forecasts (tenant_id, created_at DESC);

CREATE TABLE ai_support_chat_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    session_id UUID NOT NULL,
    message TEXT NOT NULL,
    reply TEXT NOT NULL,
    model_version VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_support_chat_logs (tenant_id, created_at DESC);

CREATE TABLE ai_procurement_agents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    rfq_id VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL,
    best_offer_amount NUMERIC(10, 2),
    model_version VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_procurement_agents (tenant_id, created_at DESC);

CREATE TABLE ai_fraud_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL,
    fraud_score FLOAT NOT NULL,
    model_version VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_fraud_scores (tenant_id, created_at DESC);

CREATE TABLE ai_document_extractions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    document_s3_key VARCHAR(255) NOT NULL,
    confidence FLOAT NOT NULL,
    model_version VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_document_extractions (tenant_id, created_at DESC);

CREATE TABLE ai_generated_content (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    product_id UUID NOT NULL,
    content_hash VARCHAR(64) NOT NULL,
    model_version VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_generated_content (tenant_id, created_at DESC);

CREATE TABLE ai_demand_forecasts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    category_id UUID NOT NULL,
    projected_volume NUMERIC(15, 2) NOT NULL,
    confidence FLOAT NOT NULL,
    model_version VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_demand_forecasts (tenant_id, created_at DESC);

CREATE TABLE ai_churn_predictions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    company_id UUID NOT NULL,
    churn_probability FLOAT NOT NULL,
    model_version VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_churn_predictions (tenant_id, created_at DESC);

CREATE TABLE ai_contract_analysis_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    document_id UUID NOT NULL,
    risk_score FLOAT NOT NULL,
    model_version VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_contract_analysis_results (tenant_id, created_at DESC);

CREATE TABLE ai_image_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    image_url VARCHAR(255) NOT NULL,
    quality_score FLOAT NOT NULL,
    model_version VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_image_scores (tenant_id, created_at DESC);

CREATE TABLE ai_product_recommendations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    base_product_id UUID NOT NULL,
    recommended_product_id UUID NOT NULL,
    score FLOAT NOT NULL,
    model_version VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_product_recommendations (tenant_id, base_product_id);

CREATE TABLE ai_email_extractions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    email_message_id VARCHAR(255) NOT NULL,
    confidence FLOAT NOT NULL,
    model_version VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_email_extractions (tenant_id, created_at DESC);

CREATE TABLE ai_sentiment_analysis (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    source_id UUID NOT NULL,
    sentiment_score FLOAT NOT NULL,
    model_version VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_sentiment_analysis (tenant_id, created_at DESC);

CREATE TABLE ai_supplier_risk_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    supplier_id UUID NOT NULL,
    risk_score FLOAT NOT NULL,
    model_version VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_supplier_risk_scores (tenant_id, created_at DESC);

CREATE TABLE ai_tax_classifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    product_id UUID NOT NULL,
    suggested_tax_code VARCHAR(50) NOT NULL,
    confidence FLOAT NOT NULL,
    model_version VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_tax_classifications (tenant_id, created_at DESC);

CREATE TABLE ai_conversational_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    session_id UUID NOT NULL,
    intent VARCHAR(100) NOT NULL,
    model_version VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_conversational_sessions (tenant_id, created_at DESC);

CREATE TABLE ai_defect_inspections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    receipt_id UUID NOT NULL,
    is_defective BOOLEAN NOT NULL,
    confidence FLOAT NOT NULL,
    model_version VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_defect_inspections (tenant_id, created_at DESC);

CREATE TABLE ai_experiment_allocations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    experiment_id UUID NOT NULL,
    assigned_variant VARCHAR(50) NOT NULL,
    model_version VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_experiment_allocations (tenant_id, experiment_id);

CREATE TABLE ai_inventory_proposals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(255) NOT NULL,
    source_warehouse_id UUID NOT NULL,
    target_warehouse_id UUID NOT NULL,
    quantity INT NOT NULL,
    confidence_score FLOAT NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_inventory_proposals_tenant_status ON ai_inventory_proposals (tenant_id, status);

CREATE TABLE ai_churn_predictions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    customer_id UUID NOT NULL,
    churn_probability FLOAT NOT NULL,
    primary_factor VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_churn_predictions (tenant_id, customer_id);

CREATE TABLE ai_price_optimizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(255) NOT NULL,
    customer_tier VARCHAR(50) NOT NULL,
    recommended_price DECIMAL(10,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_price_optimizations (tenant_id, sku);

CREATE TABLE ai_fraud_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL,
    risk_score FLOAT NOT NULL,
    action_taken VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_fraud_logs (tenant_id, risk_score);

CREATE TABLE ai_generated_pos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    supplier_id UUID NOT NULL,
    total_estimated_cost DECIMAL(12,2) NOT NULL,
    status VARCHAR(50) DEFAULT 'draft',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_generated_pos (tenant_id, supplier_id);

CREATE TABLE ai_search_queries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    raw_query TEXT NOT NULL,
    vector_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_search_queries (tenant_id);

CREATE TABLE ai_abandonment_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    cart_id UUID NOT NULL,
    action_type VARCHAR(50) NOT NULL,
    converted BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_abandonment_actions (tenant_id, cart_id);

CREATE TABLE ai_ticket_triage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    ticket_id UUID NOT NULL,
    category VARCHAR(100) NOT NULL,
    priority VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_ticket_triage (tenant_id, priority);

CREATE TABLE ai_lead_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    company_domain VARCHAR(255) NOT NULL,
    score FLOAT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_lead_scores (tenant_id, score DESC);

CREATE TABLE ai_generated_descriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(255) NOT NULL,
    generated_text TEXT NOT NULL,
    approved BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_generated_descriptions (tenant_id, approved);

CREATE TABLE ai_environmental_forecasts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    region_id VARCHAR(100) NOT NULL,
    event_trigger VARCHAR(100) NOT NULL,
    demand_multiplier FLOAT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_environmental_forecasts (tenant_id, region_id);

CREATE TABLE ai_visual_searches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    image_hash VARCHAR(64) NOT NULL,
    matched_sku VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_visual_searches (tenant_id, image_hash);

CREATE TABLE ai_catalog_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL,
    category_weights JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_catalog_preferences (tenant_id, user_id);

CREATE TABLE ai_tax_classifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(255) NOT NULL,
    hs_code VARCHAR(50) NOT NULL,
    confidence FLOAT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_tax_classifications (tenant_id, hs_code);

CREATE TABLE ai_route_optimizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    fleet_id VARCHAR(100) NOT NULL,
    original_distance FLOAT NOT NULL,
    optimized_distance FLOAT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_route_optimizations (tenant_id, fleet_id);

CREATE TABLE ai_seo_tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(255) NOT NULL,
    title_tag VARCHAR(255) NOT NULL,
    meta_description TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_seo_tags (tenant_id, sku);

CREATE TABLE ai_competitor_prices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(255) NOT NULL,
    competitor_domain VARCHAR(255) NOT NULL,
    observed_price DECIMAL(10,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_competitor_prices (tenant_id, sku);

CREATE TABLE ai_voice_commands (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL,
    raw_transcript TEXT NOT NULL,
    parsed_intent JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_voice_commands (tenant_id, user_id);

CREATE TABLE ai_returns_triage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    return_id UUID NOT NULL,
    action_decided VARCHAR(50) NOT NULL,
    ai_confidence FLOAT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_returns_triage (tenant_id, action_decided);

CREATE TABLE ai_product_relations (
    source_sku VARCHAR(255) NOT NULL,
    target_sku VARCHAR(255) NOT NULL,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    relation_weight FLOAT NOT NULL,
    PRIMARY KEY (tenant_id, source_sku, target_sku)
  );
  CREATE INDEX ON ai_product_relations (tenant_id, source_sku);

CREATE TABLE ai_negotiation_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    customer_id UUID NOT NULL,
    recommended_discount FLOAT NOT NULL,
    win_probability FLOAT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_negotiation_metrics (tenant_id, customer_id);

CREATE TABLE ai_experiment_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    experiment_id VARCHAR(100) NOT NULL,
    winning_variant VARCHAR(100),
    traffic_allocation JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_experiment_results (tenant_id, experiment_id);

CREATE TABLE ai_inventory_aging (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(255) NOT NULL,
    risk_level VARCHAR(50) NOT NULL,
    suggested_action VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_inventory_aging (tenant_id, risk_level);

CREATE TABLE ai_customer_segments (
    customer_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    cluster_id INT NOT NULL,
    cluster_name VARCHAR(100) NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_customer_segments (tenant_id, cluster_id);

CREATE TABLE ai_bi_query_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    natural_query TEXT NOT NULL,
    generated_sql TEXT NOT NULL,
    execution_time_ms INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_bi_query_logs (tenant_id);

CREATE TABLE ai_po_extractions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    document_s3_key VARCHAR(255) NOT NULL,
    extracted_data JSONB NOT NULL,
    confidence NUMERIC(4,3) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_po_extractions (tenant_id);

CREATE TABLE ai_inventory_predictions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku_id UUID NOT NULL REFERENCES products(id),
    predicted_demand INT NOT NULL,
    target_date DATE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_inventory_predictions (tenant_id, sku_id);

CREATE TABLE ai_price_optimizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku_id UUID NOT NULL,
    segment_id UUID NOT NULL,
    suggested_price DECIMAL(10,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON ai_price_optimizations (tenant_id, sku_id, segment_id);

CREATE TABLE ai_rfq_triage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    rfq_id UUID NOT NULL REFERENCES rfqs(id),
    priority_score INT NOT NULL,
    auto_routed_to UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_rfq_triage (tenant_id, priority_score);

CREATE TABLE ai_fraud_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    transaction_id UUID NOT NULL,
    risk_score NUMERIC(5,4) NOT NULL,
    decision VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_fraud_logs (transaction_id);

CREATE TABLE ai_product_tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    product_id UUID NOT NULL REFERENCES products(id),
    auto_tags TEXT[] NOT NULL,
    confidence NUMERIC(4,3) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_product_tags USING GIN (auto_tags);

CREATE TABLE ai_sla_predictions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL REFERENCES orders(id),
    risk_score NUMERIC(4,3) NOT NULL,
    predicted_delay_hours INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_sla_predictions (tenant_id, risk_score);

CREATE TABLE ai_cache_patterns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    url_pattern VARCHAR(255) NOT NULL,
    frequency INT NOT NULL,
    last_accessed TIMESTAMPTZ NOT NULL
  );
  CREATE INDEX ON ai_cache_patterns (tenant_id, frequency DESC);

CREATE TABLE ai_translations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    product_id UUID NOT NULL REFERENCES products(id),
    locale VARCHAR(10) NOT NULL,
    translated_content JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON ai_translations (product_id, locale);

CREATE TABLE ai_supply_risks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    supplier_id UUID NOT NULL,
    risk_score NUMERIC(4,3) NOT NULL,
    event_description TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_supply_risks (tenant_id, supplier_id);

CREATE TABLE ai_shipping_routes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    origin VARCHAR(20) NOT NULL,
    destination VARCHAR(20) NOT NULL,
    optimal_carrier VARCHAR(100) NOT NULL,
    cost DECIMAL(10,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_shipping_routes (origin, destination);

CREATE TABLE ai_gateway_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    gateway_name VARCHAR(50) NOT NULL,
    success_rate NUMERIC(4,3) NOT NULL,
    avg_latency_ms INT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE ai_cart_interventions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    cart_id UUID NOT NULL,
    intervention_type VARCHAR(50) NOT NULL,
    target_user UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_cart_interventions (cart_id);

CREATE TABLE ai_churn_predictions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    company_id UUID NOT NULL,
    risk_score NUMERIC(4,3) NOT NULL,
    factors JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_churn_predictions (tenant_id, risk_score);

CREATE TABLE ai_search_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    search_query TEXT NOT NULL,
    zero_results BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE ai_bulk_validations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    file_name VARCHAR(255) NOT NULL,
    error_count INT NOT NULL,
    auto_corrected_count INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE ai_reconciliations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    invoice_id UUID NOT NULL,
    po_id UUID NOT NULL,
    is_matched BOOLEAN NOT NULL,
    variance_details JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_reconciliations (tenant_id, is_matched);

CREATE TABLE ai_product_associations (
    primary_sku UUID NOT NULL,
    associated_sku UUID NOT NULL,
    lift_score NUMERIC(5,4) NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (primary_sku, associated_sku)
  );

CREATE TABLE ai_tenant_traffic_profiles (
    tenant_id UUID PRIMARY KEY REFERENCES tenants(id),
    avg_req_per_sec INT NOT NULL,
    burst_multiplier NUMERIC(3,2) NOT NULL,
    last_analyzed TIMESTAMPTZ NOT NULL
  );

CREATE TABLE ai_rma_decisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL,
    decision VARCHAR(50) NOT NULL,
    confidence NUMERIC(4,3) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_rma_decisions (tenant_id, decision);

CREATE TABLE ai_supplier_scores (
    supplier_id UUID PRIMARY KEY,
    composite_score NUMERIC(5,2) NOT NULL,
    metrics JSONB NOT NULL,
    calculated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE ai_prewarm_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    target_pattern VARCHAR(255) NOT NULL,
    status VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE ai_tax_classifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id),
    hs_code VARCHAR(20) NOT NULL,
    confidence NUMERIC(4,3) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_tax_classifications (hs_code);

CREATE TABLE ai_document_redactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    original_s3_key VARCHAR(255) NOT NULL,
    redacted_s3_key VARCHAR(255) NOT NULL,
    fields_removed TEXT[] NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE ai_watchdog_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_name VARCHAR(100) NOT NULL,
    action_taken VARCHAR(100) NOT NULL,
    reason TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE ai_swarm_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    swarm_id VARCHAR(64) UNIQUE NOT NULL,
    state JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ai_swarm_executions (tenant_id);

CREATE TABLE edge_model_deployments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    device_id VARCHAR(128) NOT NULL,
    model_version VARCHAR(32) NOT NULL,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON edge_model_deployments (tenant_id, device_id);

CREATE TABLE digital_twin_simulations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    parameters JSONB NOT NULL,
    results JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON digital_twin_simulations (tenant_id);

CREATE TABLE neural_render_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    product_id UUID NOT NULL,
    asset_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON neural_render_jobs (tenant_id, product_id);

CREATE TABLE b2b_graph_embeddings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    entity_id VARCHAR(128) NOT NULL,
    embedding vector(384),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON b2b_graph_embeddings USING hnsw (embedding vector_l2_ops);

CREATE TABLE federated_rounds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_id VARCHAR(64) NOT NULL,
    round_number INT NOT NULL,
    global_weights BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON federated_rounds (model_id, round_number);

CREATE TABLE rfp_analyses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    document_url TEXT NOT NULL,
    parsed_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON rfp_analyses (tenant_id);

CREATE TABLE product_auto_tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    product_id UUID NOT NULL,
    tags JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON product_auto_tags (tenant_id, product_id);

CREATE TABLE procurement_bots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(128) NOT NULL,
    rules JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON procurement_bots (tenant_id, sku);

CREATE TABLE contract_negotiations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    document_id UUID NOT NULL,
    version INT NOT NULL,
    ai_risk_score FLOAT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON contract_negotiations (tenant_id, document_id);

CREATE TABLE pricing_rl_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku_group VARCHAR(128) NOT NULL,
    model_weights BYTEA NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON pricing_rl_models (tenant_id, sku_group);

CREATE TABLE fraud_evaluations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    transaction_id VARCHAR(128) NOT NULL,
    risk_score FLOAT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON fraud_evaluations (tenant_id, risk_score);

CREATE TABLE iot_telemetry (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    device_id VARCHAR(128) NOT NULL,
    metrics JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  -- Using TimescaleDB extension for hypertable
  SELECT create_hypertable('iot_telemetry', 'created_at');

CREATE TABLE product_embeddings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    product_id UUID NOT NULL,
    embedding vector(768),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON product_embeddings USING hnsw (embedding vector_cosine_ops);

CREATE TABLE tax_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    jurisdiction VARCHAR(64) NOT NULL,
    rule_expression TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON tax_rules (tenant_id, jurisdiction);

CREATE TABLE analytics_sync_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sync_status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON analytics_sync_jobs (tenant_id);

CREATE TABLE inventory_events (
    id UUID DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(255) NOT NULL,
    delta INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  SELECT create_hypertable('inventory_events', 'created_at');
  CREATE INDEX ON inventory_events (tenant_id, sku, created_at DESC);

CREATE TABLE dashboard_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    socket_id VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON dashboard_subscriptions (tenant_id);

CREATE TABLE polars_query_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    query_payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON polars_query_logs (tenant_id);

CREATE TABLE flight_export_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    bytes_exported BIGINT DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON flight_export_jobs (tenant_id);

CREATE TABLE tenant_transformations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sql_template TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON tenant_transformations (tenant_id);

CREATE TABLE olap_cube_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    dimensions JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON olap_cube_definitions (tenant_id);

CREATE TABLE warehouse_allocations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    schema_name VARCHAR(255) UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON warehouse_allocations (tenant_id);

CREATE TABLE feature_store_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    feature_name VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON feature_store_metadata (tenant_id, feature_name);

CREATE TABLE event_schemas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    event_type VARCHAR(255) NOT NULL,
    schema_definition JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON event_schemas (tenant_id, event_type);

CREATE TABLE data_lineage_edges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    source_node VARCHAR(255) NOT NULL,
    target_node VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON data_lineage_edges (tenant_id, source_node);

CREATE TABLE query_cache_invalidation_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    table_mutated VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON query_cache_invalidation_log (tenant_id);

CREATE TABLE parquet_export_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    s3_path VARCHAR(512) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON parquet_export_tasks (tenant_id);

CREATE TABLE retention_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    table_name VARCHAR(255) NOT NULL,
    days_retained INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON retention_policies (tenant_id);

CREATE TABLE anomaly_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    metric_name VARCHAR(255) NOT NULL,
    severity VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON anomaly_alerts (tenant_id, created_at DESC);

CREATE TABLE data_quality_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    table_name VARCHAR(255) NOT NULL,
    score_percentage DECIMAL(5,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON data_quality_scores (tenant_id);

CREATE TABLE encryption_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    key_material BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON encryption_keys (tenant_id);

CREATE TABLE federation_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    remote_url VARCHAR(512) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON federation_configs (tenant_id);

CREATE TABLE products_vector (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    text_embedding VECTOR(384),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON products_vector USING hnsw (text_embedding vector_cosine_ops);

CREATE TABLE api_cost_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    max_cost_per_minute INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON api_cost_limits (tenant_id);

CREATE TABLE tenant_functions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    wasm_binary BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON tenant_functions (tenant_id);

CREATE TABLE system_traces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    trace_payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON system_traces (tenant_id, created_at DESC);

CREATE TABLE segmentation_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    rule_logic JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON segmentation_rules (tenant_id);

CREATE TABLE account_hierarchies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    path ltree NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON account_hierarchies USING gist (path);

CREATE TABLE industry_benchmarks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    industry_name VARCHAR(255) NOT NULL,
    metric_name VARCHAR(255) NOT NULL,
    noisy_value DECIMAL(10,4) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON industry_benchmarks (industry_name, metric_name);

CREATE TABLE idempotency_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    idempotency_key VARCHAR(255) UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON idempotency_logs (tenant_id, idempotency_key);

CREATE TABLE lead_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    lead_id UUID NOT NULL,
    score INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON lead_scores (tenant_id, lead_id);

CREATE TABLE report_generation_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    report_type VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON report_generation_tasks (tenant_id);

CREATE TABLE pii_vault (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    token VARCHAR(255) UNIQUE NOT NULL,
    encrypted_payload BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON pii_vault (tenant_id, token);

CREATE TABLE ledger_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    base_currency VARCHAR(3) NOT NULL,
    base_amount DECIMAL(19,4) NOT NULL,
    reporting_currency VARCHAR(3) NOT NULL,
    reporting_amount DECIMAL(19,4) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ledger_entries (tenant_id);

CREATE TABLE raw_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    event_type VARCHAR(255) NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON raw_events (tenant_id, event_type);

CREATE MATERIALIZED VIEW tenant_sales_summary AS
  SELECT
    tenant_id,
    SUM(total_amount) AS total_gmv,
    COUNT(id) AS order_count
  FROM orders
  GROUP BY tenant_id;
  CREATE UNIQUE INDEX ON tenant_sales_summary (tenant_id);

CREATE TABLE data_lake_exports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    dataset VARCHAR(50) NOT NULL,
    s3_uri TEXT NOT NULL,
    status VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON data_lake_exports (tenant_id);

CREATE TABLE schema_inferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    entity_name VARCHAR(100) NOT NULL,
    inferred_json_schema JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON schema_inferences (tenant_id);

CREATE TABLE tenant_quotas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    api_tier VARCHAR(50) NOT NULL,
    max_requests_per_hour INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON tenant_quotas (tenant_id);

CREATE TABLE deduplication_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    target_table VARCHAR(100) NOT NULL,
    duplicates_found INT DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON deduplication_jobs (tenant_id);

CREATE TABLE event_store (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_id UUID NOT NULL,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    event_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,
    version INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(aggregate_id, version)
  );
  CREATE INDEX ON event_store (tenant_id, aggregate_id);

CREATE TABLE clickstream_windows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    window_start TIMESTAMPTZ NOT NULL,
    window_end TIMESTAMPTZ NOT NULL,
    event_count INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON clickstream_windows (tenant_id, window_start);

CREATE TABLE query_performance_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    query_hash TEXT NOT NULL,
    avg_execution_time_ms FLOAT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON query_performance_logs (query_hash);

CREATE TABLE catalog_items (
    id UUID NOT NULL,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(255) NOT NULL,
    -- fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tenant_id, id)
  ) PARTITION BY LIST (tenant_id);
  -- Handled dynamically: CREATE TABLE catalog_items_t1 PARTITION OF catalog_items FOR VALUES IN ('uuid');

CREATE TABLE replication_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    target_region VARCHAR(50) NOT NULL,
    lsn_pointer BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON replication_logs (tenant_id, target_region);

CREATE TABLE etl_pipelines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    configuration JSONB NOT NULL,
    last_run TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON etl_pipelines (tenant_id);

CREATE TABLE invoice_exports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    byte_size BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON invoice_exports (tenant_id);

CREATE TABLE gmv_anomalies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    severity VARCHAR(20) NOT NULL,
    metrics JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON gmv_anomalies (tenant_id, created_at);

CREATE TABLE saga_states (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    saga_type VARCHAR(50) NOT NULL,
    current_step VARCHAR(50) NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON saga_states (tenant_id, current_step);

CREATE TABLE inventory_levels (
    time TIMESTAMPTZ NOT NULL,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(255) NOT NULL,
    quantity INT NOT NULL
  );
  -- TimescaleDB specific hypertable creation
  SELECT create_hypertable('inventory_levels', 'time');

CREATE TABLE pii_audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    entity_id UUID NOT NULL,
    fields_masked JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON pii_audit_logs (tenant_id);

CREATE TABLE identity_graphs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    graph_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON identity_graphs (tenant_id);

CREATE TABLE margin_calculations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    sku VARCHAR(255) NOT NULL,
    computed_margin DECIMAL(10,4) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON margin_calculations (tenant_id, computed_margin);

CREATE TABLE tenant_compute_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    compute_ms BIGINT NOT NULL,
    storage_bytes BIGINT NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON tenant_compute_usage (tenant_id, recorded_at);

CREATE TABLE ingestion_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    stream_name VARCHAR(100) NOT NULL,
    batch_size INT NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ingestion_logs (tenant_id, stream_name);

CREATE TABLE cache_predictions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    query_hash VARCHAR(255) NOT NULL,
    access_probability FLOAT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON cache_predictions (tenant_id, access_probability);

CREATE TABLE external_tables (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    table_name VARCHAR(100) NOT NULL,
    s3_path VARCHAR(500) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON external_tables (tenant_id, table_name);

CREATE TABLE cdc_checkpoints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    lsn VARCHAR(100) NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON cdc_checkpoints (tenant_id);

CREATE TABLE federated_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    source_name VARCHAR(100) NOT NULL,
    connection_uri VARCHAR(500) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON federated_sources (tenant_id, source_name);

CREATE TABLE etl_routing_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    priority_level INT NOT NULL,
    max_tps INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON etl_routing_rules (tenant_id);

CREATE TABLE tenant_schemas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    entity_name VARCHAR(100) NOT NULL,
    schema_definition JSONB NOT NULL,
    version INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON tenant_schemas (tenant_id, entity_name, version);

CREATE TABLE aggregation_views (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    view_name VARCHAR(100) NOT NULL,
    wasm_binary BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON aggregation_views (tenant_id, view_name);

CREATE TABLE vector_sync_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    product_id UUID NOT NULL,
    embedding_status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON vector_sync_logs (tenant_id, embedding_status);

CREATE TABLE event_store (
    sequence_id BIGSERIAL PRIMARY KEY,
    aggregate_id UUID NOT NULL,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    event_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON event_store (aggregate_id);

CREATE TABLE data_residency_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    allowed_regions TEXT[] NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON data_residency_policies (tenant_id);

CREATE TABLE token_vault (
    token_id VARCHAR(100) PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    encrypted_value BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON token_vault (tenant_id);

CREATE TABLE outbox_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    aggregate_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,
    published BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON outbox_events (published, created_at);

CREATE TABLE metric_baselines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    metric_name VARCHAR(100) NOT NULL,
    mean FLOAT NOT NULL,
    std_dev FLOAT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON metric_baselines (tenant_id, metric_name);

CREATE TABLE archival_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    records_archived INT NOT NULL,
    s3_key VARCHAR(500) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON archival_jobs (tenant_id);

CREATE TABLE clean_rooms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    initiator_tenant UUID NOT NULL REFERENCES tenants(id),
    partner_tenant UUID NOT NULL REFERENCES tenants(id),
    privacy_budget FLOAT NOT NULL DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON clean_rooms (initiator_tenant, partner_tenant);

CREATE TABLE fraud_feature_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    feature_name VARCHAR(100) NOT NULL,
    redis_key_pattern VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE data_lineage_edges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    source_node VARCHAR(200) NOT NULL,
    target_node VARCHAR(200) NOT NULL,
    transformation_logic TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON data_lineage_edges (tenant_id, target_node);

CREATE TABLE shard_routing (
    tenant_id UUID PRIMARY KEY REFERENCES tenants(id),
    physical_node VARCHAR(100) NOT NULL,
    is_migrating BOOLEAN DEFAULT FALSE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE edge_sync_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    device_id VARCHAR(100) NOT NULL,
    mutation_count INT NOT NULL,
    sync_time TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON edge_sync_logs (tenant_id, device_id);

CREATE TABLE affiliate_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    affiliate_id UUID NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    amount DECIMAL(12,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON affiliate_events (tenant_id, affiliate_id);

CREATE TABLE drip_nodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    campaign_id UUID NOT NULL,
    delay_seconds INT NOT NULL,
    template_id UUID NOT NULL,
    condition_sql TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON drip_nodes (tenant_id, campaign_id);

CREATE TABLE promotions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    code VARCHAR(50) NOT NULL UNIQUE,
    rules_script TEXT NOT NULL,
    max_uses INT,
    current_uses INT DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON promotions (tenant_id, code);

CREATE TABLE abm_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    domain VARCHAR(255) NOT NULL,
    firmographics JSONB,
    enrichment_status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON abm_accounts (tenant_id, domain);

CREATE TABLE lead_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    lead_id UUID NOT NULL,
    score INT NOT NULL,
    history JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON lead_scores (tenant_id, lead_id);

CREATE TABLE health_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID NOT NULL,
    score INT NOT NULL,
    risk_factors JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON health_scores (tenant_id, account_id);

CREATE TABLE nps_responses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID NOT NULL,
    score INT NOT NULL CHECK (score >= 0 AND score <= 10),
    feedback TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON nps_responses (tenant_id, account_id);

CREATE TABLE reengagement_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID NOT NULL,
    last_active_at TIMESTAMPTZ NOT NULL,
    campaign_triggered VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON reengagement_logs (tenant_id, account_id);

CREATE TABLE usage_triggers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID NOT NULL,
    metric VARCHAR(50) NOT NULL,
    triggered_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON usage_triggers (tenant_id, account_id, metric, (DATE(triggered_at)));

CREATE TABLE funnel_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL,
    stage VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON funnel_events (tenant_id, order_id, stage);

CREATE TABLE abandoned_carts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    cart_id UUID NOT NULL UNIQUE,
    account_id UUID NOT NULL,
    total DECIMAL(12,2) NOT NULL,
    abandoned_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON abandoned_carts (tenant_id, abandoned_at);

CREATE TABLE email_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(100) NOT NULL UNIQUE,
    subject_template TEXT NOT NULL,
    html_template TEXT NOT NULL,
    text_template TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON email_templates (tenant_id, name);

CREATE TABLE notification_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID NOT NULL,
    channel VARCHAR(20) NOT NULL,
    status VARCHAR(20) NOT NULL,
    external_id VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON notification_logs (tenant_id, account_id, channel);

CREATE TABLE segments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(100) NOT NULL,
    sql_definition TEXT NOT NULL,
    refresh_interval_minutes INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE TABLE segment_members (
    segment_id UUID REFERENCES segments(id),
    account_id UUID NOT NULL,
    PRIMARY KEY (segment_id, account_id)
  );

-- Handled dynamically via complex CTEs over the subscriptions table
  -- No dedicated table, but heavily reliant on indexed subscription logs
  CREATE INDEX ON subscriptions (tenant_id, date_trunc('month', created_at));
  CREATE INDEX ON subscription_events (subscription_id, event_type);

CREATE TABLE touchpoints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID NOT NULL,
    channel VARCHAR(50) NOT NULL,
    campaign_id UUID,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON touchpoints (tenant_id, account_id, occurred_at);

CREATE TABLE partner_deals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    partner_id UUID NOT NULL,
    client_company VARCHAR(255) NOT NULL,
    estimated_value DECIMAL(12,2),
    status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON partner_deals (tenant_id, partner_id);

-- Uses existing quotes and orders tables, relies heavily on transaction blocks
  CREATE TABLE quote_conversions (
    quote_id UUID PRIMARY KEY REFERENCES quotes(id),
    order_id UUID NOT NULL REFERENCES orders(id),
    converted_by UUID NOT NULL,
    converted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE price_elasticity_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL,
    base_price DECIMAL(12,2) NOT NULL,
    elasticity_coefficient DECIMAL(8,4) NOT NULL,
    calculated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON price_elasticity_models (product_id);

CREATE TABLE subscription_modifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    subscription_id UUID NOT NULL,
    old_plan VARCHAR(100),
    new_plan VARCHAR(100),
    prorated_amount DECIMAL(12,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON subscription_modifications (tenant_id, subscription_id);

CREATE TABLE churn_predictions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID NOT NULL REFERENCES accounts(id),
    probability DECIMAL(3,2) NOT NULL,
    factors JSONB NOT NULL,
    computed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON churn_predictions (tenant_id, probability DESC);

CREATE TABLE segments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(255) NOT NULL,
    rule_ast JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON segments (tenant_id);

CREATE TABLE loyalty_ledger (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID NOT NULL REFERENCES accounts(id),
    amount INT NOT NULL,
    balance_after INT NOT NULL,
    reference_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX no_double_spend ON loyalty_ledger (reference_id) WHERE amount < 0;

CREATE TABLE volume_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    product_id UUID NOT NULL,
    min_qty INT NOT NULL,
    max_qty INT,
    discount_pct DECIMAL(5,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON volume_rules (tenant_id, product_id);

CREATE MATERIALIZED VIEW rfm_scores AS
  SELECT 
    tenant_id, account_id,
    NTILE(5) OVER (PARTITION BY tenant_id ORDER BY MAX(created_at)) as recency,
    NTILE(5) OVER (PARTITION BY tenant_id ORDER BY COUNT(id)) as frequency,
    NTILE(5) OVER (PARTITION BY tenant_id ORDER BY SUM(total)) as monetary
  FROM orders
  GROUP BY tenant_id, account_id;
  CREATE UNIQUE INDEX ON rfm_scores (tenant_id, account_id);

CREATE TABLE abandoned_carts (
    cart_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    value DECIMAL(12,2) NOT NULL,
    recovery_status VARCHAR(50) DEFAULT 'pending',
    abandoned_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE quote_revisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    quote_id UUID NOT NULL,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    proposed_discount DECIMAL(5,2),
    status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON quote_revisions (quote_id, created_at DESC);

CREATE TABLE rep_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    rep_id UUID NOT NULL,
    account_id UUID NOT NULL,
    action_data JSONB NOT NULL,
    score DECIMAL(5,4) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON rep_actions (rep_id, score DESC);

-- ClickHouse Table
  CREATE TABLE abm_events (
    tenant_id UUID,
    account_id UUID,
    user_id UUID,
    event_type String,
    category String,
    timestamp DateTime
  ) ENGINE = MergeTree()
  ORDER BY (tenant_id, account_id, timestamp);

CREATE TABLE campaign_copy (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    prompt JSONB NOT NULL,
    generated_content JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE contract_prices (
    contract_id UUID REFERENCES contracts(id),
    sku VARCHAR(100) NOT NULL,
    price DECIMAL(10,2) NOT NULL,
    effective_from TIMESTAMPTZ NOT NULL,
    effective_to TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (contract_id, sku, effective_from)
  );
  CREATE INDEX ON contract_prices (sku);

CREATE TABLE clv_predictions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID NOT NULL REFERENCES accounts(id),
    historical_value DECIMAL(12,2) NOT NULL,
    predicted_value DECIMAL(12,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID NOT NULL,
    schedule_cron VARCHAR(50) NOT NULL,
    next_run TIMESTAMPTZ NOT NULL,
    status VARCHAR(20) DEFAULT 'active'
  );
  CREATE INDEX ON subscriptions (next_run) WHERE status = 'active';

CREATE TABLE nps_responses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    order_id UUID NOT NULL,
    score INT CHECK (score >= 0 AND score <= 10),
    feedback TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON nps_responses (tenant_id, score);

CREATE TABLE rep_commissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    rep_id UUID NOT NULL,
    order_id UUID NOT NULL,
    amount DECIMAL(12,2) NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE geo_promotions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    country_code VARCHAR(2) NOT NULL,
    promo_code VARCHAR(50) NOT NULL,
    active BOOLEAN DEFAULT TRUE
  );
  CREATE INDEX ON geo_promotions (country_code) WHERE active = TRUE;

CREATE TABLE account_hierarchies (
    parent_id UUID NOT NULL REFERENCES accounts(id),
    child_id UUID NOT NULL REFERENCES accounts(id),
    PRIMARY KEY (parent_id, child_id)
  );
  -- Handled via PostgreSQL Recursive CTEs

CREATE TABLE identity_edges (
    tenant_id UUID NOT NULL,
    node_a VARCHAR(255) NOT NULL, -- e.g. cookie:xyz
    node_b VARCHAR(255) NOT NULL, -- e.g. email:buyer@corp.com
    confidence DECIMAL(3,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON identity_edges (node_a);

CREATE TABLE dynamic_price_rules (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    sku VARCHAR(100) NOT NULL,
    base_price DECIMAL(10,2) NOT NULL,
    min_stock_threshold INT NOT NULL,
    surge_multiplier DECIMAL(3,2) NOT NULL
  );

CREATE TABLE referrals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    partner_id UUID NOT NULL,
    converted_order_id UUID,
    status VARCHAR(20) DEFAULT 'click',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

-- ClickHouse Materialized View
  CREATE MATERIALIZED VIEW retention_cube
  ENGINE = SummingMergeTree()
  ORDER BY (tenant_id, cohort_month, activity_month)
  AS SELECT
    tenant_id,
    toStartOfMonth(created_at) AS cohort_month,
    toStartOfMonth(order_date) AS activity_month,
    count(distinct account_id) AS active_accounts
  FROM orders
  GROUP BY tenant_id, cohort_month, activity_month;

CREATE TABLE product_associations (
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    base_sku VARCHAR(100) NOT NULL,
    suggested_sku VARCHAR(100) NOT NULL,
    lift_score DECIMAL(5,4) NOT NULL,
    PRIMARY KEY (tenant_id, base_sku, suggested_sku)
  );

CREATE TABLE email_suppressions (
    tenant_id UUID NOT NULL,
    email VARCHAR(255) NOT NULL,
    reason VARCHAR(50) NOT NULL,
    suppressed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tenant_id, email)
  );

CREATE TABLE account_preferences (
    account_id UUID PRIMARY KEY REFERENCES accounts(id),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    opt_in BOOLEAN NOT NULL DEFAULT FALSE,
    interests JSONB DEFAULT '[]',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE churn_predictions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON churn_predictions (tenant_id);

CREATE TABLE loyalty_tiers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON loyalty_tiers (tenant_id);

CREATE TABLE replenishments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON replenishments (tenant_id);

CREATE TABLE volume_discounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON volume_discounts (tenant_id);

CREATE TABLE quote_analytics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON quote_analytics (tenant_id);

CREATE TABLE clickstream_segments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON clickstream_segments (tenant_id);

CREATE TABLE cart_recoveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON cart_recoveries (tenant_id);

CREATE TABLE cross_sells (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON cross_sells (tenant_id);

CREATE TABLE lead_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON lead_scores (tenant_id);

CREATE TABLE rfm_clusters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON rfm_clusters (tenant_id);

CREATE TABLE approval_nudges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON approval_nudges (tenant_id);

CREATE TABLE contract_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON contract_alerts (tenant_id);

CREATE TABLE ltv_cohorts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ltv_cohorts (tenant_id);

CREATE TABLE catalog_indexes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON catalog_indexes (tenant_id);

CREATE TABLE promotion_budgets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON promotion_budgets (tenant_id);

CREATE TABLE referrals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON referrals (tenant_id);

CREATE TABLE net_terms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON net_terms (tenant_id);

CREATE TABLE campaigns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON campaigns (tenant_id);

CREATE TABLE crm_sync_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON crm_sync_logs (tenant_id);

CREATE TABLE data_forms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON data_forms (tenant_id);

CREATE TABLE affiliate_ledgers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON affiliate_ledgers (tenant_id);

CREATE TABLE csat_sentiments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON csat_sentiments (tenant_id);

CREATE TABLE review_solicitations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON review_solicitations (tenant_id);

CREATE TABLE wallets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON wallets (tenant_id);

CREATE TABLE event_tickets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    -- other fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON event_tickets (tenant_id);

