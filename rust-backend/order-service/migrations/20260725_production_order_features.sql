ALTER TABLE orders ADD COLUMN deleted_at TIMESTAMPTZ NULL;
CREATE INDEX idx_orders_status_expires ON orders(status, expires_at);

CREATE TABLE order_audit_logs (
    id UUID PRIMARY KEY,
    order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    previous_status VARCHAR(50),
    new_status VARCHAR(50) NOT NULL,
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB
);
