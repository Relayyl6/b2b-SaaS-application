import os

base_path = r"c:\Users\USER\Documents\Previous\E-commerce\b2b-SaaS-application\rust-backend"

# --- 12. infra/ ---
infra_path = os.path.join(base_path, "infra")
k8s_path = os.path.join(infra_path, "k8s")
obs_path = os.path.join(infra_path, "observability")
graf_path = os.path.join(obs_path, "grafana")
pg_path = os.path.join(infra_path, "postgres")

os.makedirs(k8s_path, exist_ok=True)
os.makedirs(graf_path, exist_ok=True)
os.makedirs(pg_path, exist_ok=True)

with open(os.path.join(k8s_path, "deployment.yaml"), "w") as f:
    f.write("# Stub Deployment\n")
with open(os.path.join(k8s_path, "service.yaml"), "w") as f:
    f.write("# Stub Service\n")
with open(os.path.join(k8s_path, "configmap.yaml"), "w") as f:
    f.write("# Stub ConfigMap\n")
with open(os.path.join(k8s_path, "ingress.yaml"), "w") as f:
    f.write("# Stub Ingress\n")
with open(os.path.join(k8s_path, "hpa.yaml"), "w") as f:
    f.write("# Stub HPA\n")

with open(os.path.join(graf_path, "dashboard.json"), "w") as f:
    f.write("{}\n")
with open(os.path.join(obs_path, "alerts.yaml"), "w") as f:
    f.write("# Prometheus Alert Rules\n")

with open(os.path.join(pg_path, "rls_policy_template.sql"), "w") as f:
    f.write("""-- Canonical RLS Policy Template
ALTER TABLE {table_name} ENABLE ROW LEVEL SECURITY;
CREATE POLICY {table_name}_isolation_policy ON {table_name}
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);
""")

# --- 13. .github/workflows/ ---
gh_path = os.path.join(base_path, ".github", "workflows")
os.makedirs(gh_path, exist_ok=True)

with open(os.path.join(gh_path, "deploy.yml"), "w") as f:
    f.write("# CD Pipeline for K8s deployment\n")
with open(os.path.join(gh_path, "migration.yml"), "w") as f:
    f.write("# Automated SQLx migration testing\n")
with open(os.path.join(gh_path, "security.yml"), "w") as f:
    f.write("# Cargo audit + deny\n")

# --- 14. e2e-tests/ & gateway-tests/ ---
e2e_path = os.path.join(base_path, "e2e-tests", "src")
gw_path = os.path.join(base_path, "gateway-tests", "src")
os.makedirs(e2e_path, exist_ok=True)
os.makedirs(gw_path, exist_ok=True)

with open(os.path.join(e2e_path, "test_tenant_isolation.rs"), "w") as f:
    f.write("#[cfg(test)]\nmod tests {\n    #[ignore]\n    #[test]\n    fn test_tenant_isolation() {}\n}\n")
with open(os.path.join(e2e_path, "test_payment_flow.rs"), "w") as f:
    f.write("#[cfg(test)]\nmod tests {\n    #[ignore]\n    #[test]\n    fn test_payment_flow() {}\n}\n")
with open(os.path.join(gw_path, "api_contract_tests.rs"), "w") as f:
    f.write("#[cfg(test)]\nmod tests {\n    #[ignore]\n    #[test]\n    fn test_api_contract() {}\n}\n")

print("Phase 4 Infra & Tooling scaffolded successfully.")
