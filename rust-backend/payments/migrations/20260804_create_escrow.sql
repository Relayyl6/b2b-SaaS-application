-- Escrow holding account tables
CREATE TABLE escrow_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_id UUID NOT NULL,
    amount DECIMAL NOT NULL,
    status VARCHAR(50) NOT NULL
);
