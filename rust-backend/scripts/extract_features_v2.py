import os
import re
from collections import defaultdict
import shutil

base_path = r"c:\Users\USER\Documents\Previous\E-commerce\b2b-SaaS-application\rust-backend"
docs_path = os.path.join(base_path, "docs", "architecture")

# Remove the incorrectly generated platform migrations
platform_mig = os.path.join(base_path, "platform", "migrations")
if os.path.exists(platform_mig):
    shutil.rmtree(platform_mig)

crate_sqls = defaultdict(list)
crate_structs = defaultdict(list)

# Regex to find features:
# Looks for **Rust Crates:** `crate-name`
# followed by ```sql ... ```
feature_pattern = re.compile(
    r'\*\*\s*Rust Crates:\s*\*\*\s*`([a-zA-Z0-9_-]+)`.*?```sql\n(.*?)\n```',
    re.DOTALL | re.IGNORECASE
)

api_pattern = re.compile(
    r'\*\*\s*Rust Crates:\s*\*\*\s*`([a-zA-Z0-9_-]+)`.*?```(?:json|typescript)\n(.*?)\n```',
    re.DOTALL | re.IGNORECASE
)

for filename in os.listdir(docs_path):
    if not filename.endswith(".md"):
        continue
        
    filepath = os.path.join(docs_path, filename)
    with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
        content = f.read()
        
    # Extract SQL
    for match in feature_pattern.finditer(content):
        crate = match.group(1).lower()
        sql = match.group(2).strip()
        if "platform" not in crate and "Specific-crate" not in crate: # Ignore template string
            crate_sqls[crate].append(sql)

    # Extract JSON APIs to turn into structs (rough conceptual mapping)
    for match in api_pattern.finditer(content):
        crate = match.group(1).lower()
        api_text = match.group(2).strip()
        if "platform" not in crate and "Specific-crate" not in crate:
            crate_structs[crate].append(api_text)


for crate, sql_statements in crate_sqls.items():
    if not sql_statements:
        continue
        
    migrations_dir = os.path.join(base_path, crate, "migrations")
    os.makedirs(migrations_dir, exist_ok=True)
    
    mig_file = os.path.join(migrations_dir, "20260810_800_features_foundation.sql")
    
    with open(mig_file, "w", encoding="utf-8") as f:
        f.write("-- Auto-generated SQL schema foundation for Blueprint Features\n\n")
        seen = set()
        for sql in sql_statements:
            if sql not in seen:
                seen.add(sql)
                f.write(sql + "\n\n")

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
                f.write(f"/* Blueprint API Payload {i}:\n{struct_json}\n*/\n\n")

print(f"Distributed SQL schemas to {len(crate_sqls)} crates.")
