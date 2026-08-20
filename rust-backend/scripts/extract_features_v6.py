import os
import re
from collections import defaultdict

base_path = r"c:\Users\USER\Documents\Previous\E-commerce\b2b-SaaS-application\rust-backend"
docs_path = os.path.join(base_path, "docs", "architecture")

domain_map = {
    "ai_and_automation.md": "analytics",
    "b2b_commerce_workflows.md": "order-service",
    "catalog_and_inventory.md": "product-catalog",
    "data_engineering.md": "analytics",
    "developer_experience.md": "platform",
    "fintech_and_billing.md": "payments",
    "growth_and_crm.md": "analytics",
    "infrastructure_and_sre.md": "platform",
    "logistics_and_supply_chain.md": "logistics",
    "marketplace_and_multivendor.md": "supplier-management",
    "notifications_and_communications.md": "notifications",
    "observability_and_ops.md": "platform",
    "security_and_compliance.md": "tenant-management",
    "tenant_management.md": "tenant-management"
}

# Allow optional leading spaces before ```sql and ```
sql_pattern = re.compile(r'```sql\s*\n(.*?)\n\s*```', re.DOTALL)
json_pattern = re.compile(r'```(?:json|typescript)\s*\n(.*?)\n\s*```', re.DOTALL)

crate_sqls = defaultdict(list)
crate_structs = defaultdict(list)

for filename in os.listdir(docs_path):
    if filename not in domain_map:
        continue
        
    crate = domain_map[filename]
    filepath = os.path.join(docs_path, filename)
    
    with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
        content = f.read()
        
    for sql in sql_pattern.findall(content):
        clean = sql.strip()
        if clean and "CREATE" in clean.upper():
            crate_sqls[crate].append(clean)
            
    for json_txt in json_pattern.findall(content):
        clean = json_txt.strip()
        if clean and ("{" in clean or "client" in clean):
            crate_structs[crate].append(clean)

total_sqls = 0
for crate, sql_statements in crate_sqls.items():
    if not sql_statements:
        continue
        
    migrations_dir = os.path.join(base_path, crate, "migrations")
    os.makedirs(migrations_dir, exist_ok=True)
    
    mig_file = os.path.join(migrations_dir, "20260810_800_features_foundation.sql")
    
    with open(mig_file, "w", encoding="utf-8") as f:
        f.write("-- Auto-generated foundation from 800+ feature architecture blueprints\n\n")
        seen = set()
        for sql in sql_statements:
            if sql not in seen:
                seen.add(sql)
                f.write(sql + "\n\n")
                total_sqls += 1

total_structs = 0
for crate, structs in crate_structs.items():
    if not structs:
        continue
    
    models_file = os.path.join(base_path, crate, "src", "blueprint_models.rs")
    os.makedirs(os.path.dirname(models_file), exist_ok=True)
    
    with open(models_file, "w", encoding="utf-8") as f:
        f.write("// Auto-generated foundational structs from blueprints\n")
        f.write("// These must be integrated into models.rs manually\n\n")
        f.write("use serde::{Serialize, Deserialize};\n\n")
        seen = set()
        for i, struct_json in enumerate(structs):
            if struct_json not in seen:
                seen.add(struct_json)
                # Escape comments properly
                f.write(f"/* Blueprint API Payload {i}:\n{struct_json}\n*/\n\n")
                total_structs += 1

print(f"Successfully extracted {total_sqls} SQL tables and {total_structs} JSON API stubs across {len(crate_sqls)} crates.")
