import os
from collections import defaultdict

base_path = r"c:\Users\USER\Documents\Previous\E-commerce\b2b-SaaS-application\rust-backend"
docs_path = os.path.join(base_path, "docs", "architecture")

crate_sqls = defaultdict(list)
crate_apis = defaultdict(list)

for filename in os.listdir(docs_path):
    if not filename.endswith(".md"):
        continue
        
    filepath = os.path.join(docs_path, filename)
    with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
        lines = f.readlines()
        
    current_crate = None
    in_sql = False
    in_api = False
    sql_buffer = []
    api_buffer = []
    
    for line in lines:
        if "**Rust Crates:**" in line:
            # Extract crate name, e.g. **Rust Crates:** `order-service`
            parts = line.split("`")
            if len(parts) >= 3:
                current_crate = parts[1].strip()
        
        # Handle SQL extraction
        if line.startswith("```sql"):
            in_sql = True
            sql_buffer = []
            continue
        if in_sql and line.startswith("```"):
            in_sql = False
            if current_crate and current_crate != "specific-crate" and "platform" not in current_crate:
                crate_sqls[current_crate].append("".join(sql_buffer).strip())
            continue
        if in_sql:
            sql_buffer.append(line)
            
        # Handle JSON/TypeScript API extraction
        if line.startswith("```json") or line.startswith("```typescript"):
            in_api = True
            api_buffer = []
            continue
        if in_api and line.startswith("```"):
            in_api = False
            if current_crate and current_crate != "specific-crate" and "platform" not in current_crate:
                crate_apis[current_crate].append("".join(api_buffer).strip())
            continue
        if in_api:
            api_buffer.append(line)

total_sqls = 0
for crate, sqls in crate_sqls.items():
    if not sqls: continue
    migrations_dir = os.path.join(base_path, crate, "migrations")
    os.makedirs(migrations_dir, exist_ok=True)
    
    mig_file = os.path.join(migrations_dir, "20260810_800_features_foundation.sql")
    with open(mig_file, "w", encoding="utf-8") as f:
        f.write("-- Auto-generated SQL schema foundation for Blueprint Features\n\n")
        seen = set()
        for sql in sqls:
            if sql not in seen and sql:
                seen.add(sql)
                f.write(sql + "\n\n")
                total_sqls += 1

for crate, apis in crate_apis.items():
    if not apis: continue
    models_file = os.path.join(base_path, crate, "src", "blueprint_models.rs")
    os.makedirs(os.path.dirname(models_file), exist_ok=True)
    
    with open(models_file, "w", encoding="utf-8") as f:
        f.write("// Auto-generated foundational structs from blueprints\n")
        f.write("// These must be integrated into models.rs manually\n\n")
        f.write("use serde::{Serialize, Deserialize};\n\n")
        seen = set()
        for i, api in enumerate(apis):
            if api not in seen and api:
                seen.add(api)
                f.write(f"/* Blueprint API Payload {i}:\n{api}\n*/\n\n")

print(f"Distributed {total_sqls} SQL schemas to {len(crate_sqls)} crates safely.")
