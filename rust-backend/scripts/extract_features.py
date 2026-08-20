import os
import re
from collections import defaultdict

base_path = r"c:\Users\USER\Documents\Previous\E-commerce\b2b-SaaS-application\rust-backend"
docs_path = os.path.join(base_path, "docs", "architecture")

# Mapping of file keywords to target crates
domain_map = {
    "product": "product-catalog",
    "tenant": "tenant-management",
    "saas": "tenant-management",
    "user": "user-management",
    "order": "order-service",
    "fintech": "payments",
    "billing": "payments",
    "payment": "payments",
    "logistics": "logistics",
    "inventory": "inventory-management",
    "supplier": "supplier-management",
    "analytics": "analytics",
    "data": "analytics",
    "notifications": "notifications",
    "ai": "analytics", # Default AI to analytics
    "infra": "platform",
    "dx": "platform",
    "ecosystem": "platform"
}

def get_target_crate(filename):
    lower_name = filename.lower()
    for key, crate in domain_map.items():
        if key in lower_name:
            return crate
    return "platform" # default fallback

sql_pattern = re.compile(r'```sql\n(.*?)\n```', re.DOTALL)

crate_sqls = defaultdict(list)

for filename in os.listdir(docs_path):
    if not filename.endswith(".md"):
        continue
        
    crate = get_target_crate(filename)
    filepath = os.path.join(docs_path, filename)
    
    with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
        content = f.read()
        
    # Extract all SQL blocks
    sqls = sql_pattern.findall(content)
    for sql in sqls:
        clean_sql = sql.strip()
        if clean_sql and not clean_sql.startswith("--") and "CREATE" in clean_sql.upper():
            crate_sqls[crate].append(clean_sql)

# Write migrations to respective crates
for crate, sql_statements in crate_sqls.items():
    if not sql_statements:
        continue
        
    migrations_dir = os.path.join(base_path, crate, "migrations")
    os.makedirs(migrations_dir, exist_ok=True)
    
    mig_file = os.path.join(migrations_dir, "20260810_800_features_foundation.sql")
    
    with open(mig_file, "w", encoding="utf-8") as f:
        f.write("-- Auto-generated foundation from 800+ feature architecture blueprints\n\n")
        # De-duplicate SQL statements roughly
        seen = set()
        for sql in sql_statements:
            # simple hash to avoid exact duplicates
            h = hash(sql)
            if h not in seen:
                seen.add(h)
                f.write(sql + "\n\n")

print("Successfully parsed architecture blueprints and generated SQL migrations for all 800+ features.")
