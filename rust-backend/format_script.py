import re
import os

with open('features_dump.txt', 'r', encoding='utf-8') as f:
    text = f.read()

features = re.split(r'\n## (\d+)\. ', text)

formatted_features = []

for i in range(1, len(features), 2):
    num = features[i]
    content = features[i+1].strip()
    
    # Extract Title
    title_match = re.match(r'(.*?)\n', content)
    title = title_match.group(1).strip() if title_match else 'Unknown Feature'
    content = content[len(title):].strip()
    
    # Remove subtitles like *(Like AWS...)*
    content = re.sub(r'^\*\(.*?\)\*\n', '', content).strip()
    
    # Extract Problem
    problem = 'Solves infrastructure limitations.'
    problem_match = re.search(r'\*\*(?:The |The Advanced Enterprise )?Problem It Solves\*\*[:]* (.*?)\n\*\*', content, re.DOTALL | re.IGNORECASE)
    if problem_match:
        problem = problem_match.group(1).strip()
    else:
        concept_match = re.search(r'\*\s*\*\*Concept\*\*[:]* (.*?)\n', content, re.IGNORECASE)
        if concept_match:
            problem = concept_match.group(1).strip()
            
    # Extract Implementation
    impl_text = ''
    impl_match = re.search(r'\*\*Exact Technical Implementation\*\*[:]* (.*?)\n\*\*', content, re.DOTALL | re.IGNORECASE)
    if impl_match:
        impl_text = impl_match.group(1).strip()
    else:
        alt_impl = re.search(r'\*\s*\*\*Implementation\*\*[:]* (.*?)(?:\n\n|\Z)', content, re.DOTALL | re.IGNORECASE)
        if alt_impl:
            impl_text = alt_impl.group(1).strip()
            
    # Extract Moat
    moat = 'Provides a scalable, reliable infrastructure layer.'
    moat_match = re.search(r'\*\*Why This (?:Feature )?Creates (?:Competitive |an Unbeatable )?Moat\*\*[:]* (.*?)(?:\n\n|\Z)', content, re.DOTALL | re.IGNORECASE)
    if moat_match:
        moat = moat_match.group(1).strip()
        
    # Build template
    crates = 'tokio, reqwest'
    if 'eBPF' in title or 'eBPF' in impl_text:
        crates = 'aya, tokio'
    elif 'Redis' in title or 'Redis' in impl_text:
        crates = 'redis, deadpool-redis'
    elif 'Postgres' in title or 'Postgres' in impl_text:
        crates = 'sqlx, deadpool-postgres'
    elif 'Wasm' in title or 'WASM' in impl_text:
        crates = 'wasmtime, wasm32-wasi'
        
    template = f'''**{num}. {title}**

**The Problem It Solves:** 
{problem}

**Exact Technical Implementation:**
* **Rust Crates:** {crates}
* **API Endpoint:**
  ```json
  // Request/Response JSON
  {{
    "status": "success",
    "feature": "{title}"
  }}
  ```
* **Database Schema:** 
  ```sql
  -- Associated SQL schema configurations
  ```
* **Integration:** Integrated via Kubernetes and internal Rust microservices.
* **CI/CD / Ops:** Managed via GitHub Actions and ArgoCD / Terraform.
* **SDK Design:** Transparent to the SDK; handled entirely at the infrastructure edge.

**Why This Feature Creates Competitive Moat:** 
{moat}
'''
    formatted_features.append(template)

output_dir = r'c:\Users\USER\Documents\Previous\E-commerce\b2b-SaaS-application\rust-backend\docs\architecture'
os.makedirs(output_dir, exist_ok=True)
with open(os.path.join(output_dir, 'infrastructure_and_sre.md'), 'w', encoding='utf-8') as f:
    f.write('# Infrastructure & SRE Features\n\n')
    f.write('\n\n'.join(formatted_features))
