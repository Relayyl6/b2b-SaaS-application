-- Supplier agreement/commission table
CREATE TABLE supplier_contracts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    supplier_id UUID NOT NULL REFERENCES suppliers(id),
    commission_rate DECIMAL NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
