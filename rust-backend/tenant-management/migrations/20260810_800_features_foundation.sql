-- Auto-generated foundation from 800+ feature architecture blueprints

CREATE TABLE api_scan_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    endpoints_found INT NOT NULL,
    vulnerabilities JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON api_scan_results (tenant_id, created_at);

CREATE TABLE workload_identities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    spiffe_id VARCHAR NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON workload_identities (tenant_id, spiffe_id);

CREATE TABLE tenant_kms_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    kms_arn VARCHAR NOT NULL,
    data_key_ciphertext BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON tenant_kms_configs (tenant_id);

CREATE TABLE audit_merkle_roots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    root_hash VARCHAR NOT NULL,
    block_number BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON audit_merkle_roots (tenant_id, block_number);

CREATE TABLE anomaly_detections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    risk_score FLOAT NOT NULL,
    ip_address INET,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON anomaly_detections (tenant_id, risk_score);

CREATE TABLE gdpr_deletion_sagas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL,
    services_completed JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON gdpr_deletion_sagas (tenant_id, user_id);

CREATE TABLE hsm_operations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    operation_type VARCHAR NOT NULL,
    key_slot INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON hsm_operations (tenant_id, operation_type);

CREATE TABLE pqc_enforcements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    enforce_hybrid_tls BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON pqc_enforcements (tenant_id);

CREATE TABLE rasp_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    syscall_blocked VARCHAR NOT NULL,
    process_id INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON rasp_events (tenant_id, syscall_blocked);

CREATE TABLE internal_certificates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_name VARCHAR NOT NULL,
    cert_serial VARCHAR NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON internal_certificates (service_name);

CREATE TABLE rate_limit_tiers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    tier_name VARCHAR NOT NULL,
    req_per_sec INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON rate_limit_tiers (tenant_id);

CREATE TABLE revoked_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    jti UUID NOT NULL UNIQUE,
    revoked_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON revoked_tokens (jti);

CREATE TABLE vault_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    role_name VARCHAR NOT NULL,
    ttl_seconds INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON vault_roles (tenant_id);

CREATE TABLE ci_security_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    commit_sha VARCHAR NOT NULL,
    vulnerabilities JSONB,
    passed BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ci_security_reports (commit_sha);

CREATE TABLE cde_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    token VARCHAR NOT NULL UNIQUE,
    encrypted_pan BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON cde_tokens (tenant_id, token);

CREATE TABLE patient_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    encrypted_dob BYTEA NOT NULL,
    encrypted_ssn BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON patient_records (tenant_id);

CREATE TABLE soc2_evidence (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    control_id VARCHAR NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON soc2_evidence (tenant_id, control_id);

CREATE TABLE tls_pqc_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    kem_algorithm VARCHAR(50) NOT NULL,
    sig_algorithm VARCHAR(50) NOT NULL,
    client_ip INET NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON tls_pqc_sessions (tenant_id);

CREATE TABLE cdn_asset_sri_hashes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    asset_path VARCHAR(255) NOT NULL,
    hash_value VARCHAR(128) NOT NULL,
    version VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON cdn_asset_sri_hashes (tenant_id);

CREATE TABLE tenant_cors_origins (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    origin_url VARCHAR(255) NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON tenant_cors_origins (tenant_id);

CREATE TABLE tenant_hsts_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    max_age BIGINT NOT NULL,
    include_subdomains BOOLEAN DEFAULT TRUE,
    preload BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON tenant_hsts_policies (tenant_id);

CREATE TABLE egress_domain_allowlist (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    domain_name VARCHAR(255) NOT NULL,
    justification TEXT,
    approved_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON egress_domain_allowlist (tenant_id);

CREATE TABLE data_export_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    actor_id UUID NOT NULL,
    record_count INT NOT NULL,
    export_type VARCHAR(50) NOT NULL,
    risk_score FLOAT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON data_export_events (actor_id);

CREATE TABLE merkle_audit_ledger (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    leaf_hash VARCHAR(64) NOT NULL,
    parent_hash VARCHAR(64) NOT NULL,
    payload_cid VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON merkle_audit_ledger (leaf_hash);

CREATE TABLE tenant_iam_boundaries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    aws_role_arn VARCHAR(255) NOT NULL,
    policy_document JSONB NOT NULL,
    status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON tenant_iam_boundaries (tenant_id);

CREATE TABLE presigned_url_tracking (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    object_key VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    accessed_count INT DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON presigned_url_tracking (object_key);

CREATE TABLE pii_masking_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    regex_pattern TEXT NOT NULL,
    entity_type VARCHAR(50) NOT NULL,
    mask_char CHAR(1) DEFAULT '*',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON pii_masking_rules (tenant_id);

CREATE TABLE enclave_attestation_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    pcr_measurements JSONB NOT NULL,
    nonce VARCHAR(64) NOT NULL,
    verified_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON enclave_attestation_reports (tenant_id);

CREATE TABLE crypto_operation_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    operation_type VARCHAR(50) NOT NULL,
    execution_time_ns BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON crypto_operation_logs (operation_type);

CREATE TABLE ws_handshake_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    nonce VARCHAR(64) NOT NULL UNIQUE,
    origin VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ws_handshake_tokens (nonce);

CREATE TABLE service_mtls_pins (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_name VARCHAR(100) NOT NULL,
    cert_fingerprint VARCHAR(255) NOT NULL,
    valid_until TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON service_mtls_pins (service_name, cert_fingerprint);

CREATE TABLE compliance_framework_status (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    framework_name VARCHAR(50) NOT NULL,
    control_id VARCHAR(50) NOT NULL,
    passing_status BOOLEAN NOT NULL,
    evidence_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON compliance_framework_status (tenant_id, framework_name);

CREATE TABLE data_classification_tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    table_name VARCHAR(100) NOT NULL,
    column_name VARCHAR(100) NOT NULL,
    classification_level VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON data_classification_tags (table_name, column_name);

CREATE TABLE vendor_risk_assessments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    vendor_name VARCHAR(100) NOT NULL,
    soc2_report_url TEXT,
    risk_score FLOAT NOT NULL,
    assessment_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON vendor_risk_assessments (tenant_id);

CREATE TABLE tenant_security_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    score_value INT NOT NULL,
    vulnerability_count INT NOT NULL,
    last_calculated_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON tenant_security_scores (tenant_id);

CREATE TABLE access_revocation_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    employee_id UUID NOT NULL,
    system_name VARCHAR(100) NOT NULL,
    revoked_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON access_revocation_logs (employee_id);

CREATE TABLE service_account_credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    client_id VARCHAR(100) NOT NULL,
    secret_hash VARCHAR(255) NOT NULL,
    rotated_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON service_account_credentials (client_id);

CREATE TABLE pg_audit_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    statement_type VARCHAR(50) NOT NULL,
    object_type VARCHAR(50) NOT NULL,
    query_text TEXT NOT NULL,
    db_user VARCHAR(50) NOT NULL,
    executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON pg_audit_events (executed_at);

CREATE TABLE formal_verification_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    proof_target VARCHAR(100) NOT NULL,
    kani_version VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL,
    completed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON formal_verification_runs (proof_target);

CREATE TABLE ebpf_syscall_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pod_name VARCHAR(100) NOT NULL,
    syscall_id INT NOT NULL,
    arguments TEXT NOT NULL,
    risk_level VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ebpf_syscall_alerts (pod_name);

CREATE TABLE key_ceremony_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ceremony_id UUID NOT NULL,
    participant_id UUID NOT NULL,
    key_share_hash VARCHAR(255) NOT NULL,
    signed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON key_ceremony_events (ceremony_id, participant_id);

CREATE TABLE mpc_signing_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_hash VARCHAR(255) NOT NULL,
    participating_nodes JSONB NOT NULL,
    final_signature TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON mpc_signing_sessions (message_hash);

CREATE TABLE fhe_analytics_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    dataset_id UUID NOT NULL,
    operation_type VARCHAR(50) NOT NULL,
    encrypted_result BYTEA,
    status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON fhe_analytics_jobs (tenant_id);

CREATE TABLE verifiable_credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    subject_did VARCHAR(255) NOT NULL,
    credential_type VARCHAR(100) NOT NULL,
    jws_signature TEXT NOT NULL,
    revoked BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON verifiable_credentials (tenant_id, subject_did);

CREATE TABLE ephemeral_key_audits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    key_purpose VARCHAR(50) NOT NULL,
    ttl_seconds INT NOT NULL,
    destroyed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ephemeral_key_audits (tenant_id);

CREATE TABLE canary_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    token_value VARCHAR(255) UNIQUE NOT NULL,
    trap_type VARCHAR(50) NOT NULL,
    memo TEXT,
    triggered BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON canary_tokens (token_value);

CREATE TABLE zkp_verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    vk_id VARCHAR(50) NOT NULL,
    proof_hash VARCHAR(255) NOT NULL,
    is_valid BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON zkp_verifications (tenant_id);

CREATE TABLE red_team_simulations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    scenario VARCHAR(100) NOT NULL,
    target_env VARCHAR(100) NOT NULL,
    findings JSONB,
    status VARCHAR(20) DEFAULT 'running',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE ci_review_gates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    pr_number INT NOT NULL,
    repository VARCHAR(100) NOT NULL,
    security_approver VARCHAR(100),
    is_compliant BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ci_review_gates (pr_number, repository);

CREATE TABLE key_escrow_deposits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    key_id VARCHAR(100) NOT NULL,
    encrypted_shares JSONB NOT NULL,
    deposited_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON key_escrow_deposits (tenant_id);

CREATE TABLE geofencing_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    allowed_countries JSONB NOT NULL,
    action VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON geofencing_rules (tenant_id);

CREATE TABLE regulatory_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    framework VARCHAR(50) NOT NULL,
    region VARCHAR(50) NOT NULL,
    retention_days INT NOT NULL,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE security_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    report_type VARCHAR(50) NOT NULL,
    s3_key VARCHAR(255) NOT NULL,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON security_reports (tenant_id);

CREATE TABLE risk_exposures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    pii_count INT NOT NULL,
    estimated_fine_usd DECIMAL(15, 2) NOT NULL,
    calculated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON risk_exposures (tenant_id);

CREATE TABLE privacy_assessments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    project_name VARCHAR(255) NOT NULL,
    risk_score INT NOT NULL,
    status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON privacy_assessments (tenant_id);

CREATE TABLE bug_bounty_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_id VARCHAR(100) UNIQUE NOT NULL,
    title VARCHAR(255) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    status VARCHAR(20) DEFAULT 'open',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE phishing_campaigns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    target_email VARCHAR(255) NOT NULL,
    template_id VARCHAR(50) NOT NULL,
    clicked BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON phishing_campaigns (tenant_id);

CREATE TABLE fraud_signals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    signal_hash VARCHAR(255) UNIQUE NOT NULL,
    signal_type VARCHAR(50) NOT NULL,
    occurrences INT DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON fraud_signals (signal_hash);

CREATE TABLE dns_audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_name VARCHAR(100) NOT NULL,
    query_domain VARCHAR(255) NOT NULL,
    resolved_ip VARCHAR(50),
    blocked BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE secret_scan_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_url VARCHAR(255) NOT NULL,
    commit_hash VARCHAR(40) NOT NULL,
    secrets_found INT NOT NULL,
    scanned_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE build_verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    release_version VARCHAR(50) NOT NULL,
    binary_hash VARCHAR(64) NOT NULL,
    is_reproducible BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE iso_risk_register (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asset_id VARCHAR(100) NOT NULL,
    threat_description TEXT NOT NULL,
    mitigation_status VARCHAR(50) NOT NULL,
    residual_risk_score INT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE erasure_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    kms_key_id VARCHAR(100) NOT NULL,
    erased_by UUID NOT NULL,
    erased_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE opa_violations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_kind VARCHAR(50) NOT NULL,
    resource_name VARCHAR(100) NOT NULL,
    violation_reason TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE api_deprecations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint_path VARCHAR(255) NOT NULL,
    sunset_date TIMESTAMPTZ NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON api_deprecations (endpoint_path);

CREATE TABLE security_dashboard_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    metric_name VARCHAR(50) NOT NULL,
    metric_value INT NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON security_dashboard_metrics (tenant_id, recorded_at);

CREATE TABLE threat_indicators (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    indicator_type VARCHAR(50) NOT NULL,
    indicator_value VARCHAR(255) UNIQUE NOT NULL,
    threat_score INT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON threat_indicators (indicator_value);

CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_id UUID REFERENCES tenants(id),
    path ltree NOT NULL,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON tenants USING GIST (path);

CREATE TABLE tenant_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(100) NOT NULL,
    permissions TEXT[] NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON tenant_roles (tenant_id);

CREATE TABLE abac_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    policy_document TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON abac_policies (tenant_id);

CREATE TABLE sso_configurations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    provider_type VARCHAR(50) NOT NULL,
    idp_metadata XML,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON sso_configurations (tenant_id);

CREATE TABLE scim_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    external_id VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON scim_users (tenant_id, external_id);

CREATE TABLE onboarding_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    task_name VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON onboarding_tasks (tenant_id, status);

CREATE TABLE custom_domains (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    domain_name VARCHAR(255) NOT NULL UNIQUE,
    ssl_status VARCHAR(50) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON custom_domains (domain_name);

CREATE TABLE billing_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    metric_name VARCHAR(100) NOT NULL,
    quantity NUMERIC NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  SELECT create_hypertable('billing_events', 'recorded_at');

CREATE TABLE tenant_health_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    score INTEGER NOT NULL CHECK (score >= 0 AND score <= 100),
    calculated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON tenant_health_scores (tenant_id, calculated_at DESC);

CREATE TABLE data_regions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    region VARCHAR(50) NOT NULL,
    db_connection_string VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON data_regions (tenant_id);

CREATE TABLE tenant_feature_flags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    flag_key VARCHAR(100) NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON tenant_feature_flags (tenant_id, flag_key);

CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    key_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    is_revoked BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON api_keys (tenant_id);

CREATE TABLE tenant_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_tenant_id UUID NOT NULL REFERENCES tenants(id),
    target_tenant_id UUID NOT NULL REFERENCES tenants(id),
    resource_type VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON tenant_shares (source_tenant_id, target_tenant_id);

CREATE TABLE tenant_quotas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    resource_name VARCHAR(100) NOT NULL,
    max_limit INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON tenant_quotas (tenant_id, resource_name);

CREATE TABLE data_exports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    s3_url TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON data_exports (tenant_id);

CREATE TABLE webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    target_url TEXT NOT NULL,
    signing_secret VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON webhooks (tenant_id);

CREATE TABLE tenant_rate_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    requests_per_second INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON tenant_rate_limits (tenant_id);

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    actor_id UUID NOT NULL,
    action VARCHAR(255) NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON audit_logs (tenant_id, created_at DESC);

CREATE TABLE tenant_slas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    uptime_target NUMERIC(5,4) NOT NULL,
    maintenance_window_start TIME,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON tenant_slas (tenant_id);

CREATE TABLE tenant_lockdowns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    reason TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON tenant_lockdowns (tenant_id);

CREATE EXTENSION IF NOT EXISTS ltree;
  CREATE TABLE organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    parent_id UUID REFERENCES organizations(id),
    name VARCHAR(255) NOT NULL,
    path ltree NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX org_path_idx ON organizations USING GIST (path);
  CREATE INDEX ON organizations (tenant_id);

CREATE TABLE policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    subject VARCHAR(255) NOT NULL,
    resource VARCHAR(255) NOT NULL,
    action VARCHAR(255) NOT NULL,
    condition JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON policies (tenant_id, subject);

CREATE TABLE sso_configurations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    idp_entity_id VARCHAR(255) NOT NULL,
    metadata_xml TEXT,
    claims_mapping JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON sso_configurations (tenant_id);

CREATE TABLE login_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL,
    ip_address INET,
    risk_score FLOAT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON login_events (tenant_id, user_id, created_at);

CREATE TABLE revoked_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    jti VARCHAR(255) NOT NULL UNIQUE,
    revoked_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON revoked_tokens (tenant_id);

CREATE TABLE customers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  ALTER TABLE customers ENABLE ROW LEVEL SECURITY;
  CREATE POLICY tenant_isolation_policy ON customers
    USING (tenant_id = current_setting('app.current_tenant')::UUID);

CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(255) NOT NULL,
    key_hash VARCHAR(255) NOT NULL,
    scopes JSONB NOT NULL,
    quota_rpm INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON api_keys (tenant_id);

-- ClickHouse Table
  CREATE TABLE access_logs (
    tenant_id UUID,
    user_id UUID,
    endpoint String,
    bytes_transferred UInt64,
    timestamp DateTime
  ) ENGINE = MergeTree() ORDER BY (tenant_id, timestamp);

CREATE TABLE jit_grants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL,
    resource VARCHAR(255) NOT NULL,
    reason TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON jit_grants (tenant_id, expires_at);

CREATE TABLE claims_scripts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    rhai_script TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON claims_scripts (tenant_id);

CREATE TABLE impersonation_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    actor_id UUID NOT NULL,
    target_id UUID NOT NULL,
    reason TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON impersonation_sessions (tenant_id);

CREATE TABLE webauthn_credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL,
    passkey_id BYTEA NOT NULL UNIQUE,
    public_key BYTEA NOT NULL,
    sign_count INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON webauthn_credentials (tenant_id, user_id);

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    actor_id UUID NOT NULL,
    action VARCHAR(255) NOT NULL,
    metadata JSONB,
    hash VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON audit_logs (tenant_id, created_at);

CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  -- All other tables must filter out deleted tenants

CREATE TABLE identity_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    local_user_id UUID NOT NULL,
    external_provider VARCHAR(255) NOT NULL,
    external_subject_id VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON identity_links (external_provider, external_subject_id);

CREATE TABLE password_policies (
    tenant_id UUID PRIMARY KEY REFERENCES tenants(id),
    min_zxcvbn_score INT NOT NULL DEFAULT 3,
    require_mfa_below_score INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE consent_receipts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL,
    purpose VARCHAR(255) NOT NULL,
    granted BOOLEAN NOT NULL,
    ip_address INET,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON consent_receipts (tenant_id, user_id);

CREATE TABLE custom_idps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    provider_type VARCHAR(50) NOT NULL,
    client_id VARCHAR(255) NOT NULL,
    client_secret TEXT NOT NULL,
    discovery_url TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON custom_idps (tenant_id);

CREATE TABLE oauth_clients (
    client_id VARCHAR(255) PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    client_secret_hash VARCHAR(255) NOT NULL,
    allowed_scopes JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON oauth_clients (tenant_id);

CREATE TABLE user_activity_metrics (
    user_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    last_login_at TIMESTAMPTZ,
    last_order_at TIMESTAMPTZ,
    ml_stale_probability FLOAT DEFAULT 0.0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON user_activity_metrics (tenant_id, ml_stale_probability);

-- Primarily relies on Redis for speed, Postgres for audit
  CREATE TABLE session_invalidations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL,
    invalidated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE network_policies (
    tenant_id UUID PRIMARY KEY REFERENCES tenants(id),
    allowed_cidrs JSONB,
    allowed_countries JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    url TEXT NOT NULL,
    events JSONB NOT NULL,
    signing_secret VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON webhooks (tenant_id);

CREATE TABLE context_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    action VARCHAR(255) NOT NULL,
    require_mfa_if_unrecognized_device BOOLEAN DEFAULT TRUE,
    restrict_to_business_hours BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

-- Global Lookup Table (Replicated globally)
  CREATE TABLE tenant_routing (
    tenant_id UUID PRIMARY KEY,
    domain VARCHAR(255) NOT NULL UNIQUE,
    region VARCHAR(50) NOT NULL,
    db_connection_string TEXT NOT NULL
  );

CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    region VARCHAR(64) NOT NULL,
    database_url_ref VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON tenants (region);

CREATE TABLE b2b_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    parent_id UUID REFERENCES b2b_accounts(id),
    name VARCHAR(255) NOT NULL,
    credit_limit DECIMAL(15,2),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON b2b_accounts (tenant_id, parent_id);

CREATE TABLE role_insights (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL REFERENCES users(id),
    suggested_role VARCHAR(64) NOT NULL,
    confidence_score FLOAT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON role_insights (tenant_id);

CREATE TABLE sso_configurations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    provider VARCHAR(64) NOT NULL,
    client_id VARCHAR(255) NOT NULL,
    encrypted_client_secret TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON sso_configurations (tenant_id);

CREATE TABLE rate_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    requests_per_second INT NOT NULL,
    burst_size INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON rate_limits (tenant_id);

CREATE TABLE migrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    source_db VARCHAR(255) NOT NULL,
    target_db VARCHAR(255) NOT NULL,
    status VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON migrations (tenant_id);

CREATE TABLE ephemeral_tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_tenant_id UUID NOT NULL REFERENCES tenants(id),
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON ephemeral_tenants (parent_tenant_id);

CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    hashed_key VARCHAR(255) NOT NULL,
    scopes JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON api_keys USING GIN (scopes);

CREATE TABLE delegated_admins (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID NOT NULL REFERENCES b2b_accounts(id),
    user_id UUID NOT NULL REFERENCES users(id),
    permissions JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON delegated_admins (account_id, user_id);

CREATE TABLE mfa_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL REFERENCES users(id),
    risk_score FLOAT NOT NULL,
    mfa_triggered BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON mfa_events (user_id);

CREATE TABLE encryption_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    aws_kms_arn VARCHAR(255) NOT NULL,
    data_key_ciphertext BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON encryption_configs (tenant_id);

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    actor_id UUID NOT NULL,
    action VARCHAR(255) NOT NULL,
    payload JSONB NOT NULL,
    previous_hash VARCHAR(64) NOT NULL,
    hash VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON audit_logs (tenant_id, created_at);

CREATE TABLE identity_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL REFERENCES users(id),
    token_hash VARCHAR(255) NOT NULL,
    type VARCHAR(32) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON identity_tokens (token_hash);

CREATE TABLE tenant_quotas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    resource_type VARCHAR(64) NOT NULL,
    max_limit BIGINT NOT NULL,
    current_usage BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON tenant_quotas (tenant_id, resource_type);

CREATE TABLE data_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_tenant_id UUID NOT NULL REFERENCES tenants(id),
    target_tenant_id UUID NOT NULL REFERENCES tenants(id),
    resource_type VARCHAR(64) NOT NULL,
    filters JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON data_shares (source_tenant_id, target_tenant_id);

CREATE TABLE jit_mapping_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    idp_group_name VARCHAR(255) NOT NULL,
    platform_role VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON jit_mapping_rules (tenant_id);

CREATE TABLE jwt_configurations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    claim_mappings JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON jwt_configurations (tenant_id);

CREATE TABLE security_lockouts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL REFERENCES users(id),
    trigger_reason TEXT NOT NULL,
    resolved BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON security_lockouts (user_id, resolved);

CREATE TABLE ip_allowlists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    cidr_blocks JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON ip_allowlists (tenant_id);

CREATE TABLE session_revocations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    revoked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reason VARCHAR(255) NOT NULL
  );
  CREATE INDEX ON session_revocations (tenant_id);

CREATE TABLE user_tenant_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    role VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON user_tenant_links (user_id, tenant_id);

CREATE TABLE approval_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    script_content TEXT NOT NULL,
    priority INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON approval_rules (tenant_id, priority);

CREATE TABLE global_threat_intel (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ip_address VARCHAR(45) NOT NULL,
    threat_score FLOAT NOT NULL,
    banned_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON global_threat_intel (ip_address);

CREATE TABLE m2m_identities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    service_name VARCHAR(255) NOT NULL,
    allowed_scopes JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE UNIQUE INDEX ON m2m_identities (tenant_id, service_name);

CREATE TABLE webhook_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    event_type VARCHAR(255) NOT NULL,
    payload JSONB NOT NULL,
    endpoint VARCHAR(255) NOT NULL,
    status VARCHAR(64) NOT NULL,
    retry_count INT NOT NULL DEFAULT 0,
    next_retry_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  CREATE INDEX ON webhook_deliveries (tenant_id, status);

