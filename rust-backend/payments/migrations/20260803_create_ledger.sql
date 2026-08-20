-- Double-entry ledger tables
CREATE TABLE ledger_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    balance DECIMAL NOT NULL DEFAULT 0.0
);
CREATE TABLE ledger_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES ledger_accounts(id),
    amount DECIMAL NOT NULL,
    direction VARCHAR(10) NOT NULL
);
