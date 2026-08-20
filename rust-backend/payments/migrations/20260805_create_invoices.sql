-- B2B Invoice tables
CREATE TABLE b2b_invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    status VARCHAR(50) NOT NULL,
    due_date TIMESTAMPTZ NOT NULL
);
