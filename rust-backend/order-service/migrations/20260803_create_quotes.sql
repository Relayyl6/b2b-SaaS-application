-- B2B Quote Approval workflow tables
CREATE TABLE b2b_quotes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    status VARCHAR(50) NOT NULL,
    total_amount DECIMAL NOT NULL
);
