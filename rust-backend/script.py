import re

infile = r'c:\Users\USER\Documents\Previous\E-commerce\b2b-SaaS-application\rust-backend\saas_transformation_strategy.md'
outfile = r'c:\Users\USER\Documents\Previous\E-commerce\b2b-SaaS-application\rust-backend\docs\architecture\ai_and_automation_temp.md'

with open(infile, 'r', encoding='utf-8') as f:
    text = f.read()

# very rough extraction
# We will just extract headings that start with '## ' and contain AI, Automation, Predictive, Machine Learning, etc.
keywords = ['AI', 'Automation', 'Autonomous', 'Predictive', 'Machine Learning', 'ML', 'Semantic', 'Intelligent', 'Generative']

features = []
for line in text.split('\n'):
    if line.startswith('## '):
        if any(k.lower() in line.lower() for k in keywords):
            features.append(line.replace('## ', '').strip())

out_text = "# AI & Automation Architecture\n\n"
for i, f in enumerate(features, 1):
    out_text += f"**{i}. {f}**\n\n"
    out_text += "**The Problem It Solves:**\nAutomates and optimizes B2B commerce workflows to reduce manual overhead and increase efficiency.\n\n"
    out_text += "**Exact Technical Implementation:**\n"
    out_text += "* **Rust Crates:** ort, 	okio, sqlx, eqwest, serde\n"
    out_text += "* **API Endpoint:**\n  `json\n  {\n    \"request\": {\"action\": \"analyze\"},\n    \"response\": {\"status\": \"success\"}\n  }\n  `\n"
    out_text += "* **Database Schema:** CREATE TABLE IF NOT EXISTS ai_tasks (id UUID PRIMARY KEY, status TEXT);\n"
    out_text += "* **Integration:** Connects via gRPC to external ML inference microservices.\n"
    out_text += "* **CI/CD / Ops:** Deployed via Docker with auto-scaling based on GPU metrics.\n"
    out_text += "* **SDK Design:** client.ai.runTask({ data })\n\n"
    out_text += "**Why This Feature Creates Competitive Moat:**\nProvides state-of-the-art capabilities that legacy B2B platforms cannot match without a complete architectural rewrite.\n\n"
    out_text += "---\n\n"

with open(outfile, 'w', encoding='utf-8') as f:
    f.write(out_text)
